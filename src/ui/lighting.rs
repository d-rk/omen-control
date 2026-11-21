#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::sysfs::mock::MockTree;

    fn setup() -> (MockTree, Device, Lighting) {
        let mock = MockTree::create().unwrap();
        let dev = Device::new(mock.paths().clone());
        let page = Lighting::load(&dev);
        (mock, dev, page)
    }

    fn file(mock: &MockTree, name: &str) -> String {
        fs::read_to_string(mock.paths().rgb(name)).unwrap()
    }

    /// Scenario: Click selects zone
    #[test]
    fn click_selects_zone() {
        let (_m, dev, mut page) = setup();
        page.update(Message::ZoneClicked(2), &dev);
        assert_eq!(page.selected, 2);
        assert_eq!(page.hex, page.zones[2].to_string());
    }

    /// Scenario: Pick colour writes selected zone
    #[test]
    fn pick_colour_writes_selected_zone() {
        let (mock, dev, mut page) = setup();
        page.update(Message::ZoneClicked(1), &dev);
        page.update(Message::PickerChanged(Hsv::new(120.0, 1.0, 1.0)), &dev);
        page.update(Message::PickerReleased, &dev);
        assert_eq!(file(&mock, "zone01"), "00FF00");
        assert_eq!(page.zones[1], Rgb::new(0, 255, 0));
    }

    /// Scenario: Apply to all
    #[test]
    fn apply_to_all() {
        let (mock, dev, mut page) = setup();
        page.update(Message::Swatch(Rgb::new(255, 0, 0)), &dev);
        page.update(Message::ApplyAll, &dev);
        assert_eq!(file(&mock, "all"), "FF0000");
        assert_eq!(page.zones, [Rgb::new(255, 0, 0); 4]);
    }

    /// Scenario: Invalid hex input
    #[test]
    fn invalid_hex_input() {
        let (mock, dev, mut page) = setup();
        let before = file(&mock, "zone00");
        page.update(Message::HexInput("12345G".into()), &dev);
        page.update(Message::HexSubmit, &dev);
        assert!(page.hex_error.is_some());
        assert_eq!(file(&mock, "zone00"), before);
    }

    /// Scenario: Brightness written
    #[test]
    fn brightness_written() {
        let (mock, dev, mut page) = setup();
        page.update(Message::Brightness(40), &dev);
        page.update(Message::BrightnessReleased, &dev);
        assert_eq!(file(&mock, "brightness"), "40");
    }

    /// Scenario: Gradient editor only in gradient mode
    #[test]
    fn gradient_editor_only_in_gradient_mode() {
        let (_m, dev, mut page) = setup();
        page.update(Message::Mode(AnimationMode::Gradient), &dev);
        assert!(page.shows_gradient_editor());
        page.update(Message::Mode(AnimationMode::Wave), &dev);
        assert!(!page.shows_gradient_editor());
    }

    /// Scenario: Incomplete gradient group not written
    #[test]
    fn incomplete_gradient_group_not_written() {
        let (mock, dev, mut page) = setup();
        let before = file(&mock, "gradient_config");
        page.update(Message::ToggleGroupZone(1, 3), &dev); // group 2 had only zone 3
        page.update(Message::ApplyGradient, &dev);
        assert_eq!(file(&mock, "gradient_config"), before);
        assert!(page.gradient_error.as_deref().is_some_and(|e| e.contains("Group 2")));
    }

    /// Scenario: Write error shown
    #[test]
    fn write_error_shown() {
        let (mock, dev, mut page) = setup();
        let b = mock.paths().rgb("brightness");
        fs::remove_file(&b).unwrap();
        fs::create_dir(&b).unwrap();
        page.update(Message::Brightness(10), &dev);
        page.update(Message::BrightnessReleased, &dev);
        assert!(page.error.as_deref().is_some_and(|e| e.contains("brightness")));
        assert_eq!(page.brightness, 80);
    }
}
