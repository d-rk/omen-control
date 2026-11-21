//! Inline HSV colour picker: a saturation/value square and a hue bar, drawn on a canvas.

use iced::mouse::{self, Cursor};
use iced::widget::canvas::{self, Action, Event, Frame, Geometry, Path, Stroke, gradient};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

use super::color::{Hsv, hue_color};
use super::theme::{self, with_alpha};

const BAR_W: f32 = 22.0;
const GAP: f32 = 14.0;

pub struct ColorPicker<F, M> {
    pub hsv: Hsv,
    pub on_change: F,
    pub on_release: M,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Drag {
    #[default]
    None,
    Square,
    Hue,
}

fn layout(size: Size) -> (Rectangle, Rectangle) {
    let sq_w = (size.width - BAR_W - GAP).max(10.0);
    (
        Rectangle { x: 0.0, y: 0.0, width: sq_w, height: size.height },
        Rectangle { x: sq_w + GAP, y: 0.0, width: BAR_W, height: size.height },
    )
}

impl<F, M> ColorPicker<F, M>
where
    F: Fn(Hsv) -> M,
{
    fn value_at(&self, drag: Drag, size: Size, p: Point) -> Hsv {
        let (sq, bar) = layout(size);
        match drag {
            Drag::Square => Hsv::new(
                self.hsv.h,
                ((p.x - sq.x) / sq.width).clamp(0.0, 1.0),
                1.0 - ((p.y - sq.y) / sq.height).clamp(0.0, 1.0),
            ),
            Drag::Hue => Hsv::new(((p.y - bar.y) / bar.height).clamp(0.0, 0.9999) * 360.0, self.hsv.s, self.hsv.v),
            Drag::None => self.hsv,
        }
    }
}

impl<F, M> canvas::Program<M> for ColorPicker<F, M>
where
    F: Fn(Hsv) -> M,
    M: Clone,
{
    type State = Drag;

    fn update(&self, drag: &mut Drag, event: &Event, bounds: Rectangle, cursor: Cursor) -> Option<Action<M>> {
        let size = bounds.size();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let p = cursor.position_in(bounds)?;
                let (sq, bar) = layout(size);
                *drag = if sq.contains(p) {
                    Drag::Square
                } else if bar.contains(p) {
                    Drag::Hue
                } else {
                    return None;
                };
                Some(Action::publish((self.on_change)(self.value_at(*drag, size, p))).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if *drag != Drag::None => {
                let pos = cursor.position()?;
                let p = Point::new(pos.x - bounds.x, pos.y - bounds.y);
                Some(Action::publish((self.on_change)(self.value_at(*drag, size, p))).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if *drag != Drag::None => {
                *drag = Drag::None;
                Some(Action::publish(self.on_release.clone()).and_capture())
            }
            _ => None,
        }
    }

    fn draw(&self, _: &Drag, renderer: &Renderer, _: &Theme, bounds: Rectangle, _: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let (sq, bar) = layout(bounds.size());
        let r = 10.0;

        // Saturation (left to right) over the hue, then value (top to bottom) to black.
        let sq_path = Path::rounded_rectangle(sq.position(), sq.size(), r.into());
        frame.fill(
            &sq_path,
            gradient::Linear::new(Point::new(sq.x, sq.y), Point::new(sq.x + sq.width, sq.y))
                .add_stop(0.0, Color::WHITE)
                .add_stop(1.0, theme::to_color(hue_color(self.hsv.h))),
        );
        frame.fill(
            &sq_path,
            gradient::Linear::new(Point::new(sq.x, sq.y), Point::new(sq.x, sq.y + sq.height))
                .add_stop(0.0, Color::TRANSPARENT)
                .add_stop(1.0, Color::BLACK),
        );
        frame.stroke(&sq_path, Stroke::default().with_color(theme::BORDER).with_width(1.0));

        let bar_path = Path::rounded_rectangle(bar.position(), bar.size(), (BAR_W / 2.0).into());
        let mut hue = gradient::Linear::new(Point::new(bar.x, bar.y), Point::new(bar.x, bar.y + bar.height));
        for i in 0..=6 {
            hue = hue.add_stop(i as f32 / 6.0, theme::to_color(hue_color(i as f32 * 60.0)));
        }
        frame.fill(&bar_path, hue);
        frame.stroke(&bar_path, Stroke::default().with_color(theme::BORDER).with_width(1.0));

        // Markers.
        let cur = theme::to_color(self.hsv.to_rgb());
        let m = Point::new(sq.x + self.hsv.s * sq.width, sq.y + (1.0 - self.hsv.v) * sq.height);
        frame.fill(&Path::circle(m, 9.0), with_alpha(Color::BLACK, 0.35));
        frame.fill(&Path::circle(m, 7.0), cur);
        frame.stroke(&Path::circle(m, 7.5), Stroke::default().with_color(Color::WHITE).with_width(2.5));

        let hy = bar.y + self.hsv.h / 360.0 * bar.height;
        let knob = Rectangle { x: bar.x - 3.0, y: hy - 5.0, width: BAR_W + 6.0, height: 10.0 };
        let knob_path = Path::rounded_rectangle(knob.position(), knob.size(), 5.0.into());
        frame.fill(&knob_path, theme::to_color(hue_color(self.hsv.h)));
        frame.stroke(&knob_path, Stroke::default().with_color(Color::WHITE).with_width(2.5));

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(&self, drag: &Drag, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if *drag != Drag::None || cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}
