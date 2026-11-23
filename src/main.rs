//! omen-control: desktop UI for the omen-rgb-keyboard driver.

mod detect;
mod profiles;
mod sysfs;
mod ui;

use std::sync::Arc;

use profiles::ProfileStore;
use sysfs::{Device, Paths, mock::MockTree};
use ui::app::{App, Config};

/// Wayland matches windows to `omen-control.desktop` by this id; the desktop entry supplies the icon there.
const APP_ID: &str = "omen-control";

/// 64x64 RGBA rendering of `data/omen-control.svg`, used as the window icon on X11 (Wayland ignores it). Regenerate
/// with `magick -background none data/omen-control.svg -resize 64x64 -depth 8 rgba:data/omen-control-64.rgba`.
const ICON_RGBA: &[u8] = include_bytes!("../data/omen-control-64.rgba");

fn window_settings() -> iced::window::Settings {
    iced::window::Settings {
        size: iced::Size::new(1280.0, 880.0),
        icon: iced::window::icon::from_rgba(ICON_RGBA.to_vec(), 64, 64).ok(),
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: APP_ID.to_owned(),
            ..Default::default()
        },
        ..Default::default()
    }
}

const USAGE: &str = "\
Usage: omen-control [--mock]

  --mock     Run against a fake driver tree in a temporary directory.
  -h, --help Show this help.

Environment:
  OMEN_SYSFS_ROOT  Driver sysfs directory (default /sys/devices/platform/omen-rgb-keyboard).";

fn main() -> iced::Result {
    let mut mock = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--mock" => mock = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            other => {
                eprintln!("omen-control: unknown argument {other:?}\n\n{USAGE}");
                std::process::exit(2);
            }
        }
    }

    let (paths, mock) = if mock {
        let tree = match MockTree::create() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("omen-control: creating mock tree: {e}");
                std::process::exit(1);
            }
        };
        eprintln!("omen-control: mock driver tree at {}", tree.paths().driver.display());
        (tree.paths().clone(), Some(Arc::new(tree)))
    } else {
        (Paths::from_env(), None)
    };

    let config = Config { device: Device::new(paths), mock, store: ProfileStore::default_location() };

    iced::application(move || App::new(config.clone()), App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window(window_settings())
        .antialiasing(true)
        .run()
}
