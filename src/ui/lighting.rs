//! Lighting page: zone colours, brightness, animation and gradient.

use std::time::Instant;

use iced::mouse::{self, Cursor};
use iced::widget::canvas::{self, Action, Event, Frame, Geometry};
use iced::widget::{
    Column, button, canvas as canvas_widget, column, container, pick_list, row, slider, space, text, text_input,
    toggler,
};
use iced::{Center, Element, Fill, Rectangle, Renderer, Theme};

use super::color::Hsv;
use super::color_picker::ColorPicker;
use super::keyboard::Layout;
use super::preview::{self, PreviewInput};
use super::theme;
use super::{field_label, inline_error, section_title};
use crate::sysfs::{AnimationMode, Device, GradientGroup, Rgb, ZONE_COUNT, validate_gradient_group};

pub const SWATCHES: [Rgb; 12] = [
    Rgb::new(0xE4, 0x00, 0x2B),
    Rgb::new(0xFF, 0x45, 0x00),
    Rgb::new(0xFF, 0xA5, 0x00),
    Rgb::new(0xFF, 0xE0, 0x00),
    Rgb::new(0x7C, 0xFF, 0x00),
    Rgb::new(0x00, 0xFF, 0x6A),
    Rgb::new(0x00, 0xE5, 0xFF),
    Rgb::new(0x00, 0x6B, 0xFF),
    Rgb::new(0x6A, 0x00, 0xFF),
    Rgb::new(0xD1, 0x00, 0xFF),
    Rgb::new(0xFF, 0x00, 0x9D),
    Rgb::WHITE,
];

#[derive(Debug, Clone)]
pub enum Message {
    ZoneClicked(usize),
    ZoneHovered(Option<usize>),
    PickerChanged(Hsv),
    PickerReleased,
    HexInput(String),
    HexSubmit,
    Swatch(Rgb),
    ApplyAll,
    Brightness(u8),
    BrightnessReleased,
    BrightnessLock(bool),
    Speed(u8),
    SpeedReleased,
    Mode(AnimationMode),
    AddGroup,
    RemoveGroup(usize),
    ToggleGroupZone(usize, u8),
    AddGroupColor(usize),
    RemoveGroupColor(usize, usize),
    ApplyGradient,
    Frame(Instant),
}

/// Which files exist; missing ones hide or disable their control.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Available {
    pub zones: [bool; ZONE_COUNT],
    pub all: bool,
    pub brightness: bool,
    pub lock: bool,
    pub mode: bool,
    pub speed: bool,
    pub gradient: bool,
}

#[derive(Debug, Clone)]
pub struct Lighting {
    pub available: Available,
    pub layout: Layout,
    pub zones: [Rgb; ZONE_COUNT],
    pub selected: usize,
    pub hovered: Option<usize>,
    pub hsv: Hsv,
    pub hex: String,
    pub hex_error: Option<String>,
    pub brightness: u8,
    /// Last brightness read from or written to the driver.
    pub brightness_driver: u8,
    pub lock: bool,
    pub mode: AnimationMode,
    pub speed: u8,
    pub speed_driver: u8,
    pub gradient: Vec<GradientGroup>,
    pub gradient_error: Option<String>,
    pub error: Option<String>,
    started: Instant,
    pub t: f32,
}

impl Lighting {
    pub fn load(dev: &Device) -> Lighting {
        let s = dev.read_lighting();
        let zones = s.zones.map(|z| z.unwrap_or(Rgb::BLACK));
        let brightness = s.brightness.unwrap_or(100);
        let speed = s.speed.unwrap_or(5);
        let selected = 0;
        Lighting {
            available: Available {
                zones: s.zones.map(|z| z.is_some()),
                all: s.has_all,
                brightness: s.brightness.is_some(),
                lock: s.brightness_lock.is_some(),
                mode: s.mode.is_some(),
                speed: s.speed.is_some(),
                gradient: s.gradient.is_some(),
            },
            layout: Layout::detect(),
            zones,
            selected,
            hovered: None,
            hsv: Hsv::from_rgb(zones[selected]),
            hex: zones[selected].to_string(),
            hex_error: None,
            brightness,
            brightness_driver: brightness,
            lock: s.brightness_lock.unwrap_or(false),
            mode: s.mode.unwrap_or_default(),
            speed,
            speed_driver: speed,
            gradient: s.gradient.unwrap_or_default(),
            gradient_error: None,
            error: None,
            started: Instant::now(),
            t: 0.0,
        }
    }

    pub fn shows_gradient_editor(&self) -> bool {
        self.mode == AnimationMode::Gradient && self.available.gradient
    }

