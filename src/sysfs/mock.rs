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
