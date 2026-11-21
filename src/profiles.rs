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