    /// Whether the preview needs animation frames.
    pub fn animating(&self) -> bool {
        self.mode != AnimationMode::Static
    }

    pub fn preview_colors(&self) -> [Rgb; ZONE_COUNT] {
        preview::colors(
            &PreviewInput {
                mode: self.mode,
                speed: self.speed,
                brightness: self.brightness,
                zones: self.zones,
                gradient: &self.gradient,
            },
            self.t,
        )
    }

    fn report(&mut self, r: Result<(), crate::sysfs::Error>) -> bool {
        match r {
            Ok(()) => {
                self.error = None;
                true
            }
            Err(e) => {
                self.error = Some(e.to_string());
                false
            }
        }
    }

    fn select_color(&mut self, c: Rgb) {
        self.hsv = self.hsv.follow(c);
        self.hex = c.to_string();
        self.hex_error = None;
    }

    fn commit_color(&mut self, dev: &Device, c: Rgb) {
        self.select_color(c);
        self.zones[self.selected] = c;
        if self.available.zones[self.selected] {
            let r = dev.write_zone(self.selected, c);
            self.report(r);
        }
    }

    pub fn update(&mut self, msg: Message, dev: &Device) {
        match msg {
            Message::ZoneClicked(z) if z < ZONE_COUNT => {
                self.selected = z;
                self.select_color(self.zones[z]);
                self.hsv = Hsv::from_rgb(self.zones[z]);
            }
            Message::ZoneClicked(_) => {}
            Message::ZoneHovered(z) => self.hovered = z,
            Message::PickerChanged(hsv) => {
                self.hsv = hsv;
                let c = hsv.to_rgb();
                self.hex = c.to_string();
                self.hex_error = None;
                self.zones[self.selected] = c;
            }
            Message::PickerReleased => {
                let hsv = self.hsv;
                self.commit_color(dev, hsv.to_rgb());
                self.hsv = hsv;
            }
            Message::HexInput(s) => {
                self.hex = s;
                self.hex_error = None;
            }
            Message::HexSubmit => match self.hex.parse::<Rgb>() {
                Ok(c) => self.commit_color(dev, c),
                Err(_) => self.hex_error = Some(format!("\"{}\" is not a colour; use six hex digits", self.hex)),
            },
            Message::Swatch(c) => self.commit_color(dev, c),
            Message::ApplyAll => {
                let c = self.zones[self.selected];
                if self.report(dev.write_all_zones(c)) {
                    self.zones = [c; ZONE_COUNT];
                }
            }
            Message::Brightness(b) => self.brightness = b.min(100),
            Message::BrightnessReleased => {
                if self.report(dev.write_brightness(self.brightness)) {
                    self.brightness_driver = self.brightness;
                } else {
                    self.brightness = self.brightness_driver;
                }
            }
            Message::BrightnessLock(on) => {
                if self.report(dev.write_brightness_lock(on)) {
                    self.lock = on;
                }
            }
            Message::Speed(s) => self.speed = s.clamp(1, 10),
            Message::SpeedReleased => {
                if self.report(dev.write_speed(self.speed)) {
                    self.speed_driver = self.speed;
                } else {
                    self.speed = self.speed_driver;
                }
            }
            Message::Mode(m) => {
                if self.report(dev.write_mode(m)) {
                    self.mode = m;
                    self.started = Instant::now();
                    self.t = 0.0;
                }
            }
            Message::AddGroup => {
                let used: Vec<u8> = self.gradient.iter().flat_map(|g| g.zones.clone()).collect();
                let zones = (0..ZONE_COUNT as u8).filter(|z| !used.contains(z)).take(1).collect();
                self.gradient.push(GradientGroup { zones, colors: vec![self.zones[self.selected]] });
            }
            Message::RemoveGroup(g) => {
                if g < self.gradient.len() {
                    self.gradient.remove(g);
                }
            }
            Message::ToggleGroupZone(g, z) => {
                if let Some(grp) = self.gradient.get_mut(g) {
                    if let Some(pos) = grp.zones.iter().position(|&x| x == z) {
                        grp.zones.remove(pos);
                    } else {
                        grp.zones.push(z);
                        grp.zones.sort_unstable();
                    }
                }
            }
            Message::AddGroupColor(g) => {
                let c = self.hsv.to_rgb();
                if let Some(grp) = self.gradient.get_mut(g) {
                    grp.colors.push(c);
                }
            }
            Message::RemoveGroupColor(g, c) => {
                if let Some(grp) = self.gradient.get_mut(g)
                    && c < grp.colors.len()
                {
                    grp.colors.remove(c);
                }
            }
            Message::ApplyGradient => {
                self.gradient_error =
                    self.gradient.iter().enumerate().find_map(|(i, g)| {
                        validate_gradient_group(g).err().map(|why| format!("Group {}: {why}", i + 1))
                    });
                if self.gradient_error.is_none() {
                    let r = dev.write_gradient(&self.gradient);
                    self.report(r);
                }
            }
            Message::Frame(now) => self.t = now.duration_since(self.started).as_secs_f32(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let keyboard = container(
            canvas_widget(KeyboardCanvas {
                layout: self.layout,
                colors: self.preview_colors(),
                selected: self.selected,
                hovered: self.hovered,
            })
            .width(Fill)
            .height(290),
        )
        .padding(16)
        .style(theme::card);

        let left = column![keyboard, self.effects_card()].spacing(18).width(Fill);
        let right = column![self.color_card()].spacing(18).width(360);

        let mut page = column![row![left, right].spacing(18)].spacing(18);
        if self.shows_gradient_editor() {
            page = page.push(self.gradient_card());
        }
        if let Some(e) = &self.error {
            page = page.push(inline_error(e));
        }
        page.into()
    }

    fn color_card(&self) -> Element<'_, Message> {
        let picker = canvas_widget(ColorPicker {
            hsv: self.hsv,
            on_change: Message::PickerChanged,
            on_release: Message::PickerReleased,
        })
        .width(Fill)
        .height(220);

        let current = self.hsv.to_rgb();
        let hex_row = row![
            container(space().width(36).height(36)).style(move |_: &Theme| container::Style {
                background: Some(theme::to_color(current).into()),
                border: iced::Border { color: theme::BORDER, width: 1.0, radius: 8.0.into() },
                ..container::Style::default()
            }),
            text_input("RRGGBB", &self.hex)
                .on_input(Message::HexInput)
                .on_submit(Message::HexSubmit)
                .padding(8)
                .style(theme::input)
                .font(iced::Font::MONOSPACE),
            button(text("Set").font(theme::SEMIBOLD)).on_press(Message::HexSubmit).padding([8, 14]).style(theme::ghost),
        ]
        .spacing(10)
        .align_y(Center);

        let swatches = SWATCHES.chunks(6).fold(Column::new().spacing(8), |col, chunk| {
            col.push(chunk.iter().fold(row![].spacing(8), |r, &c| {
                r.push(
                    button(space().width(Fill).height(26))
                        .width(Fill)
                        .on_press(Message::Swatch(c))
                        .style(theme::swatch(c, c == current)),
                )
            }))
        });

        let mut col = column![section_title(format!("Zone {} colour", self.selected + 1)), picker, hex_row, swatches,]
            .spacing(14);
        if let Some(e) = &self.hex_error {
            col = col.push(inline_error(e));
        }
        if !self.available.zones[self.selected] {
            col = col.push(text("This zone's file is missing; changes are preview only.").size(12).color(theme::MUTED));
        }
        let all = button(text("Apply to all zones").font(theme::SEMIBOLD).width(Fill).align_x(Center))
            .width(Fill)
            .padding(10)
            .style(theme::primary)
            .on_press_maybe(self.available.all.then_some(Message::ApplyAll));
        col = col.push(all);
        container(col).padding(18).style(theme::card).into()
    }

    fn effects_card(&self) -> Element<'_, Message> {
        let a = &self.available;
        let mut col = column![section_title("Effects")].spacing(14);

        if a.mode {
            col = col.push(
                row![
                    field_label("Animation"),
                    pick_list(&AnimationMode::ALL[..], Some(self.mode), Message::Mode).padding([8, 12]).width(220),
                    space::horizontal(),
                    text(if self.animating() { "Live preview ↑" } else { "" }).size(12).color(theme::MUTED),
                ]
                .spacing(12)
                .align_y(Center),
            );
        }
        if a.brightness {
            col = col.push(
                row![
                    field_label("Brightness"),
                    slider(0..=100u8, self.brightness, Message::Brightness)
                        .on_release(Message::BrightnessReleased)
                        .width(Fill),
                    text(format!("{:>3}%", self.brightness)).font(iced::Font::MONOSPACE).width(48),
                ]
                .spacing(12)
                .align_y(Center),
            );
        }
        if a.speed {
            let speed_enabled = self.animating();
            let s = slider(1..=10u8, self.speed, Message::Speed).on_release(Message::SpeedReleased).width(Fill);
            col = col.push(
                row![
                    field_label("Speed"),
                    s,
                    text(format!("{:>3}", self.speed)).font(iced::Font::MONOSPACE).width(48).color(if speed_enabled {
                        theme::TEXT
                    } else {
                        theme::MUTED
                    }),
                ]
                .spacing(12)
                .align_y(Center),
            );
        }
        if a.lock {
            col = col.push(
                row![
                    field_label("Lock brightness"),
                    toggler(self.lock).on_toggle(Message::BrightnessLock),
                    text("Stops the desktop from dimming the keyboard when idle.").size(12).color(theme::MUTED),
                ]
                .spacing(12)
                .align_y(Center),
            );
        }
        container(col).padding(18).width(Fill).style(theme::card).into()
    }

