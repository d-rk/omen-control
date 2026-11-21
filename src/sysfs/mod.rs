//! Userspace client for the omen-rgb-keyboard driver's sysfs interface.

mod device;
pub mod mock;
mod thermal;
pub mod values;

use std::path::{Path, PathBuf};

pub use device::{Device, FanState};
pub use values::*;

pub const DEFAULT_DRIVER_ROOT: &str = "/sys/devices/platform/omen-rgb-keyboard";
pub const DEFAULT_THERMAL_ROOT: &str = "/sys/class/thermal";
pub const DEFAULT_PLATFORM_PROFILE: &str = "/sys/firmware/acpi/platform_profile";
pub const ROOT_ENV: &str = "OMEN_SYSFS_ROOT";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: std::io::Error },
    #[error("{0}")]
    Invalid(#[from] ParseError),
}

/// Where the driver, the thermal zones and the platform profile live.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub driver: PathBuf,
    pub thermal: PathBuf,
    pub platform_profile: PathBuf,
}

impl Paths {
    /// System paths, with the driver root taken from `env_root` (the value of `OMEN_SYSFS_ROOT`) when set.
    pub fn from_env_value(env_root: Option<&str>) -> Paths {
        let driver = match env_root {
            Some(r) if !r.is_empty() => PathBuf::from(r),
            _ => PathBuf::from(DEFAULT_DRIVER_ROOT),
        };
        Paths {
            driver,
            thermal: PathBuf::from(DEFAULT_THERMAL_ROOT),
            platform_profile: PathBuf::from(DEFAULT_PLATFORM_PROFILE),
        }
    }

    pub fn from_env() -> Paths {
        Paths::from_env_value(std::env::var(ROOT_ENV).ok().as_deref())
    }

    pub fn rgb(&self, file: &str) -> PathBuf {
        self.driver.join("rgb_zones").join(file)
    }

    pub fn fan(&self, file: &str) -> PathBuf {
        self.driver.join("fan").join(file)
    }

    /// The driver counts as present when its root holds an `rgb_zones` or a `fan` directory.
    pub fn driver_present(&self) -> bool {
        is_dir(&self.driver.join("rgb_zones")) || is_dir(&self.driver.join("fan"))
    }
}

fn is_dir(p: &Path) -> bool {
    p.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Default root
    #[test]
    fn default_root() {
        assert_eq!(Paths::from_env_value(None).driver, PathBuf::from("/sys/devices/platform/omen-rgb-keyboard"));
    }

    /// Scenario: Root from environment
    #[test]
    fn root_from_environment() {
        assert_eq!(Paths::from_env_value(Some("/tmp/fake")).driver, PathBuf::from("/tmp/fake"));
    }

    /// Scenario: Root missing
    #[test]
    fn root_missing() {
        let p = Paths::from_env_value(Some("/nonexistent/omen-control-test"));
        assert!(!p.driver_present());
    }

    /// Scenario: Only fan directory
    #[test]
    fn only_fan_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("fan")).unwrap();
        let p = Paths::from_env_value(dir.path().to_str());
        assert!(p.driver_present());
    }
}
