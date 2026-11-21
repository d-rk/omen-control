//! Reading and writing the driver's files. Every file is optional.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::thermal;
use super::values::*;
use super::{Error, Paths};

/// Lighting values as read from `rgb_zones/`. `None` means the file is missing or unreadable.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LightingState {
    pub zones: [Option<Rgb>; ZONE_COUNT],
    pub has_all: bool,
    pub brightness: Option<u8>,
    pub brightness_lock: Option<bool>,
    pub mode: Option<AnimationMode>,
    pub speed: Option<u8>,
    pub gradient: Option<Vec<GradientGroup>>,
}

/// Fan values as read from `fan/`, the thermal zones and the platform profile.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FanState {
    pub cpu_rpm: Option<u32>,
    pub gpu_rpm: Option<u32>,
    pub max_rpm: Option<MaxRpm>,
    pub max_fan: Option<bool>,
    pub thermal_profile: Option<ThermalProfile>,
    pub power_limits: Option<PowerLimits>,
    pub cpu_curve: Option<Vec<CurvePoint>>,
    pub gpu_curve: Option<GpuCurve>,
    pub curve_enable: Option<bool>,
    pub cpu_temp: Option<f32>,
    pub gpu_temp: Option<f32>,
    pub platform_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    paths: Paths,
}

impl Device {
    pub fn new(paths: Paths) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    pub fn driver_present(&self) -> bool {
        self.paths.driver_present()
    }

    pub fn has_rgb(&self, file: &str) -> bool {
        self.paths.rgb(file).exists()
    }

    pub fn has_fan(&self, file: &str) -> bool {
        self.paths.fan(file).exists()
    }

    // -- reads --------------------------------------------------------------------------------------------------------

    pub fn read_lighting(&self) -> LightingState {
        let p = &self.paths;
        LightingState {
            zones: std::array::from_fn(|i| read_parsed(&p.rgb(&format!("zone{i:02}")), str::parse)),
            has_all: p.rgb("all").exists(),
            brightness: read_parsed(&p.rgb("brightness"), |s| s.trim().parse::<u8>().map_err(|_| ())),
            brightness_lock: read_parsed(&p.rgb("brightness_lock"), parse_bool01),
            mode: read_parsed(&p.rgb("animation_mode"), str::parse),
            speed: read_parsed(&p.rgb("animation_speed"), |s| s.trim().parse::<u8>().map_err(|_| ())),
            gradient: read_parsed(&p.rgb("gradient_config"), parse_gradient_config),
        }
    }

    pub fn read_fans(&self) -> FanState {
        let p = &self.paths;
        let live = self.read_live();
        FanState {
            max_rpm: read_parsed(&p.fan("max_rpm"), parse_max_rpm).flatten(),
            max_fan: read_parsed(&p.fan("max_fan"), parse_bool01),
            thermal_profile: read_parsed(&p.fan("thermal_profile"), str::parse),
            power_limits: read_parsed(&p.fan("power_limits"), parse_power_limits),
            cpu_curve: read_parsed(&p.fan("fan_curve"), parse_curve),
            gpu_curve: read_parsed(&p.fan("gpu_fan_curve"), parse_gpu_curve),
            curve_enable: read_parsed(&p.fan("fan_curve_enable"), parse_bool01),
            ..live
        }
    }

    /// The values that change on their own: RPM, temperatures, platform profile.
    pub fn read_live(&self) -> FanState {
        let p = &self.paths;
        let cpu_zone = read_text(&p.fan("fan_temp_zone"));
        let gpu_zone = read_text(&p.fan("gpu_fan_temp_zone"));
        FanState {
            cpu_rpm: read_parsed(&p.fan("cpu_fan_rpm"), parse_rpm).flatten(),
            gpu_rpm: read_parsed(&p.fan("gpu_fan_rpm"), parse_rpm).flatten(),
            cpu_temp: thermal::cpu_temp(&p.thermal, cpu_zone.as_deref()),
            gpu_temp: thermal::gpu_temp(&p.thermal, gpu_zone.as_deref(), cpu_zone.as_deref()),
            platform_profile: read_text(&p.platform_profile).map(|s| s.trim().to_owned()),
            ..FanState::default()
        }
    }

    // -- writes -------------------------------------------------------------------------------------------------------

    pub fn write_zone(&self, zone: usize, color: Rgb) -> Result<(), Error> {
        write(&self.paths.rgb(&format!("zone{zone:02}")), &color.to_string())
    }

    pub fn write_all_zones(&self, color: Rgb) -> Result<(), Error> {
        write(&self.paths.rgb("all"), &color.to_string())
    }

