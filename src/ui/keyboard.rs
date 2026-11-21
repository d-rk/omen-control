//! Keyboard drawing for the lighting page: a laptop layout split into the driver's four zones.

use iced::widget::canvas::{self, Frame, Path, Stroke, Text};
use iced::{Color, Point, Rectangle, Size, alignment};

use super::theme::{self, with_alpha};
use crate::sysfs::{Rgb, ZONE_COUNT};

/// Layout width in key units (main block 15u + numpad 4u).
const UNITS_W: f32 = 19.0;
/// Layout height in key units (function row is 0.6u).
const UNITS_H: f32 = 5.6;
const ROW_H: [f32; 6] = [0.6, 1.0, 1.0, 1.0, 1.0, 1.0];
/// Zone boundaries in key units, left to right. Chosen so that no key centre sits on an edge.
const ZONE_EDGES: [f32; 3] = [5.3, 10.1, 15.0];
const PADDING: f32 = 18.0;

/// (label, width in units)
type Row = &'static [(&'static str, f32)];

const ROWS: [Row; 6] = [
    &[
        ("esc", 1.0),
        ("F1", 1.0),
        ("F2", 1.0),
        ("F3", 1.0),
        ("F4", 1.0),
        ("F5", 1.0),
        ("F6", 1.0),
        ("F7", 1.0),
        ("F8", 1.0),
        ("F9", 1.0),
        ("F10", 1.0),
        ("F11", 1.0),
        ("F12", 1.0),
        ("ins", 1.0),
        ("prt", 1.0),
        ("del", 1.0),
        ("home", 1.0),
        ("end", 1.0),
        ("pgup", 1.0),
    ],
    &[
        ("`", 1.0),
        ("1", 1.0),
        ("2", 1.0),
        ("3", 1.0),
        ("4", 1.0),
        ("5", 1.0),
        ("6", 1.0),
        ("7", 1.0),
        ("8", 1.0),
        ("9", 1.0),
        ("0", 1.0),
        ("-", 1.0),
        ("=", 1.0),
        ("⌫", 2.0),
        ("num", 1.0),
        ("/", 1.0),
        ("*", 1.0),
        ("-", 1.0),
    ],
    &[
        ("tab", 1.5),
        ("Q", 1.0),
        ("W", 1.0),
        ("E", 1.0),
        ("R", 1.0),
        ("T", 1.0),
        ("Y", 1.0),
        ("U", 1.0),
        ("I", 1.0),
        ("O", 1.0),
        ("P", 1.0),
        ("[", 1.0),
        ("]", 1.0),
        ("\\", 1.5),
        ("7", 1.0),
        ("8", 1.0),
        ("9", 1.0),
        ("+", 1.0),
    ],
    &[
        ("caps", 1.75),
        ("A", 1.0),
        ("S", 1.0),
        ("D", 1.0),
        ("F", 1.0),
        ("G", 1.0),
        ("H", 1.0),
        ("J", 1.0),
        ("K", 1.0),
        ("L", 1.0),
        (";", 1.0),
        ("'", 1.0),
        ("enter", 2.25),
        ("4", 1.0),
        ("5", 1.0),
        ("6", 1.0),
        ("", 1.0),
    ],
    &[
        ("shift", 2.25),
        ("Z", 1.0),
        ("X", 1.0),
        ("C", 1.0),
        ("V", 1.0),
        ("B", 1.0),
        ("N", 1.0),
        ("M", 1.0),
        (",", 1.0),
        (".", 1.0),
        ("/", 1.0),
        ("shift", 2.75),
        ("1", 1.0),
        ("2", 1.0),
        ("3", 1.0),
        ("ent", 1.0),
    ],
    &[
        ("ctrl", 1.25),
        ("fn", 1.0),
        ("⊞", 1.25),
        ("alt", 1.25),
        ("", 5.0),
        ("alt", 1.0),
        ("ctrl", 1.25),
        ("◀", 1.0),
        ("▲▼", 1.0),
        ("▶", 1.0),
        ("0", 2.0),
        (".", 1.0),
        ("", 1.0),
    ],
];

/// Where the keyboard sits in a canvas of `size`, and its key unit in pixels.
fn fit(size: Size) -> (Rectangle, f32) {
    let unit = ((size.width - 2.0 * PADDING) / UNITS_W).min((size.height - 2.0 * PADDING) / UNITS_H).max(1.0);
    let w = unit * UNITS_W;
    let h = unit * UNITS_H;
    (Rectangle { x: (size.width - w) / 2.0, y: (size.height - h) / 2.0, width: w, height: h }, unit)
}

fn zone_of(x_units: f32) -> usize {
    ZONE_EDGES.iter().take_while(|&&e| x_units >= e).count()
}

/// The zone under a canvas-local point, or `None` outside the keyboard outline.
pub fn zone_at(size: Size, p: Point) -> Option<usize> {
    let (kb, unit) = fit(size);
    kb.contains(p).then(|| zone_of((p.x - kb.x) / unit))
}

