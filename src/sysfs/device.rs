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
