#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: Size = Size::new(900.0, 320.0);

    /// Scenario: Click selects zone
    #[test]
    fn click_selects_zone() {
        let (kb, unit) = fit(SIZE);
        // Middle of the zone-2 band, on the Q-row.
        let p = Point::new(kb.x + 12.5 * unit, kb.y + 2.0 * unit);
        assert_eq!(zone_at(SIZE, p), Some(2));
        // Every key's centre maps to the zone the key is drawn in.
        let (_, _, ks) = keys(SIZE);
        assert!(ks.iter().all(|k| zone_at(SIZE, k.rect.center()) == Some(k.zone)));
    }

    /// Scenario: Click outside keyboard keeps selection
    #[test]
    fn click_outside_keyboard_keeps_selection() {
        assert_eq!(zone_at(SIZE, Point::new(2.0, 2.0)), None);
    }
}
