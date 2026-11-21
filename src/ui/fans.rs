//! Fans page: live readings, thermal profile, switches, curve editor and power limits.

use iced::mouse::{self, Cursor};
use iced::widget::canvas::{self, Action, Event, Frame, Geometry, Path, Stroke, Text, gradient};
use iced::widget::{
    button, canvas as canvas_widget, checkbox, column, container, row, space, text, text_input, toggler,
};
use iced::{Center, Color, Element, Fill, Point, Rectangle, Renderer, Theme, alignment};

use super::curve::{self, CHART_MAX_TEMP, CHART_MIN_TEMP};
use super::theme::{self, with_alpha};
use super::{field_label, inline_error, section_title};
use crate::sysfs::{CurvePoint, Device, FanState, GpuCurve, PowerLimits, ThermalProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CurveTab {
    #[default]
    Cpu,
    Gpu,
}

#[derive(Debug, Clone)]
pub enum Message {
    Profile(ThermalProfile),
    MaxFan(bool),
    CurveEnable(bool),
    Tab(CurveTab),
    CurveDrag { idx: usize, temp: f32, percent: f32 },
    CurveAdd { temp: f32, percent: f32 },
    CurveRemove(usize),
    GpuFollows(bool),
    ApplyCurves,
    RevertCurves,
    Pl1(String),
    Pl2(String),
    PowerApply,
    PowerReset,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Available {
    pub thermal_profile: bool,
    pub max_fan: bool,
    pub curve_enable: bool,
    pub cpu_curve: bool,
    pub gpu_curve: bool,
    pub power_limits: bool,
}

#[derive(Debug, Clone)]
pub struct Fans {
    pub available: Available,
    pub live: FanState,
    pub thermal_profile: Option<ThermalProfile>,
    pub max_fan: bool,
    pub curve_enable: bool,
    pub tab: CurveTab,
    pub cpu_curve: Vec<CurvePoint>,
    pub gpu_curve: Vec<CurvePoint>,
    pub gpu_follows: bool,
    pub dirty: bool,
    pub power: Option<PowerLimits>,
    pub pl1: String,
    pub pl2: String,
    pub power_error: Option<String>,
    pub error: Option<String>,
}

/// Used when the driver has no curve to show.
fn fallback_curve() -> Vec<CurvePoint> {
    vec![CurvePoint::new(40, 0), CurvePoint::new(60, 40), CurvePoint::new(75, 70), CurvePoint::new(90, 100)]
}

pub fn fmt_rpm(rpm: Option<u32>) -> String {
    rpm.map_or_else(|| "n/a".to_owned(), |r| format!("{r}"))
}

pub fn fmt_temp(t: Option<f32>) -> String {
    t.map_or_else(|| "n/a".to_owned(), |t| format!("{t:.0}"))
}

impl Fans {
    pub fn load(dev: &Device) -> Fans {
        let s = dev.read_fans();
        let mut page = Fans {
            available: Available {
                thermal_profile: s.thermal_profile.is_some(),
                max_fan: s.max_fan.is_some(),
                curve_enable: s.curve_enable.is_some(),
                cpu_curve: s.cpu_curve.is_some(),
                gpu_curve: s.gpu_curve.is_some(),
                power_limits: matches!(s.power_limits, Some(p) if p != PowerLimits::Unsupported),
            },
            thermal_profile: s.thermal_profile,
            max_fan: s.max_fan.unwrap_or(false),
            curve_enable: s.curve_enable.unwrap_or(false),
            tab: CurveTab::Cpu,
            cpu_curve: Vec::new(),
            gpu_curve: Vec::new(),
            gpu_follows: true,
            dirty: false,
            power: s.power_limits,
            pl1: String::new(),
            pl2: String::new(),
            power_error: None,
            error: None,
            live: FanState::default(),
        };
        page.set_curves(&s);
        if let Some(PowerLimits::Custom { pl1, pl2 }) = s.power_limits {
            page.pl1 = pl1.to_string();
            page.pl2 = pl2.to_string();
        }
        page.live = s;
        page
    }

    fn set_curves(&mut self, s: &FanState) {
        self.cpu_curve = s.cpu_curve.clone().filter(|c| c.len() >= 2).unwrap_or_else(fallback_curve);
        match &s.gpu_curve {
            Some(GpuCurve::Own(p)) if p.len() >= 2 => {
                self.gpu_follows = false;
                self.gpu_curve = p.clone();
            }
            _ => {
                self.gpu_follows = true;
                self.gpu_curve = self.cpu_curve.clone();
            }
        }
        self.dirty = false;
    }

    pub fn set_live(&mut self, live: FanState) {
        self.live.cpu_rpm = live.cpu_rpm;
        self.live.gpu_rpm = live.gpu_rpm;
        self.live.cpu_temp = live.cpu_temp;
        self.live.gpu_temp = live.gpu_temp;
        self.live.platform_profile = live.platform_profile;
    }

    pub fn power_limits_enabled(&self) -> bool {
        self.available.power_limits
    }

    fn editing_gpu(&self) -> bool {
        self.tab == CurveTab::Gpu && !self.gpu_follows
    }

    fn edited_curve(&mut self) -> Option<&mut Vec<CurvePoint>> {
        match self.tab {
            CurveTab::Cpu => Some(&mut self.cpu_curve),
            CurveTab::Gpu if !self.gpu_follows => Some(&mut self.gpu_curve),
            CurveTab::Gpu => None,
        }
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

    pub fn update(&mut self, msg: Message, dev: &Device) {
        match msg {
            Message::Profile(p) => {
                if self.report(dev.write_thermal_profile(p)) {
                    self.thermal_profile = Some(p);
                    // The driver shows the preset's curve until a custom one is saved.
                    if !self.dirty {
                        let s = dev.read_fans();
                        self.set_curves(&s);
                    }
                }
            }
            Message::MaxFan(on) => {
                if self.report(dev.write_max_fan(on)) {
                    self.max_fan = on;
                }
            }
            Message::CurveEnable(on) => {
                if self.report(dev.write_curve_enable(on)) {
                    self.curve_enable = on;
                }
            }
            Message::Tab(t) => self.tab = t,
            Message::CurveDrag { idx, temp, percent } => {
                if let Some(c) = self.edited_curve() {
                    curve::drag_to(c, idx, temp, percent);
                    self.dirty = true;
                }
            }
            Message::CurveAdd { temp, percent } => {
                if let Some(c) = self.edited_curve()
                    && curve::insert_at(c, temp, percent)
                {
                    self.dirty = true;
                }
            }
            Message::CurveRemove(idx) => {
                if let Some(c) = self.edited_curve()
                    && curve::remove(c, idx)
                {
                    self.dirty = true;
                }
            }
            Message::GpuFollows(on) => {
                self.gpu_follows = on;
                if !on && self.gpu_curve.len() < 2 {
                    self.gpu_curve = self.cpu_curve.clone();
                }
                self.dirty = true;
            }
            Message::ApplyCurves => {
                let mut ok = self.report(dev.write_cpu_curve(&self.cpu_curve));
                if ok && self.available.gpu_curve {
                    let gpu = (!self.gpu_follows).then_some(self.gpu_curve.as_slice());
                    ok = self.report(dev.write_gpu_curve(gpu));
                }
                if ok {
                    self.dirty = false;
                    // Writing CPU points turns the custom-curve switch on in the driver.
                    self.curve_enable = dev.read_fans().curve_enable.unwrap_or(true);
                }
            }
            Message::RevertCurves => {
                let s = dev.read_fans();
                self.set_curves(&s);
            }
            Message::Pl1(s) => {
                self.pl1 = s;
                self.power_error = None;
            }
            Message::Pl2(s) => {
                self.pl2 = s;
                self.power_error = None;
            }
            Message::PowerApply => {
                let parsed = match (self.pl1.trim().parse::<u32>(), self.pl2.trim().parse::<u32>()) {
                    (Ok(a), Ok(b)) => PowerLimits::custom(a, b),
                    _ => Err("PL1 and PL2 must be whole numbers of watts".to_owned()),
                };
                match parsed {
                    Ok(p) => {
                        if self.report(dev.write_power_limits(p)) {
                            self.power = Some(p);
                        }
                    }
                    Err(e) => self.power_error = Some(e),
                }
            }
            Message::PowerReset => {
                if self.report(dev.write_power_limits(PowerLimits::Default)) {
                    self.power = Some(PowerLimits::Default);
                    self.pl1.clear();
                    self.pl2.clear();
                    self.power_error = None;
                }
            }
        }
    }

    // -- view ---------------------------------------------------------------------------------------------------------

    pub fn view(&self) -> Element<'_, Message> {
        let gauges = row![
            gauge("CPU FAN", fmt_rpm(self.live.cpu_rpm), "RPM", theme::CPU),
            gauge("GPU FAN", fmt_rpm(self.live.gpu_rpm), "RPM", theme::GPU),
            gauge("CPU TEMP", fmt_temp(self.live.cpu_temp), "°C", theme::CPU),
            gauge("GPU TEMP", fmt_temp(self.live.gpu_temp), "°C", theme::GPU),
        ]
        .spacing(18);

        let left = column![self.curve_card()].width(Fill);
        let right = column![self.mode_card(), self.power_card()].spacing(18).width(360);
        let mut page = column![gauges, row![left, right].spacing(18)].spacing(18);
        if let Some(e) = &self.error {
            page = page.push(inline_error(e));
        }
        page.into()
    }

    fn mode_card(&self) -> Element<'_, Message> {
        let a = &self.available;
        let mut col = column![section_title("Performance")].spacing(14);
        if a.thermal_profile {
            let seg = ThermalProfile::ALL.iter().fold(row![].spacing(6), |r, &p| {
                let label = match p {
                    ThermalProfile::Silent => "Silent",
                    ThermalProfile::Normal => "Balanced",
                    ThermalProfile::Performance => "Performance",
                };
                r.push(
                    button(text(label).font(theme::SEMIBOLD).size(13).width(Fill).align_x(Center))
                        .width(Fill)
                        .padding([9, 6])
                        .style(theme::segment(self.thermal_profile == Some(p)))
                        .on_press(Message::Profile(p)),
                )
            });
            col = col.push(seg);
        }
        if let Some(pp) = &self.live.platform_profile {
            col = col.push(
                row![
                    text("Platform profile").size(12).color(theme::MUTED),
                    space::horizontal(),
                    container(text(pp.clone()).size(12).font(theme::SEMIBOLD))
                        .padding([2, 8])
                        .style(theme::badge(theme::MUTED)),
                ]
                .align_y(Center),
            );
        }
        if a.max_fan {
            col = col.push(switch_row("Max fan", "Both fans at full speed.", self.max_fan, Message::MaxFan));
        }
        if a.curve_enable {
            col = col.push(switch_row(
                "Custom curve",
                "Off: the EC controls the fans.",
                self.curve_enable,
                Message::CurveEnable,
            ));
        }
        container(col).padding(18).width(Fill).style(theme::card).into()
    }

    fn power_card(&self) -> Element<'_, Message> {
        let enabled = self.power_limits_enabled();
        let current = match self.power {
            Some(PowerLimits::Custom { pl1, pl2 }) => format!("{pl1} W / {pl2} W"),
            Some(PowerLimits::Default) => "BIOS default".to_owned(),
            Some(PowerLimits::Unsupported) => "Not supported".to_owned(),
            None => "Unavailable".to_owned(),
        };
        let input = |placeholder: &'static str, value: &str, f: fn(String) -> Message| {
            let i = text_input(placeholder, value).padding(8).style(theme::input).width(Fill);
            if enabled { i.on_input(f).on_submit(Message::PowerApply) } else { i }
        };
        let mut col = column![
            row![section_title("CPU power limits"), space::horizontal(), text(current).size(12).color(theme::MUTED)]
                .align_y(Center),
            row![
                column![field_label("PL1 (W)"), input("e.g. 65", &self.pl1, Message::Pl1)].spacing(6).width(Fill),
                column![field_label("PL2 (W)"), input("e.g. 90", &self.pl2, Message::Pl2)].spacing(6).width(Fill),
            ]
            .spacing(12),
            row![
                button(text("Reset to default").width(Fill).align_x(Center))
                    .width(Fill)
                    .padding(9)
                    .style(theme::ghost)
                    .on_press_maybe(enabled.then_some(Message::PowerReset)),
                button(text("Apply").font(theme::SEMIBOLD).width(Fill).align_x(Center))
                    .width(Fill)
                    .padding(9)
                    .style(theme::primary)
                    .on_press_maybe(enabled.then_some(Message::PowerApply)),
            ]
            .spacing(12),
        ]
        .spacing(12);
        if let Some(e) = &self.power_error {
            col = col.push(inline_error(e));
        }
        container(col).padding(18).width(Fill).style(theme::card).into()
    }

    fn curve_card(&self) -> Element<'_, Message> {
        let tab = |label: &'static str, t: CurveTab| {
            button(text(label).font(theme::SEMIBOLD).size(13))
                .padding([7, 16])
                .style(theme::segment(self.tab == t))
                .on_press(Message::Tab(t))
        };
        let (points, other, live_temp, accent) = match self.tab {
            CurveTab::Cpu => {
                (&self.cpu_curve, (!self.gpu_follows).then_some(&self.gpu_curve), self.live.cpu_temp, theme::CPU)
            }
            CurveTab::Gpu if self.gpu_follows => (&self.cpu_curve, None, self.live.gpu_temp, theme::GPU),
            CurveTab::Gpu => (&self.gpu_curve, Some(&self.cpu_curve), self.live.gpu_temp, theme::GPU),
        };
        let editable = self.tab == CurveTab::Cpu || self.editing_gpu();
        let chart = canvas_widget(CurveChart {
            points,
            other: other.map(Vec::as_slice),
            live_temp,
            accent,
            editable: editable && self.available.cpu_curve,
        })
        .width(Fill)
        .height(360);

        let mut header =
            row![section_title("Fan curve"), space::horizontal(), tab("CPU", CurveTab::Cpu)].spacing(8).align_y(Center);
        if self.available.gpu_curve {
            header = header.push(tab("GPU", CurveTab::Gpu));
        }
        let hint = if self.tab == CurveTab::Gpu && self.gpu_follows {
            "The GPU fan follows the CPU curve. Untick \"GPU follows CPU\" to give it its own."
        } else {
            "Drag points to edit · click empty space to add · right-click a point to remove (2–8 points)."
        };
        let mut footer = row![].spacing(10).align_y(Center);
        if self.available.gpu_curve {
            footer = footer.push(checkbox(self.gpu_follows).label("GPU follows CPU").on_toggle(Message::GpuFollows));
        }
        footer = footer.push(space::horizontal());
        if self.dirty {
            footer =
                footer.push(container(text("Unsaved").size(12)).padding([2, 8]).style(theme::badge(theme::WARNING)));
        }
        footer = footer
            .push(
                button(text("Revert"))
                    .padding([8, 16])
                    .style(theme::ghost)
                    .on_press_maybe(self.dirty.then_some(Message::RevertCurves)),
            )
            .push(
                button(text("Apply curve").font(theme::SEMIBOLD))
                    .padding([8, 16])
                    .style(theme::primary)
                    .on_press_maybe(self.available.cpu_curve.then_some(Message::ApplyCurves)),
            );

        container(column![header, chart, text(hint).size(12).color(theme::MUTED), footer].spacing(12))
            .padding(18)
            .width(Fill)
            .style(theme::card)
            .into()
    }
}