    pub fn write_brightness(&self, value: u8) -> Result<(), Error> {
        write(&self.paths.rgb("brightness"), &value.min(100).to_string())
    }

    pub fn write_brightness_lock(&self, on: bool) -> Result<(), Error> {
        write(&self.paths.rgb("brightness_lock"), bool01(on))
    }

    pub fn write_mode(&self, mode: AnimationMode) -> Result<(), Error> {
        write(&self.paths.rgb("animation_mode"), mode.as_str())
    }

    pub fn write_speed(&self, speed: u8) -> Result<(), Error> {
        write(&self.paths.rgb("animation_speed"), &speed.clamp(1, 10).to_string())
    }

    pub fn write_gradient(&self, groups: &[GradientGroup]) -> Result<(), Error> {
        for g in groups {
            validate_gradient_group(g).map_err(|reason| ParseError {
                what: "gradient config",
                reason: reason.into(),
                value: format_gradient_config(std::slice::from_ref(g)),
            })?;
        }
        write(&self.paths.rgb("gradient_config"), &format_gradient_config(groups))
    }

    pub fn write_thermal_profile(&self, profile: ThermalProfile) -> Result<(), Error> {
        write(&self.paths.fan("thermal_profile"), profile.as_str())
    }

    pub fn write_max_fan(&self, on: bool) -> Result<(), Error> {
        write(&self.paths.fan("max_fan"), bool01(on))
    }

    pub fn write_curve_enable(&self, on: bool) -> Result<(), Error> {
        write(&self.paths.fan("fan_curve_enable"), bool01(on))
    }

    pub fn write_cpu_curve(&self, points: &[CurvePoint]) -> Result<(), Error> {
        write(&self.paths.fan("fan_curve"), &format_curve(points)?)
    }

    /// `None` makes the GPU fan follow the CPU curve again.
    pub fn write_gpu_curve(&self, points: Option<&[CurvePoint]>) -> Result<(), Error> {
        let text = match points {
            Some(p) => format_curve(p)?,
            None => "cpu".to_owned(),
        };
        write(&self.paths.fan("gpu_fan_curve"), &text)
    }

    pub fn write_power_limits(&self, limits: PowerLimits) -> Result<(), Error> {
        let text = format_power_limits(limits).ok_or_else(|| ParseError {
            what: "power limits",
            reason: "not supported by this machine".into(),
            value: String::new(),
        })?;
        write(&self.paths.fan("power_limits"), &text)
    }
}

fn bool01(on: bool) -> &'static str {
    if on { "1" } else { "0" }
}

fn read_text(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => {
            eprintln!("omen-control: reading {}: {e}", path.display());
            None
        }
    }
}

fn read_parsed<T, E: std::fmt::Debug>(path: &Path, parse: impl FnOnce(&str) -> Result<T, E>) -> Option<T> {
    let text = read_text(path)?;
    match parse(&text) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("omen-control: parsing {}: {e:?}", path.display());
            None
        }
    }
}

fn write(path: &Path, text: &str) -> Result<(), Error> {
    // sysfs store functions want one write; `fs::write` does a single write for these short values.
    fs::write(path, text).map_err(|source| Error::Io { path: PathBuf::from(path), source })
}

#[cfg(test)]
mod tests {
    use super::super::mock::MockTree;
    use super::*;

    /// Scenario: Missing file is unavailable
    #[test]
    fn missing_file_is_unavailable() {
        let mock = MockTree::create().unwrap();
        fs::remove_file(mock.paths().fan("gpu_fan_rpm")).unwrap();
        let dev = Device::new(mock.paths().clone());
        let fans = dev.read_fans();
        assert_eq!(fans.gpu_rpm, None);
        assert!(fans.cpu_rpm.is_some());
        assert!(fans.thermal_profile.is_some());
        assert!(fans.cpu_curve.is_some());
    }

    /// Scenario: Failed write reports file
    #[test]
    fn failed_write_reports_file() {
        let mock = MockTree::create().unwrap();
        let dev = Device::new(mock.paths().clone());
        // A directory where the file should be makes the write fail.
        let target = mock.paths().fan("max_fan");
        fs::remove_file(&target).unwrap();
        fs::create_dir(&target).unwrap();
        let err = dev.write_max_fan(true).unwrap_err();
        assert!(err.to_string().contains("max_fan"), "{err}");
    }

    /// Scenario: Platform profile shown
    #[test]
    fn platform_profile_shown() {
        let mock = MockTree::create().unwrap();
        fs::write(&mock.paths().platform_profile, "balanced\n").unwrap();
        let dev = Device::new(mock.paths().clone());
        assert_eq!(dev.read_live().platform_profile.as_deref(), Some("balanced"));
    }
}