struct Key {
    rect: Rectangle,
    label: &'static str,
    zone: usize,
}

fn keys(size: Size) -> (Rectangle, f32, Vec<Key>) {
    let (kb, unit) = fit(size);
    let gap = (unit * 0.1).max(1.5);
    let mut out = Vec::new();
    let mut y = kb.y;
    for (row, h) in ROWS.iter().zip(ROW_H) {
        let mut x = 0.0;
        for &(label, w) in row.iter() {
            out.push(Key {
                rect: Rectangle {
                    x: kb.x + x * unit + gap / 2.0,
                    y: y + gap / 2.0,
                    width: w * unit - gap,
                    height: h * unit - gap,
                },
                label,
                zone: zone_of(x + w / 2.0),
            });
            x += w;
        }
        y += h * unit;
    }
    (kb, unit, out)
}

/// Draws the keyboard with each key lit in its zone's colour.
pub fn draw(frame: &mut Frame, colors: &[Rgb; ZONE_COUNT], selected: Option<usize>, hovered: Option<usize>) {
    let size = frame.size();
    let (kb, unit, keys) = keys(size);
    let radius = (unit * 0.14).max(2.0);

    // Deck.
    let deck = Rectangle { x: kb.x - 12.0, y: kb.y - 12.0, width: kb.width + 24.0, height: kb.height + 24.0 };
    frame.fill(
        &Path::rounded_rectangle(deck.position(), deck.size(), 18.0.into()),
        canvas::gradient::Linear::new(deck.position(), Point::new(deck.x, deck.y + deck.height))
            .add_stop(0.0, iced::color!(0x1a1c23))
            .add_stop(1.0, iced::color!(0x0d0e12)),
    );
    frame.stroke(
        &Path::rounded_rectangle(deck.position(), deck.size(), 18.0.into()),
        Stroke::default().with_color(theme::BORDER).with_width(1.0),
    );

    // Underglow first so caps draw on top of it.
    for k in &keys {
        let c = theme::to_color(colors[k.zone]);
        for (grow, alpha) in [(5.0, 0.07), (3.0, 0.12), (1.5, 0.22)] {
            let r = expand(k.rect, grow);
            frame.fill(&Path::rounded_rectangle(r.position(), r.size(), (radius + grow).into()), with_alpha(c, alpha));
        }
    }

    for k in &keys {
        let lit = colors[k.zone];
        let c = theme::to_color(lit);
        let cap = mix(iced::color!(0x15161b), c, 0.22);
        let path = Path::rounded_rectangle(k.rect.position(), k.rect.size(), radius.into());
        frame.fill(&path, cap);
        frame.stroke(&path, Stroke::default().with_color(with_alpha(c, 0.85)).with_width(1.1));
        if !k.label.is_empty() {
            let legend =
                if luminance(lit) < 0.04 { with_alpha(theme::MUTED, 0.35) } else { mix(c, Color::WHITE, 0.25) };
            frame.fill_text(Text {
                content: k.label.to_owned(),
                position: k.rect.center(),
                color: legend,
                size: (unit * if k.label.chars().count() > 2 { 0.24 } else { 0.32 }).into(),
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });
        }
    }

    // Zone outlines.
    for zone in 0..ZONE_COUNT {
        let is_sel = selected == Some(zone);
        let is_hover = hovered == Some(zone);
        if !is_sel && !is_hover {
            continue;
        }
        // Zones are ragged (rows are staggered), so outline each of the zone's keys rather than a bounding box.
        let zone_keys: Vec<&Key> = keys.iter().filter(|k| k.zone == zone).collect();
        let Some(b) = zone_keys.iter().map(|k| k.rect).reduce(|a, b| a.union(&b)) else { continue };
        for k in &zone_keys {
            let r = expand(k.rect, 1.5);
            let path = Path::rounded_rectangle(r.position(), r.size(), (radius + 1.5).into());
            if is_sel {
                frame.stroke(&path, Stroke::default().with_color(with_alpha(Color::WHITE, 0.25)).with_width(4.0));
                frame.stroke(&path, Stroke::default().with_color(Color::WHITE).with_width(1.6));
            } else {
                frame.stroke(&path, Stroke::default().with_color(with_alpha(Color::WHITE, 0.45)).with_width(1.2));
            }
        }
        frame.fill_text(Text {
            content: format!("ZONE {}", zone + 1),
            position: Point::new(b.center_x(), deck.y - 2.0),
            color: if is_sel { Color::WHITE } else { theme::MUTED },
            size: 11.0.into(),
            font: theme::BOLD,
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Bottom,
            ..Text::default()
        });
    }
}

fn expand(r: Rectangle, by: f32) -> Rectangle {
    Rectangle { x: r.x - by, y: r.y - by, width: r.width + 2.0 * by, height: r.height + 2.0 * by }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgb(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t)
}

fn luminance(c: Rgb) -> f32 {
    (0.2126 * f32::from(c.r) + 0.7152 * f32::from(c.g) + 0.0722 * f32::from(c.b)) / 255.0
}

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