fn switch_row<'a>(label: &'a str, help: &'a str, on: bool, f: impl Fn(bool) -> Message + 'a) -> Element<'a, Message> {
    row![
        column![text(label).font(theme::SEMIBOLD), text(help).size(12).color(theme::MUTED)].spacing(2),
        space::horizontal(),
        toggler(on).on_toggle(f).size(22),
    ]
    .align_y(Center)
    .into()
}

fn gauge<'a>(label: &'a str, value: String, unit: &'a str, accent: Color) -> Element<'a, Message> {
    container(
        column![
            text(label).size(11).font(theme::BOLD).color(theme::MUTED),
            row![text(value).size(34).font(theme::BOLD).color(theme::TEXT), text(unit).size(14).color(accent)]
                .spacing(6)
                .align_y(iced::Bottom),
            container(space().width(Fill).height(3)).style(move |_: &Theme| container::Style {
                background: Some(
                    iced::gradient::Linear::new(0.0)
                        .add_stop(0.0, accent)
                        .add_stop(1.0, with_alpha(accent, 0.0))
                        .into(),
                ),
                border: iced::Border { radius: 2.0.into(), ..Default::default() },
                ..container::Style::default()
            }),
        ]
        .spacing(6),
    )
    .padding(16)
    .width(Fill)
    .style(theme::card)
    .into()
}

