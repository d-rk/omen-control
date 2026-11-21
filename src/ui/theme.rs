//! Dark "gaming" theme and shared widget styles.

use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Font, Shadow, Theme, Vector, color, font};

use crate::sysfs::Rgb;

pub const BG: Color = color!(0x0b0c10);
pub const SURFACE: Color = color!(0x14161c);
pub const SURFACE_HI: Color = color!(0x1c1f27);
pub const BORDER: Color = color!(0x2a2d37);
pub const TEXT: Color = color!(0xe9eaf0);
pub const MUTED: Color = color!(0x8b8f9c);
pub const ACCENT: Color = color!(0xe4002b);
pub const ACCENT_HI: Color = color!(0xff2d55);
pub const SUCCESS: Color = color!(0x2fd28a);
pub const WARNING: Color = color!(0xf5a623);
pub const DANGER: Color = color!(0xff5a5f);
pub const CPU: Color = color!(0xff3355);
pub const GPU: Color = color!(0x3aa8ff);

pub const BOLD: Font = Font { weight: font::Weight::Bold, ..Font::DEFAULT };
pub const SEMIBOLD: Font = Font { weight: font::Weight::Semibold, ..Font::DEFAULT };

pub fn theme() -> Theme {
    Theme::custom(
        "OMEN Dark",
        iced::theme::Palette {
            background: BG,
            text: TEXT,
            primary: ACCENT,
            success: SUCCESS,
            warning: WARNING,
            danger: DANGER,
        },
    )
}

pub fn to_color(c: Rgb) -> Color {
    Color::from_rgb8(c.r, c.g, c.b)
}

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

pub fn card(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        border: Border { color: BORDER, width: 1.0, radius: 14.0.into() },
        shadow: Shadow { color: with_alpha(Color::BLACK, 0.45), offset: Vector::new(0.0, 6.0), blur_radius: 18.0 },
        ..container::Style::default()
    }
}

pub fn header(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(color!(0x0f1116))),
        border: Border { color: BORDER, width: 1.0, radius: 0.0.into() },
        ..container::Style::default()
    }
}

pub fn error_box(_: &Theme) -> container::Style {
    container::Style {
        text_color: Some(DANGER),
        background: Some(Background::Color(with_alpha(DANGER, 0.08))),
        border: Border { color: with_alpha(DANGER, 0.5), width: 1.0, radius: 8.0.into() },
        ..container::Style::default()
    }
}

pub fn badge(c: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        text_color: Some(c),
        background: Some(Background::Color(with_alpha(c, 0.12))),
        border: Border { color: with_alpha(c, 0.5), width: 1.0, radius: 6.0.into() },
        ..container::Style::default()
    }
}

/// Tabs and segmented buttons: filled accent when `active`.
pub fn segment(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let disabled = matches!(status, button::Status::Disabled);
        let (bg, fg, border) = if active {
            (if hovered { ACCENT_HI } else { ACCENT }, Color::WHITE, ACCENT)
        } else if hovered {
            (SURFACE_HI, TEXT, with_alpha(ACCENT, 0.6))
        } else {
            (SURFACE, if disabled { with_alpha(MUTED, 0.5) } else { MUTED }, BORDER)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border { color: border, width: 1.0, radius: 8.0.into() },
            shadow: if active {
                Shadow { color: with_alpha(ACCENT, 0.45), offset: Vector::ZERO, blur_radius: 12.0 }
            } else {
                Shadow::default()
            },
            snap: true,
        }
    }
}

/// Secondary action button.
pub fn ghost(_: &Theme, status: button::Status) -> button::Style {
    let (bg, border, fg) = match status {
        button::Status::Hovered | button::Status::Pressed => (SURFACE_HI, with_alpha(ACCENT, 0.7), TEXT),
        button::Status::Disabled => (SURFACE, BORDER, with_alpha(MUTED, 0.5)),
        button::Status::Active => (SURFACE, BORDER, TEXT),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: fg,
        border: Border { color: border, width: 1.0, radius: 8.0.into() },
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Primary action button.
pub fn primary(_: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => ACCENT_HI,
        button::Status::Disabled => with_alpha(ACCENT, 0.3),
        button::Status::Active => ACCENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border { color: bg, width: 1.0, radius: 8.0.into() },
        shadow: Shadow { color: with_alpha(ACCENT, 0.35), offset: Vector::ZERO, blur_radius: 10.0 },
        snap: true,
    }
}

/// A colour swatch; `selected` draws a white ring.
pub fn swatch(c: Rgb, selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(to_color(c))),
            text_color: TEXT,
            border: Border {
                color: if selected || hovered { Color::WHITE } else { BORDER },
                width: if selected { 2.0 } else { 1.0 },
                radius: 6.0.into(),
            },
            shadow: Shadow { color: with_alpha(to_color(c), 0.5), offset: Vector::ZERO, blur_radius: 8.0 },
            snap: true,
        }
    }
}

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut s = text_input::default(theme, status);
    s.background = Background::Color(color!(0x0f1116));
    s.border = Border {
        color: match status {
            text_input::Status::Focused { .. } => ACCENT,
            _ => BORDER,
        },
        width: 1.0,
        radius: 8.0.into(),
    };
    s
}
