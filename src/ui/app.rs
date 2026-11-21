#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::sysfs::Paths;

    /// Scenario: Check again finds driver
    #[test]
    fn check_again_finds_driver() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::from_env_value(dir.path().join("driver").to_str());
        let mut app = App::new(Config { device: Device::new(paths.clone()), mock: None, store: None });
        assert!(!app.driver_present);
        fs::create_dir_all(paths.driver.join("rgb_zones")).unwrap();
        app.update(Message::CheckAgain);
        assert!(app.driver_present);
    }

    /// Scenario: Choose profile from header
    #[test]
    fn choose_profile_from_header() {
        let mock = MockTree::create().unwrap();
        let dev = Device::new(mock.paths().clone());
        let dir = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(dir.path().to_owned());
        let mut gaming = Profile::capture(&dev, false);
        gaming.lighting.brightness = Some(30);
        store.save("gaming", &gaming).unwrap();

        let mut app = App::new(Config { device: dev.clone(), mock: None, store: Some(store) });
        assert_eq!(app.profiles, ["gaming"]);
        app.update(Message::ProfileChosen("gaming".into()));
        assert_eq!(dev.read_lighting().brightness, Some(30));
        assert_eq!(app.lighting.brightness, 30);
        assert_eq!(app.current_profile.as_deref(), Some("gaming"));
    }
}
