//! iced user interface.

pub mod app;
mod color;
mod color_picker;
mod curve;
mod driver_missing;
mod fans;
mod keyboard;
mod lighting;
mod preview;
pub mod theme;

use iced::Element;
use iced::widget::{container, text};

fn section_title<'a, M: 'a>(s: impl text::IntoFragment<'a>) -> Element<'a, M> {
    text(s).size(15).font(theme::BOLD).color(theme::TEXT).into()
}

fn field_label<'a, M: 'a>(s: impl text::IntoFragment<'a>) -> Element<'a, M> {
    text(s).size(13).color(theme::MUTED).width(110).into()
}

fn inline_error<'a, M: 'a>(s: impl text::IntoFragment<'a>) -> Element<'a, M> {
    container(text(s).size(13)).padding([8, 12]).width(iced::Fill).style(theme::error_box).into()
}
