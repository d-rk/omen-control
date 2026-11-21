mod device;
pub mod mock;
mod thermal;
pub mod values;

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
