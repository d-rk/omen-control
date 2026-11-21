//! Shown instead of the pages when the driver's sysfs root is missing.

use iced::widget::{button, column, container, row, text};
use iced::{Center, Element, Fill, Font};

use super::theme;
use crate::detect::SecureBoot;

pub fn view<'a, M: Clone + 'a>(root: String, secure_boot: SecureBoot, check_again: M) -> Element<'a, M> {
    let step = |n: &'static str, title: &'static str, body: Element<'a, M>| -> Element<'a, M> {
        row![
            container(text(n).font(theme::BOLD).color(theme::ACCENT))
                .padding([2, 9])
                .style(theme::badge(theme::ACCENT)),
            column![text(title).font(theme::SEMIBOLD), body].spacing(6).width(Fill),
        ]
        .spacing(14)
        .into()
    };
    let code = |s: &'static str| -> Element<'a, M> {
        container(text(s).font(Font::MONOSPACE).size(12).color(theme::TEXT))
            .padding([8, 12])
            .width(Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::color!(0x0b0c10).into()),
                border: iced::Border { color: theme::BORDER, width: 1.0, radius: 8.0.into() },
                ..container::Style::default()
            })
            .into()
    };
    let muted = |s: &'static str| -> Element<'a, M> { text(s).size(13).color(theme::MUTED).into() };

    let sb_note: Element<'a, M> = match secure_boot {
        SecureBoot::Enabled => text("Secure Boot is enabled on this machine: the module must be signed with an enrolled MOK, or modprobe fails with \"Key was rejected by service\".")
            .size(13)
            .color(theme::WARNING)
            .into(),
        SecureBoot::Disabled => muted("Secure Boot is disabled on this machine, so module signing is not needed."),
        SecureBoot::Unknown => muted("If Secure Boot is enabled, the module must be signed with an enrolled MOK (check with `mokutil --sb-state`)."),
    };

    let body = column![
        text("Driver not found").size(28).font(theme::BOLD),
        text(format!("omen-control could not find the omen-rgb-keyboard driver at {root}."))
            .color(theme::MUTED),
        step(
            "1",
            "Install the omen-rgb-keyboard DKMS driver",
            column![
                muted("omen-control is only a front end; the kernel driver does the work."),
                code("git clone https://github.com/alessandromrc/omen-rgb-keyboard.git\ncd omen-rgb-keyboard && sudo ./install.sh"),
            ]
            .spacing(8)
            .into(),
        ),
        step(
            "2",
            "Allow your user to write the controls",
            column![
                muted("The udev rules give the input group write access. Log out and back in afterwards."),
                code("sudo ./install-udev-rules.sh"),
            ]
            .spacing(8)
            .into(),
        ),
        step(
            "3",
            "Blacklist hp_wmi",
            column![
                muted("hp_wmi and this driver fight over the same WMI interface."),
                code("echo \"blacklist hp_wmi\" | sudo tee /etc/modprobe.d/blacklist-hp.conf\nsudo modprobe -r hp_wmi && sudo modprobe omen_rgb_keyboard"),
            ]
            .spacing(8)
            .into(),
        ),
        step("4", "Secure Boot", sb_note),
        row![
            button(text("Check again").font(theme::SEMIBOLD)).padding([10, 22]).style(theme::primary).on_press(check_again),
            text("Developing without the hardware? Run with --mock.").size(12).color(theme::MUTED),
        ]
        .spacing(16)
        .align_y(Center),
    ]
    .spacing(22)
    .max_width(720);

    container(container(body).padding(32).style(theme::card)).center_x(Fill).padding(32).into()
}