    fn gradient_card(&self) -> Element<'_, Message> {
        let mut col = column![
            row![
                section_title("Gradient"),
                space::horizontal(),
                button(text("+ Group")).on_press(Message::AddGroup).padding([6, 12]).style(theme::ghost),
                button(text("Apply gradient").font(theme::SEMIBOLD))
                    .on_press(Message::ApplyGradient)
                    .padding([6, 14])
                    .style(theme::primary),
            ]
            .spacing(10)
            .align_y(Center),
            text("Each group cycles its zones through its colours. \"+\" adds the picker's colour.")
                .size(12)
                .color(theme::MUTED),
        ]
        .spacing(12);

        for (gi, g) in self.gradient.iter().enumerate() {
            let zones = (0..ZONE_COUNT as u8).fold(row![].spacing(6), |r, z| {
                r.push(
                    button(text(format!("Z{}", z + 1)).size(12))
                        .padding([5, 10])
                        .style(theme::segment(g.zones.contains(&z)))
                        .on_press(Message::ToggleGroupZone(gi, z)),
                )
            });
            let colors = g.colors.iter().enumerate().fold(row![].spacing(6), |r, (ci, &c)| {
                r.push(
                    button(text("×").size(12).color(iced::Color::WHITE))
                        .width(30)
                        .padding([5, 8])
                        .style(theme::swatch(c, false))
                        .on_press(Message::RemoveGroupColor(gi, ci)),
                )
            });
            col = col.push(
                container(
                    row![
                        text(format!("Group {}", gi + 1)).font(theme::SEMIBOLD).width(70),
                        zones,
                        container(space().width(1).height(24)).style(|_: &Theme| {
                            container::Style { background: Some(theme::BORDER.into()), ..container::Style::default() }
                        }),
                        colors,
                        button(text("+").size(14))
                            .padding([4, 10])
                            .style(theme::ghost)
                            .on_press(Message::AddGroupColor(gi)),
                        space::horizontal(),
                        button(text("Remove").size(12))
                            .padding([5, 10])
                            .style(theme::ghost)
                            .on_press(Message::RemoveGroup(gi)),
                    ]
                    .spacing(12)
                    .align_y(Center),
                )
                .padding(10)
                .style(|_: &Theme| container::Style {
                    background: Some(theme::SURFACE_HI.into()),
                    border: iced::Border { color: theme::BORDER, width: 1.0, radius: 10.0.into() },
                    ..container::Style::default()
                }),
            );
        }
        if let Some(e) = &self.gradient_error {
            col = col.push(inline_error(e));
        }
        container(col).padding(18).width(Fill).style(theme::card).into()
    }
}

