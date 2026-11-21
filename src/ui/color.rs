#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: HSV round trip
    #[test]
    fn hsv_round_trip() {
        for r in (0..=255).step_by(15) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(51) {
                    let c = Rgb::new(r as u8, g as u8, b as u8);
                    let back = Hsv::from_rgb(c).to_rgb();
                    let d = |a: u8, b: u8| (i16::from(a) - i16::from(b)).abs();
                    assert!(d(c.r, back.r) <= 1 && d(c.g, back.g) <= 1 && d(c.b, back.b) <= 1, "{c:?} -> {back:?}");
                }
            }
        }
    }
}
