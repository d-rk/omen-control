//! Named settings profiles, one TOML file each.

use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::sysfs::{
    self, AnimationMode, CurvePoint, Device, GpuCurve, GradientGroup, PowerLimits, Rgb, ThermalProfile, ZONE_COUNT,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid profile name: {0}")]
    InvalidName(&'static str),
    #[error("profile {0:?} not found")]
    NotFound(String),
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: std::io::Error },
    #[error("{}: {source}", path.display())]
    Decode { path: PathBuf, source: toml::de::Error },
    #[error("encoding profile: {0}")]
    Encode(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LightingSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zones: Option<[Rgb; ZONE_COUNT]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brightness: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<AnimationMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gradient: Option<Vec<GradientGroup>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuCurveSetting {
    FollowsCpu,
    Own(Vec<CurvePoint>),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FanSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thermal_profile: Option<ThermalProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_fan: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curve_enable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_curve: Option<Vec<CurvePoint>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_curve: Option<GpuCurveSetting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power_limits: Option<PowerLimits>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub lighting: LightingSettings,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fans: Option<FanSettings>,
}

impl Profile {
    /// The device's current settings; fan settings only if `include_fans`.
    pub fn capture(dev: &Device, include_fans: bool) -> Profile {
        let l = dev.read_lighting();
        let zones = (l.zones.iter().all(Option::is_some)).then(|| l.zones.map(|z| z.unwrap_or_default()));
        let lighting =
            LightingSettings { zones, brightness: l.brightness, mode: l.mode, speed: l.speed, gradient: l.gradient };
        let fans = include_fans.then(|| {
            let f = dev.read_fans();
            FanSettings {
                thermal_profile: f.thermal_profile,
                max_fan: f.max_fan,
                curve_enable: f.curve_enable,
                cpu_curve: f.cpu_curve.filter(|c| !c.is_empty()),
                gpu_curve: f.gpu_curve.map(|g| match g {
                    GpuCurve::FollowsCpu => GpuCurveSetting::FollowsCpu,
                    GpuCurve::Own(p) => GpuCurveSetting::Own(p),
                }),
                power_limits: f.power_limits.filter(|p| *p != PowerLimits::Unsupported),
            }
        });
        Profile { lighting, fans }
    }

    /// Writes the profile to the device. Values whose file is missing are skipped; every failed write is
    /// returned and does not stop the others.
    pub fn apply(&self, dev: &Device) -> Vec<sysfs::Error> {
        let mut errors = Vec::new();
        let mut run = |r: Result<(), sysfs::Error>| {
            if let Err(e) = r {
                errors.push(e);
            }
        };
        let l = &self.lighting;
        if let Some(g) = &l.gradient
            && dev.has_rgb("gradient_config")
        {
            run(dev.write_gradient(g));
        }
        if let Some(zones) = l.zones {
            for (i, c) in zones.into_iter().enumerate() {
                if dev.has_rgb(&format!("zone{i:02}")) {
                    run(dev.write_zone(i, c));
                }
            }
        }
        if let Some(b) = l.brightness
            && dev.has_rgb("brightness")
        {
            run(dev.write_brightness(b));
        }
        if let Some(s) = l.speed
            && dev.has_rgb("animation_speed")
        {
            run(dev.write_speed(s));
        }
        if let Some(m) = l.mode
            && dev.has_rgb("animation_mode")
        {
            run(dev.write_mode(m));
        }

        if let Some(f) = &self.fans {
            if let Some(p) = f.thermal_profile
                && dev.has_fan("thermal_profile")
            {
                run(dev.write_thermal_profile(p));
            }
            if let Some(c) = &f.cpu_curve
                && dev.has_fan("fan_curve")
            {
                run(dev.write_cpu_curve(c));
            }
            if let Some(g) = &f.gpu_curve
                && dev.has_fan("gpu_fan_curve")
            {
                run(match g {
                    GpuCurveSetting::FollowsCpu => dev.write_gpu_curve(None),
                    GpuCurveSetting::Own(p) => dev.write_gpu_curve(Some(p)),
                });
            }
            // After the curves, because writing a CPU curve turns the switch on.
            if let Some(on) = f.curve_enable
                && dev.has_fan("fan_curve_enable")
            {
                run(dev.write_curve_enable(on));
            }
            if let Some(on) = f.max_fan
                && dev.has_fan("max_fan")
            {
                run(dev.write_max_fan(on));
            }
            if let Some(p) = f.power_limits
                && dev.has_fan("power_limits")
            {
                run(dev.write_power_limits(p));
            }
        }
        errors
    }
}

/// Checks a profile name and returns it trimmed.
pub fn validate_name(name: &str) -> Result<&str, Error> {
    let n = name.trim();
    if n.is_empty() {
        return Err(Error::InvalidName("name is empty"));
    }
    if n.chars().count() > 64 {
        return Err(Error::InvalidName("name is longer than 64 characters"));
    }
    if n.starts_with('.') {
        return Err(Error::InvalidName("name starts with '.'"));
    }
    if n.chars().any(|c| c == '/' || c == '\\' || c.is_control()) {
        return Err(Error::InvalidName("name contains '/', '\\' or a control character"));
    }
    Ok(n)
}

#[derive(Debug, Clone)]
pub struct ProfileStore {
    dir: PathBuf,
}

impl ProfileStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// `~/.config/omen-control/profiles` (XDG config dir).
    pub fn default_location() -> Option<ProfileStore> {
        let dirs = directories::ProjectDirs::from("", "", "omen-control")?;
        Some(ProfileStore::new(dirs.config_dir().join("profiles")))
    }

    fn file(&self, name: &str) -> Result<PathBuf, Error> {
        Ok(self.dir.join(format!("{}.toml", validate_name(name)?)))
    }

    /// Profile names, sorted case-insensitively.
    pub fn list(&self) -> Result<Vec<String>, Error> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(e) => e,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(Error::Io { path: self.dir.clone(), source });
            }
        };
        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "toml"))
            .filter_map(|p| p.file_stem()?.to_str().map(str::to_owned))
            .filter(|n| validate_name(n).is_ok())
            .collect();
        names.sort_by_key(|n| n.to_lowercase());
        Ok(names)
    }

    pub fn save(&self, name: &str, profile: &Profile) -> Result<(), Error> {
        let path = self.file(name)?;
        let text = toml::to_string_pretty(profile)?;
        fs::create_dir_all(&self.dir).map_err(|source| Error::Io { path: self.dir.clone(), source })?;
        fs::write(&path, text).map_err(|source| Error::Io { path, source })
    }

    pub fn load(&self, name: &str) -> Result<Profile, Error> {
        let path = self.file(name)?;
        let text = fs::read_to_string(&path).map_err(|source| match source.kind() {
            ErrorKind::NotFound => Error::NotFound(name.to_owned()),
            _ => Error::Io { path: path.clone(), source },
        })?;
        toml::from_str(&text).map_err(|source| Error::Decode { path, source })
    }

    pub fn delete(&self, name: &str) -> Result<(), Error> {
        let path = self.file(name)?;
        fs::remove_file(&path).map_err(|source| match source.kind() {
            ErrorKind::NotFound => Error::NotFound(name.to_owned()),
            _ => Error::Io { path, source },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysfs::mock::MockTree;

    fn full_profile() -> Profile {
        Profile {
            lighting: LightingSettings {
                zones: Some([Rgb::new(255, 0, 0), Rgb::new(0, 255, 0), Rgb::new(0, 0, 255), Rgb::WHITE]),
                brightness: Some(70),
                mode: Some(AnimationMode::Gradient),
                speed: Some(3),
                gradient: Some(vec![GradientGroup { zones: vec![0, 1], colors: vec![Rgb::WHITE, Rgb::BLACK] }]),
            },
            fans: Some(FanSettings {
                thermal_profile: Some(ThermalProfile::Performance),
                max_fan: Some(false),
                curve_enable: Some(true),
                cpu_curve: Some(vec![CurvePoint::new(40, 0), CurvePoint::new(90, 100)]),
                gpu_curve: Some(GpuCurveSetting::Own(vec![CurvePoint::new(50, 10), CurvePoint::new(85, 90)])),
                power_limits: Some(PowerLimits::Custom { pl1: 65, pl2: 90 }),
            }),
        }
    }

    /// Scenario: Save and load round trip
    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(dir.path().join("profiles"));
        let p = full_profile();
        store.save("gaming", &p).unwrap();
        assert_eq!(store.load("gaming").unwrap(), p);
        let follows = Profile {
            fans: Some(FanSettings { gpu_curve: Some(GpuCurveSetting::FollowsCpu), ..Default::default() }),
            ..Default::default()
        };
        store.save("quiet", &follows).unwrap();
        assert_eq!(store.load("quiet").unwrap(), follows);
    }

    /// Scenario: List profiles sorted
    #[test]
    fn list_profiles_sorted() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(dir.path().to_owned());
        for n in ["Work", "gaming", "Night"] {
            store.save(n, &Profile::default()).unwrap();
        }
        assert_eq!(store.list().unwrap(), ["gaming", "Night", "Work"]);
    }

    /// Scenario: Delete profile
    #[test]
    fn delete_profile() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(dir.path().to_owned());
        store.save("Night", &Profile::default()).unwrap();
        store.delete("Night").unwrap();
        assert!(store.list().unwrap().is_empty());
    }

    /// Scenario: Reject path in name
    #[test]
    fn reject_path_in_name() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(dir.path().join("profiles"));
        assert!(matches!(store.save("../evil", &Profile::default()), Err(Error::InvalidName(_))));
        assert!(!dir.path().join("evil.toml").exists());
        assert!(!dir.path().join("profiles").exists());
    }

    /// Scenario: Profile without fan settings leaves fans alone
    #[test]
    fn profile_without_fan_settings_leaves_fans_alone() {
        let mock = MockTree::create().unwrap();
        let dev = Device::new(mock.paths().clone());
        let fan_dir = mock.paths().driver.join("fan");
        let before: Vec<_> = snapshot(&fan_dir);
        let p = Profile { fans: None, ..full_profile() };
        assert!(p.apply(&dev).is_empty());
        assert_eq!(snapshot(&fan_dir), before);
        assert_eq!(dev.read_lighting().brightness, Some(70));
    }

    /// Scenario: Apply continues after a failed write
    #[test]
    fn apply_continues_after_a_failed_write() {
        let mock = MockTree::create().unwrap();
        let dev = Device::new(mock.paths().clone());
        let b = mock.paths().rgb("brightness");
        fs::remove_file(&b).unwrap();
        fs::create_dir(&b).unwrap();
        let errors = full_profile().apply(&dev);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("brightness"));
        let l = dev.read_lighting();
        assert_eq!(l.speed, Some(3));
        assert_eq!(l.mode, Some(AnimationMode::Gradient));
        assert_eq!(dev.read_fans().thermal_profile, Some(ThermalProfile::Performance));
    }

    fn snapshot(dir: &std::path::Path) -> Vec<(String, String)> {
        let mut v: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                (e.file_name().to_string_lossy().into_owned(), fs::read_to_string(e.path()).unwrap())
            })
            .collect();
        v.sort();
        v
    }
}