/// Interactive curve chart.
struct CurveChart<'a> {
    points: &'a [CurvePoint],
    /// The other fan's curve, drawn faintly for comparison.
    other: Option<&'a [CurvePoint]>,
    live_temp: Option<f32>,
    accent: Color,
    editable: bool,
}

#[derive(Debug, Default)]
struct ChartState {
    dragging: Option<usize>,
    hovered: Option<usize>,
}

impl canvas::Program<Message> for CurveChart<'_> {
    type State = ChartState;

    fn update(&self, st: &mut ChartState, event: &Event, bounds: Rectangle, cursor: Cursor) -> Option<Action<Message>> {
        if !self.editable {
            return None;
        }
        let plot = curve::plot_area(bounds.size());
        let local = |c: Cursor| c.position().map(|p| Point::new(p.x - bounds.x, p.y - bounds.y));
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let p = cursor.position_in(bounds)?;
                if let Some(i) = curve::point_at(self.points, plot, p) {
                    st.dragging = Some(i);
                    return Some(Action::capture());
                }
                if !plot.contains(p) {
                    return None;
                }
                let (temp, percent) = curve::from_screen(plot, p);
                Some(Action::publish(Message::CurveAdd { temp, percent }).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let p = cursor.position_in(bounds)?;
                let i = curve::point_at(self.points, plot, p)?;
                Some(Action::publish(Message::CurveRemove(i)).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(idx) = st.dragging {
                    let (temp, percent) = curve::from_screen(plot, local(cursor)?);
                    return Some(Action::publish(Message::CurveDrag { idx, temp, percent }).and_capture());
                }
                let h = cursor.position_in(bounds).and_then(|p| curve::point_at(self.points, plot, p));
                if h != st.hovered {
                    st.hovered = h;
                    return Some(Action::request_redraw());
                }
                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if st.dragging.is_some() => {
                st.dragging = None;
                Some(Action::capture())
            }
            _ => None,
        }
    }

    fn draw(&self, st: &ChartState, renderer: &Renderer, _: &Theme, bounds: Rectangle, _: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let plot = curve::plot_area(bounds.size());
        let grid = Stroke::default().with_color(with_alpha(Color::WHITE, 0.06)).with_width(1.0);
        let label = |frame: &mut Frame, s: String, p: Point, ax: alignment::Horizontal, ay: alignment::Vertical| {
            frame.fill_text(Text {
                content: s,
                position: p,
                color: theme::MUTED,
                size: 11.0.into(),
                align_x: ax.into(),
                align_y: ay,
                ..Text::default()
            });
        };

        for pct in (0..=100).step_by(20) {
            let y = curve::to_screen(plot, CHART_MIN_TEMP, pct as f32).y;
            frame.stroke(&Path::line(Point::new(plot.x, y), Point::new(plot.x + plot.width, y)), grid);
            label(
                &mut frame,
                format!("{pct}%"),
                Point::new(plot.x - 8.0, y),
                alignment::Horizontal::Right,
                alignment::Vertical::Center,
            );
        }
        for t in (CHART_MIN_TEMP as u32..=CHART_MAX_TEMP as u32).step_by(10) {
            let x = curve::to_screen(plot, t as f32, 0.0).x;
            frame.stroke(&Path::line(Point::new(x, plot.y), Point::new(x, plot.y + plot.height)), grid);
            label(
                &mut frame,
                format!("{t}°"),
                Point::new(x, plot.y + plot.height + 8.0),
                alignment::Horizontal::Center,
                alignment::Vertical::Top,
            );
        }

        let screen = |pts: &[CurvePoint]| -> Vec<Point> {
            let mut s: Vec<Point> =
                pts.iter().map(|p| curve::to_screen(plot, f32::from(p.temp), f32::from(p.percent))).collect();
            // Flat extension to the chart edges, as the driver clamps outside the points.
            if let (Some(first), Some(last)) = (s.first().copied(), s.last().copied()) {
                s.insert(0, Point::new(plot.x, first.y));
                s.push(Point::new(plot.x + plot.width, last.y));
            }
            s
        };
        let polyline = |pts: &[Point]| {
            Path::new(|b| {
                for (i, p) in pts.iter().enumerate() {
                    if i == 0 { b.move_to(*p) } else { b.line_to(*p) }
                }
            })
        };

        if let Some(other) = self.other {
            let s = screen(other);
            frame.stroke(&polyline(&s), Stroke::default().with_color(with_alpha(theme::MUTED, 0.45)).with_width(1.5));
        }

        let s = screen(self.points);
        if s.len() >= 2 {
            let area = Path::new(|b| {
                b.move_to(Point::new(plot.x, plot.y + plot.height));
                for p in &s {
                    b.line_to(*p);
                }
                b.line_to(Point::new(plot.x + plot.width, plot.y + plot.height));
                b.close();
            });
            frame.fill(
                &area,
                gradient::Linear::new(Point::new(0.0, plot.y), Point::new(0.0, plot.y + plot.height))
                    .add_stop(0.0, with_alpha(self.accent, 0.35))
                    .add_stop(1.0, with_alpha(self.accent, 0.02)),
            );
            frame.stroke(&polyline(&s), Stroke::default().with_color(with_alpha(self.accent, 0.3)).with_width(7.0));
            frame.stroke(&polyline(&s), Stroke::default().with_color(self.accent).with_width(2.5));
        }

        if let Some(t) = self.live_temp
            && (CHART_MIN_TEMP..=CHART_MAX_TEMP).contains(&t)
        {
            let x = curve::to_screen(plot, t, 0.0).x;
            frame.stroke(
                &Path::line(Point::new(x, plot.y), Point::new(x, plot.y + plot.height)),
                Stroke::default().with_color(with_alpha(Color::WHITE, 0.5)).with_width(1.0),
            );
            label(
                &mut frame,
                format!("{t:.0}°C now"),
                Point::new(x + 6.0, plot.y + 4.0),
                alignment::Horizontal::Left,
                alignment::Vertical::Top,
            );
        }

        for (i, p) in self.points.iter().enumerate() {
            let c = curve::to_screen(plot, f32::from(p.temp), f32::from(p.percent));
            let active = st.dragging == Some(i) || st.hovered == Some(i);
            let r = if active { 8.0 } else { 6.0 };
            frame.fill(&Path::circle(c, r + 4.0), with_alpha(self.accent, if active { 0.35 } else { 0.18 }));
            frame.fill(&Path::circle(c, r), theme::SURFACE);
            frame.stroke(&Path::circle(c, r), Stroke::default().with_color(self.accent).with_width(2.5));
            if active {
                frame.fill_text(Text {
                    content: format!("{}°C · {}%", p.temp, p.percent),
                    position: Point::new(c.x, c.y - 16.0),
                    color: theme::TEXT,
                    size: 12.0.into(),
                    font: theme::SEMIBOLD,
                    align_x: alignment::Horizontal::Center.into(),
                    align_y: alignment::Vertical::Bottom,
                    ..Text::default()
                });
            }
        }
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(&self, st: &ChartState, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if !self.editable {
            return mouse::Interaction::default();
        }
        if st.dragging.is_some() {
            return mouse::Interaction::Grabbing;
        }
        let plot = curve::plot_area(bounds.size());
        match cursor.position_in(bounds) {
            Some(p) if curve::point_at(self.points, plot, p).is_some() => mouse::Interaction::Grab,
            Some(p) if plot.contains(p) => mouse::Interaction::Crosshair,
            _ => mouse::Interaction::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::sysfs::mock::MockTree;

    fn setup() -> (MockTree, Device, Fans) {
        let mock = MockTree::create().unwrap();
        let dev = Device::new(mock.paths().clone());
        let page = Fans::load(&dev);
        (mock, dev, page)
    }

    fn file(mock: &MockTree, name: &str) -> String {
        fs::read_to_string(mock.paths().fan(name)).unwrap()
    }

    /// Scenario: Unavailable RPM shown as n/a
    #[test]
    fn unavailable_rpm_shown_as_na() {
        let (mock, dev, mut page) = setup();
        fs::write(mock.paths().fan("gpu_fan_rpm"), "n/a\n").unwrap();
        page.set_live(dev.read_live());
        assert_eq!(fmt_rpm(page.live.gpu_rpm), "n/a");
    }

    /// Scenario: Select thermal profile
    #[test]
    fn select_thermal_profile() {
        let (mock, dev, mut page) = setup();
        page.update(Message::Profile(ThermalProfile::Performance), &dev);
        assert_eq!(file(&mock, "thermal_profile"), "performance");
        assert_eq!(page.thermal_profile, Some(ThermalProfile::Performance));
    }

    /// Scenario: Toggle max fan
    #[test]
    fn toggle_max_fan() {
        let (mock, dev, mut page) = setup();
        page.update(Message::MaxFan(true), &dev);
        assert_eq!(file(&mock, "max_fan"), "1");
    }

    /// Scenario: Apply with GPU following CPU
    #[test]
    fn apply_with_gpu_following_cpu() {
        let (mock, dev, mut page) = setup();
        page.update(Message::GpuFollows(true), &dev);
        page.update(Message::CurveDrag { idx: 1, temp: 56.0, percent: 35.0 }, &dev);
        page.update(Message::ApplyCurves, &dev);
        assert_eq!(file(&mock, "fan_curve"), "40:0 56:35 70:55 85:80 95:100");
        assert_eq!(file(&mock, "gpu_fan_curve"), "cpu");
        assert!(!page.dirty);
    }

    /// Scenario: Revert discards edits
    #[test]
    fn revert_discards_edits() {
        let (_m, dev, mut page) = setup();
        let original = page.cpu_curve.clone();
        page.update(Message::CurveDrag { idx: 2, temp: 72.0, percent: 90.0 }, &dev);
        assert_ne!(page.cpu_curve, original);
        page.update(Message::RevertCurves, &dev);
        assert_eq!(page.cpu_curve, original);
    }

    /// Scenario: Invalid power limits not written
    #[test]
    fn invalid_power_limits_not_written() {
        let (mock, dev, mut page) = setup();
        page.update(Message::Pl1("90".into()), &dev);
        page.update(Message::Pl2("65".into()), &dev);
        page.update(Message::PowerApply, &dev);
        assert_eq!(file(&mock, "power_limits"), "default\n");
        assert!(page.power_error.as_deref().is_some_and(|e| e.contains("PL2 must be at least PL1")));
    }

    /// Scenario: Reset power limits
    #[test]
    fn reset_power_limits() {
        let (mock, dev, mut page) = setup();
        page.update(Message::Pl1("65".into()), &dev);
        page.update(Message::Pl2("90".into()), &dev);
        page.update(Message::PowerApply, &dev);
        assert_eq!(file(&mock, "power_limits"), "65 90");
        page.update(Message::PowerReset, &dev);
        assert_eq!(file(&mock, "power_limits"), "default");
    }

    /// Scenario: Power limits unsupported
    #[test]
    fn power_limits_unsupported() {
        let mock = MockTree::create().unwrap();
        fs::write(mock.paths().fan("power_limits"), "unsupported\n").unwrap();
        let page = Fans::load(&Device::new(mock.paths().clone()));
        assert!(!page.power_limits_enabled());
    }
}
