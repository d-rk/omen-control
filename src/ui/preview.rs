#[cfg(test)]
mod tests {
    use super::*;

    fn input(mode: AnimationMode) -> PreviewInput<'static> {
        PreviewInput {
            mode,
            speed: 5,
            brightness: 100,
            zones: [Rgb::new(255, 0, 0), Rgb::new(0, 255, 0), Rgb::new(0, 0, 255), Rgb::new(255, 255, 0)],
            gradient: &[],
        }
    }

    /// Scenario: Static preview shows zone colours
    #[test]
    fn static_preview_shows_zone_colours() {
        let i = input(AnimationMode::Static);
        for t in [0.0, 1.3, 7.9] {
            assert_eq!(colors(&i, t), i.zones);
        }
    }

    /// Scenario: Breathing preview dims over time
    #[test]
    fn breathing_preview_dims_over_time() {
        let i = input(AnimationMode::Breathing);
        let red = |t: f32| colors(&i, t)[0].r;
        let samples: Vec<u8> = (0..60).map(|k| red(k as f32 * 0.05)).collect();
        let max = *samples.iter().max().unwrap();
        let min = *samples.iter().min().unwrap();
        assert!(max >= 250 && min < max / 2, "min {min} max {max}");
    }
}
