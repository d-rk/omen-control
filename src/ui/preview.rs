//! Approximate per-zone colours of the driver's animations, for the keyboard preview.

use std::f32::consts::TAU;

use super::color::{Hsv, hue_color};
use crate::sysfs::{AnimationMode, GradientGroup, Rgb, ZONE_COUNT};

pub struct PreviewInput<'a> {
    pub mode: AnimationMode,
    /// 1..=10
    pub speed: u8,
    /// 0..=100
    pub brightness: u8,
    pub zones: [Rgb; ZONE_COUNT],
    pub gradient: &'a [GradientGroup],
}

/// Zone colours `t` seconds into the animation.
pub fn colors(input: &PreviewInput<'_>, t: f32) -> [Rgb; ZONE_COUNT] {
    let rate = f32::from(input.speed.clamp(1, 10)) / 5.0;
    let tr = t * rate;
    let zones = input.zones;
    let out: [Rgb; ZONE_COUNT] = std::array::from_fn(|i| {
        let fi = i as f32;
        match input.mode {
            AnimationMode::Static => zones[i],
            AnimationMode::Breathing => zones[i].scaled(0.08 + 0.92 * (0.5 - 0.5 * (TAU * tr / 3.0).cos())),
            AnimationMode::Rainbow => hue_color(tr * 72.0 + fi * 90.0),
            AnimationMode::Wave => zones[i].scaled(0.15 + 0.85 * (0.5 + 0.5 * (TAU * (tr / 2.0 - fi / 4.0)).sin())),
            AnimationMode::Pulse => zones[i].scaled(0.1 + 0.9 * (TAU * tr / 1.5).sin().max(0.0).powi(3)),
            AnimationMode::Chase => {
                let lit = (tr * 2.5) as usize % ZONE_COUNT;
                if lit == i { zones[i] } else { zones[i].scaled(0.08) }
            }
            AnimationMode::Sparkle => {
                if noise((tr * 6.0) as u32, i as u32) > 0.72 {
                    Rgb::WHITE
                } else {
                    zones[i].scaled(0.45)
                }
            }
            AnimationMode::Candle => {
                let n = noise((tr * 8.0) as u32, i as u32);
                Rgb::new(255, 96, 12).lerp(Rgb::new(255, 150, 40), n).scaled(0.55 + 0.45 * n)
            }
            AnimationMode::Aurora => {
                let h = 150.0 + 70.0 * (TAU * (tr / 6.0 + fi * 0.18)).sin();
                Hsv::new(h, 0.85, 0.6 + 0.4 * (0.5 + 0.5 * (TAU * (tr / 4.0 - fi * 0.25)).sin())).to_rgb()
            }
            AnimationMode::Disco => {
                let step = (tr * 4.0) as u32;
                hue_color(noise(step, i as u32) * 360.0).scaled(if noise(step, 99 + i as u32) > 0.3 {
                    1.0
                } else {
                    0.1
                })
            }
            AnimationMode::Gradient => gradient_color(input.gradient, i, tr).unwrap_or(zones[i]),
            AnimationMode::Spectrum => hue_color(t * 360.0 * f32::from(input.speed.clamp(1, 10)) / 20.0),
        }
    });
    let b = f32::from(input.brightness.min(100)) / 100.0;
    out.map(|c| c.scaled(b))
}

fn gradient_color(groups: &[GradientGroup], zone: usize, tr: f32) -> Option<Rgb> {
    let g = groups.iter().find(|g| g.zones.iter().any(|&z| usize::from(z) == zone))?;
    match g.colors.len() {
        0 => None,
        1 => Some(g.colors[0]),
        n => {
            let pos = (tr / 2.0).rem_euclid(n as f32);
            let k = pos as usize % n;
            Some(g.colors[k].lerp(g.colors[(k + 1) % n], pos.fract()))
        }
    }
}

/// Deterministic pseudo-random value in 0..1.
fn noise(a: u32, b: u32) -> f32 {
    let mut x = a.wrapping_mul(0x9E37_79B9) ^ b.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

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