/// The keyboard preview: clicks select a zone, hovering outlines one.
struct KeyboardCanvas {
    layout: Layout,
    colors: [Rgb; ZONE_COUNT],
    selected: usize,
    hovered: Option<usize>,
}

impl canvas::Program<Message> for KeyboardCanvas {
    type State = ();

    fn update(&self, _: &mut (), event: &Event, bounds: Rectangle, cursor: Cursor) -> Option<Action<Message>> {
        let zone = cursor.position_in(bounds).and_then(|p| super::keyboard::zone_at(self.layout, bounds.size(), p));
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                zone.map(|z| Action::publish(Message::ZoneClicked(z)).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) if zone != self.hovered => {
                Some(Action::publish(Message::ZoneHovered(zone)))
            }
            _ => None,
        }
    }

    fn draw(&self, _: &(), renderer: &Renderer, _: &Theme, bounds: Rectangle, _: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        super::keyboard::draw(&mut frame, self.layout, &self.colors, Some(self.selected), self.hovered);
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(&self, _: &(), bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        match cursor.position_in(bounds).and_then(|p| super::keyboard::zone_at(self.layout, bounds.size(), p)) {
            Some(_) => mouse::Interaction::Pointer,
            None => mouse::Interaction::default(),
        }
    }
}

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
