//! A fake driver tree in a temporary directory, for `--mock` and tests.

use std::fs;
use std::io;
use std::path::Path;

use tempfile::TempDir;

use super::Paths;

/// Owns the temporary directory; the tree is deleted when this is dropped.
#[derive(Debug)]
pub struct MockTree {
    _dir: TempDir,
    paths: Paths,
}

const RGB_FILES: &[(&str, &str)] = &[
    ("zone00", "#e4002b\n"),
    ("zone01", "#ff6a00\n"),
    ("zone02", "#ffc400\n"),
    ("zone03", "#00a3ff\n"),
    ("all", "#e4002b\n"),
    ("brightness", "80\n"),
    ("brightness_lock", "0\n"),
    ("animation_mode", "static\n"),
    ("animation_speed", "5\n"),
    ("gradient_config", "0,1,2:FF0000,00FF00,0000FF;3:800080,FFA500\n"),
    ("mute_led", "Write '1' to turn on, '0' to turn off\n"),
    ("mute_state", "Write '1' for muted, '0' for unmuted (from userspace daemon)\n"),
];

const FAN_FILES: &[(&str, &str)] = &[
    ("cpu_fan_rpm", "2400\n"),
    ("gpu_fan_rpm", "2200\n"),
    ("max_rpm", "5200 5400\n"),
    ("max_fan", "0\n"),
    ("thermal_profile", "normal\n"),
    ("power_limits", "default\n"),
    ("fan_curve", "40:0 55:30 70:55 85:80 95:100\n"),
    ("gpu_fan_curve", "(cpu)\n"),
    ("fan_curve_enable", "0\n"),
    ("fan_temp_zone", "(auto)\n"),
    ("gpu_fan_temp_zone", "(cpu)\n"),
];

impl MockTree {
    pub fn create() -> io::Result<MockTree> {
        let dir = tempfile::Builder::new().prefix("omen-control-mock-").tempdir()?;
        let root = dir.path();
        let paths = Paths {
            driver: root.join("driver"),
            thermal: root.join("thermal"),
            platform_profile: root.join("firmware").join("platform_profile"),
        };
        write_files(&paths.driver.join("rgb_zones"), RGB_FILES)?;
        write_files(&paths.driver.join("fan"), FAN_FILES)?;
        write_files(&paths.thermal.join("thermal_zone0"), &[("type", "acpitz\n"), ("temp", "41000\n")])?;
        write_files(&paths.thermal.join("thermal_zone1"), &[("type", "x86_pkg_temp\n"), ("temp", "52000\n")])?;
        fs::create_dir_all(paths.platform_profile.parent().expect("has parent"))?;
        fs::write(&paths.platform_profile, "balanced\n")?;
        Ok(MockTree { _dir: dir, paths })
    }

    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    /// Moves the RPM and temperature readings a little and mimics the driver's read-back for the files where it
    /// differs from what was written. Called once per poll in `--mock` mode.
    pub fn tick(&self, seed: u64) {
        let jitter = |range: i64| {
            ((seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) >> 33) as i64 % (2 * range + 1))
                - range
        };
        let max_fan = read(&self.paths.fan("max_fan")).trim() == "1";
        let temp_path = self.paths.thermal.join("thermal_zone1").join("temp");
        let temp: i64 = read(&temp_path).trim().parse().unwrap_or(52000);
        let temp = (temp + jitter(1500)).clamp(42000, 78000);
        let _ = fs::write(&temp_path, format!("{temp}\n"));
        for (file, base) in [("cpu_fan_rpm", 0), ("gpu_fan_rpm", -150)] {
            let rpm = if max_fan { 5200 + base } else { (temp - 38000) / 12 + 1800 + base + jitter(60) };
            let _ = fs::write(self.paths.fan(file), format!("{}\n", rpm.max(0)));
        }
        self.emulate_driver();
    }

    fn emulate_driver(&self) {
        let p = &self.paths;
        // `all` sets every zone.
        if let Ok(c) = read(&p.rgb("all")).trim().parse::<super::Rgb>() {
            let lower = format!("#{}\n", c.to_string().to_ascii_lowercase());
            if read(&p.rgb("all")) != lower {
                for i in 0..4 {
                    let _ = fs::write(p.rgb(&format!("zone{i:02}")), &lower);
                }
                let _ = fs::write(p.rgb("all"), &lower);
            }
        }
        let gpu = read(&p.fan("gpu_fan_curve"));
        if matches!(gpu.trim(), "cpu" | "default") {
            let _ = fs::write(p.fan("gpu_fan_curve"), "(cpu)\n");
        }
        let cpu = read(&p.fan("fan_curve"));
        if matches!(cpu.trim(), "default" | "ec") {
            let _ = fs::write(p.fan("fan_curve_enable"), "0\n");
        }
    }
}

fn read(p: &Path) -> String {
    fs::read_to_string(p).unwrap_or_default()
}

fn write_files(dir: &Path, files: &[(&str, &str)]) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    for (name, content) in files {
        fs::write(dir.join(name), content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::Device;
    use super::*;

    /// Scenario: Mock tree is readable as a driver
    #[test]
    fn mock_tree_is_readable_as_a_driver() {
        let mock = MockTree::create().unwrap();
        let dev = Device::new(mock.paths().clone());
        assert!(dev.driver_present());
        let l = dev.read_lighting();
        assert!(l.zones.iter().all(Option::is_some));
        assert!(l.has_all);
        assert!(l.brightness.is_some() && l.brightness_lock.is_some() && l.mode.is_some() && l.speed.is_some());
        assert!(l.gradient.is_some());
        let f = dev.read_fans();
        assert!(f.cpu_rpm.is_some() && f.gpu_rpm.is_some() && f.max_rpm.is_some() && f.max_fan.is_some());
        assert!(f.thermal_profile.is_some() && f.power_limits.is_some() && f.curve_enable.is_some());
        assert!(f.cpu_curve.is_some() && f.gpu_curve.is_some());
        assert!(f.cpu_temp.is_some() && f.gpu_temp.is_some() && f.platform_profile.is_some());
    }
}
