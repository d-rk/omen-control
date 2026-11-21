//! omen-control: desktop UI for the omen-rgb-keyboard driver.

mod detect;
mod profiles;
mod sysfs;
mod ui;

use std::sync::Arc;

use profiles::ProfileStore;
use sysfs::{Device, Paths, mock::MockTree};
use ui::app::{App, Config};

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
        .window_size((1280.0, 880.0))
        .antialiasing(true)
        .run()
}
