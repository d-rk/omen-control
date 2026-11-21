//! HSV colour model for the colour picker.

use crate::sysfs::Rgb;

/// Hue in degrees 0..360, saturation and value in 0..=1.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Hsv {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h: h.rem_euclid(360.0), s: s.clamp(0.0, 1.0), v: v.clamp(0.0, 1.0) }
    }

    pub fn from_rgb(c: Rgb) -> Hsv {
        let (r, g, b) = (f32::from(c.r) / 255.0, f32::from(c.g) / 255.0, f32::from(c.b) / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let h = if d == 0.0 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / d).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        let s = if max == 0.0 { 0.0 } else { d / max };
        Hsv::new(h, s, max)
    }

    pub fn to_rgb(self) -> Rgb {
        let c = self.v * self.s;
        let hp = self.h.rem_euclid(360.0) / 60.0;
        let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
        let (r, g, b) = match hp as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = self.v - c;
        let ch = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        Rgb::new(ch(r), ch(g), ch(b))
    }

    /// Keeps hue (and saturation) when the new colour is grey or black, so dragging to a corner and back does not
    /// lose the hue the user picked.
    pub fn follow(self, c: Rgb) -> Hsv {
        let mut n = Hsv::from_rgb(c);
        if n.s == 0.0 || n.v == 0.0 {
            n.h = self.h;
        }
        if n.v == 0.0 {
            n.s = self.s;
        }
        n
    }
}

pub fn hue_color(h: f32) -> Rgb {
    Hsv::new(h, 1.0, 1.0).to_rgb()
}

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
