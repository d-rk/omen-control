#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::sysfs::mock::MockTree;

    fn setup() -> (MockTree, Device, Fans) {
        let mock = MockTree::create().unwrap();
        let dev = Device::new(mock.paths().clone());
        let page = Fans::load(&dev);
        (mock, dev, page)
    }

    fn file(mock: &MockTree, name: &str) -> String {
        fs::read_to_string(mock.paths().fan(name)).unwrap()
    }

    /// Scenario: Unavailable RPM shown as n/a
    #[test]
    fn unavailable_rpm_shown_as_na() {
        let (mock, dev, mut page) = setup();
        fs::write(mock.paths().fan("gpu_fan_rpm"), "n/a\n").unwrap();
        page.set_live(dev.read_live());
        assert_eq!(fmt_rpm(page.live.gpu_rpm), "n/a");
    }

    /// Scenario: Select thermal profile
    #[test]
    fn select_thermal_profile() {
        let (mock, dev, mut page) = setup();
        page.update(Message::Profile(ThermalProfile::Performance), &dev);
        assert_eq!(file(&mock, "thermal_profile"), "performance");
        assert_eq!(page.thermal_profile, Some(ThermalProfile::Performance));
    }

    /// Scenario: Toggle max fan
    #[test]
    fn toggle_max_fan() {
        let (mock, dev, mut page) = setup();
        page.update(Message::MaxFan(true), &dev);
        assert_eq!(file(&mock, "max_fan"), "1");
    }

    /// Scenario: Apply with GPU following CPU
    #[test]
    fn apply_with_gpu_following_cpu() {
        let (mock, dev, mut page) = setup();
        page.update(Message::GpuFollows(true), &dev);
        page.update(Message::CurveDrag { idx: 1, temp: 56.0, percent: 35.0 }, &dev);
        page.update(Message::ApplyCurves, &dev);
        assert_eq!(file(&mock, "fan_curve"), "40:0 56:35 70:55 85:80 95:100");
        assert_eq!(file(&mock, "gpu_fan_curve"), "cpu");
        assert!(!page.dirty);
    }

    /// Scenario: Revert discards edits
    #[test]
    fn revert_discards_edits() {
        let (_m, dev, mut page) = setup();
        let original = page.cpu_curve.clone();
        page.update(Message::CurveDrag { idx: 2, temp: 72.0, percent: 90.0 }, &dev);
        assert_ne!(page.cpu_curve, original);
        page.update(Message::RevertCurves, &dev);
        assert_eq!(page.cpu_curve, original);
    }

    /// Scenario: Invalid power limits not written
    #[test]
    fn invalid_power_limits_not_written() {
        let (mock, dev, mut page) = setup();
        page.update(Message::Pl1("90".into()), &dev);
        page.update(Message::Pl2("65".into()), &dev);
        page.update(Message::PowerApply, &dev);
        assert_eq!(file(&mock, "power_limits"), "default\n");
        assert!(page.power_error.as_deref().is_some_and(|e| e.contains("PL2 must be at least PL1")));
    }

    /// Scenario: Reset power limits
    #[test]
    fn reset_power_limits() {
        let (mock, dev, mut page) = setup();
        page.update(Message::Pl1("65".into()), &dev);
        page.update(Message::Pl2("90".into()), &dev);
        page.update(Message::PowerApply, &dev);
        assert_eq!(file(&mock, "power_limits"), "65 90");
        page.update(Message::PowerReset, &dev);
        assert_eq!(file(&mock, "power_limits"), "default");
    }

    /// Scenario: Power limits unsupported
    #[test]
    fn power_limits_unsupported() {
        let mock = MockTree::create().unwrap();
        fs::write(mock.paths().fan("power_limits"), "unsupported\n").unwrap();
        let page = Fans::load(&Device::new(mock.paths().clone()));
        assert!(!page.power_limits_enabled());
    }
}
