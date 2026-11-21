//! Application shell: header with navigation and profiles, the pages, and the status bar.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, space, text, text_input};
use iced::{Center, Element, Fill, Subscription, Theme};

use super::{driver_missing, fans, lighting, theme};
use crate::detect::{self, SecureBoot};
use crate::profiles::{Profile, ProfileStore};
use crate::sysfs::{Device, mock::MockTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Lighting,
    Fans,
}

#[derive(Debug, Clone)]
pub enum Message {
    Nav(Page),
    Lighting(lighting::Message),
    Fans(fans::Message),
    Tick,
    CheckAgain,
    ProfileChosen(String),
    ToggleProfilePanel,
    ProfileName(String),
    IncludeFans(bool),
    SaveProfile,
    DeleteProfile,
}

/// What `main` hands to the app.
#[derive(Clone)]
pub struct Config {
    pub device: Device,
    pub mock: Option<Arc<MockTree>>,
    pub store: Option<ProfileStore>,
}

pub struct App {
    device: Device,
    mock: Option<Arc<MockTree>>,
    driver_present: bool,
    secure_boot: SecureBoot,
    page: Page,
    lighting: lighting::Lighting,
    fans: fans::Fans,
    store: Option<ProfileStore>,
    profiles: Vec<String>,
    current_profile: Option<String>,
    profile_panel: bool,
    profile_name: String,
    include_fans: bool,
    status: Option<(String, bool)>,
}

impl App {
    pub fn new(config: Config) -> App {
        let Config { device, mock, store } = config;
        let driver_present = device.driver_present();
        let mut app = App {
            secure_boot: if driver_present { SecureBoot::Unknown } else { detect::secure_boot_state() },
            lighting: lighting::Lighting::load(&device),
            fans: fans::Fans::load(&device),
            device,
            mock,
            driver_present,
            page: Page::Lighting,
            store,
            profiles: Vec::new(),
            current_profile: None,
            profile_panel: false,
            profile_name: String::new(),
            include_fans: false,
            status: None,
        };
        app.refresh_profiles();
        app
    }

    pub fn title(&self) -> String {
        match &self.current_profile {
            Some(p) => format!("OMEN Control — {p}"),
            None => "OMEN Control".to_owned(),
        }
    }

    pub fn theme(&self) -> Theme {
        theme::theme()
    }

    fn reload(&mut self) {
        self.lighting = lighting::Lighting::load(&self.device);
        self.fans = fans::Fans::load(&self.device);
    }

    fn refresh_profiles(&mut self) {
        if let Some(store) = &self.store {
            match store.list() {
                Ok(p) => self.profiles = p,
                Err(e) => self.status = Some((e.to_string(), true)),
            }
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Nav(p) => self.page = p,
            Message::Lighting(m) => self.lighting.update(m, &self.device),
            Message::Fans(m) => self.fans.update(m, &self.device),
            Message::Tick => {
                if let Some(mock) = &self.mock {
                    let seed = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
                    mock.tick(seed);
                }
                self.fans.set_live(self.device.read_live());
            }
            Message::CheckAgain => {
                self.driver_present = self.device.driver_present();
                if self.driver_present {
                    self.reload();
                } else {
                    self.secure_boot = detect::secure_boot_state();
                }
            }
            Message::ProfileChosen(name) => self.apply_profile(&name),
            Message::ToggleProfilePanel => {
                self.profile_panel = !self.profile_panel;
                if self.profile_panel && self.profile_name.is_empty() {
                    self.profile_name = self.current_profile.clone().unwrap_or_default();
                }
            }
            Message::ProfileName(s) => self.profile_name = s,
            Message::IncludeFans(on) => self.include_fans = on,
            Message::SaveProfile => self.save_profile(),
            Message::DeleteProfile => self.delete_profile(),
        }
    }

    fn apply_profile(&mut self, name: &str) {
        let Some(store) = &self.store else { return };
        match store.load(name) {
            Ok(profile) => {
                let errors = profile.apply(&self.device);
                if let Some(m) = &self.mock {
                    m.tick(0);
                }
                self.reload();
                self.current_profile = Some(name.to_owned());
                self.status = Some(if errors.is_empty() {
                    (format!("Applied profile \"{name}\""), false)
                } else {
                    let list = errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ");
                    (format!("Applied \"{name}\" with errors: {list}"), true)
                });
            }
            Err(e) => self.status = Some((e.to_string(), true)),
        }
    }

    fn save_profile(&mut self) {
        let Some(store) = &self.store else { return };
        let name = self.profile_name.trim().to_owned();
        let profile = Profile::capture(&self.device, self.include_fans);
        match store.save(&name, &profile) {
            Ok(()) => {
                self.current_profile = Some(name.clone());
                self.status = Some((format!("Saved profile \"{name}\""), false));
                self.refresh_profiles();
            }
            Err(e) => self.status = Some((e.to_string(), true)),
        }
    }

    fn delete_profile(&mut self) {
        let Some(store) = &self.store else { return };
        let name = self.profile_name.trim().to_owned();
        match store.delete(&name) {
            Ok(()) => {
                if self.current_profile.as_deref() == Some(name.as_str()) {
                    self.current_profile = None;
                }
                self.status = Some((format!("Deleted profile \"{name}\""), false));
                self.profile_name.clear();
                self.refresh_profiles();
            }
            Err(e) => self.status = Some((e.to_string(), true)),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if !self.driver_present {
            return Subscription::none();
        }
        let mut subs = vec![iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)];
        if self.page == Page::Lighting && self.lighting.animating() {
            subs.push(
                iced::time::every(Duration::from_millis(33)).map(|t| Message::Lighting(lighting::Message::Frame(t))),
            );
        }
        Subscription::batch(subs)
    }

    // -- view ---------------------------------------------------------------------------------------------------------

    pub fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = if self.driver_present {
            let page = match self.page {
                Page::Lighting => self.lighting.view().map(Message::Lighting),
                Page::Fans => self.fans.view().map(Message::Fans),
            };
            scrollable(container(page).padding(24).width(Fill)).height(Fill).into()
        } else {
            scrollable(driver_missing::view(
                self.device.paths().driver.display().to_string(),
                self.secure_boot,
                Message::CheckAgain,
            ))
            .height(Fill)
            .into()
        };

        let mut col = column![self.header()];
        if self.profile_panel && self.driver_present {
            col = col.push(self.profile_panel());
        }
        col.push(body).push(self.status_bar()).into()
    }

    fn header(&self) -> Element<'_, Message> {
        let logo = row![
            text("OMEN").size(22).font(theme::BOLD).color(theme::ACCENT),
            text("CONTROL").size(22).font(theme::BOLD).color(theme::TEXT),
        ]
        .spacing(6);
        let tab = |label: &'static str, p: Page| {
            button(text(label).font(theme::SEMIBOLD).size(14))
                .padding([8, 18])
                .style(theme::segment(self.page == p))
                .on_press_maybe(self.driver_present.then_some(Message::Nav(p)))
        };
        let mut r = row![logo, space().width(24), tab("Lighting", Page::Lighting), tab("Fans", Page::Fans)]
            .spacing(8)
            .align_y(Center);
        r = r.push(space::horizontal());
        if self.mock.is_some() {
            r = r.push(
                container(text("MOCK").size(11).font(theme::BOLD)).padding([3, 8]).style(theme::badge(theme::WARNING)),
            );
        }
        if self.store.is_some() && self.driver_present {
            r = r
                .push(text("Profile").size(13).color(theme::MUTED))
                .push(
                    pick_list(self.profiles.clone(), self.current_profile.clone(), Message::ProfileChosen)
                        .placeholder("Choose…")
                        .padding([7, 12])
                        .width(200),
                )
                .push(
                    button(text(if self.profile_panel { "Close" } else { "Manage" }).size(13))
                        .padding([7, 12])
                        .style(theme::ghost)
                        .on_press(Message::ToggleProfilePanel),
                );
        }
        container(r).padding([14, 24]).width(Fill).style(theme::header).into()
    }

    fn profile_panel(&self) -> Element<'_, Message> {
        let exists = self.profiles.iter().any(|p| p == self.profile_name.trim());
        container(
            row![
                text("Save current settings as").size(13).color(theme::MUTED),
                text_input("Profile name", &self.profile_name)
                    .on_input(Message::ProfileName)
                    .on_submit(Message::SaveProfile)
                    .padding(7)
                    .width(240)
                    .style(theme::input),
                checkbox(self.include_fans).label("Include fan settings").on_toggle(Message::IncludeFans),
                space::horizontal(),
                button(text("Delete").size(13))
                    .padding([7, 14])
                    .style(theme::ghost)
                    .on_press_maybe(exists.then_some(Message::DeleteProfile)),
                button(text(if exists { "Overwrite" } else { "Save" }).size(13).font(theme::SEMIBOLD))
                    .padding([7, 16])
                    .style(theme::primary)
                    .on_press_maybe((!self.profile_name.trim().is_empty()).then_some(Message::SaveProfile)),
            ]
            .spacing(12)
            .align_y(Center),
        )
        .padding([10, 24])
        .width(Fill)
        .style(|_: &Theme| container::Style {
            background: Some(theme::SURFACE.into()),
            border: iced::Border { color: theme::BORDER, width: 1.0, radius: 0.0.into() },
            ..container::Style::default()
        })
        .into()
    }

    fn status_bar(&self) -> Element<'_, Message> {
        let (msg, color) = match &self.status {
            Some((m, true)) => (m.clone(), theme::DANGER),
            Some((m, false)) => (m.clone(), theme::SUCCESS),
            None => (String::new(), theme::MUTED),
        };
        let dot = if self.driver_present { theme::SUCCESS } else { theme::DANGER };
        container(
            row![
                text("●").size(10).color(dot),
                text(self.device.paths().driver.display().to_string()).size(11).color(theme::MUTED),
                space::horizontal(),
                text(msg).size(12).color(color),
            ]
            .spacing(8)
            .align_y(Center),
        )
        .padding([6, 24])
        .width(Fill)
        .style(theme::header)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::sysfs::Paths;

    /// Scenario: Check again finds driver
    #[test]
    fn check_again_finds_driver() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::from_env_value(dir.path().join("driver").to_str());
        let mut app = App::new(Config { device: Device::new(paths.clone()), mock: None, store: None });
        assert!(!app.driver_present);
        fs::create_dir_all(paths.driver.join("rgb_zones")).unwrap();
        app.update(Message::CheckAgain);
        assert!(app.driver_present);
    }

    /// Scenario: Choose profile from header
    #[test]
    fn choose_profile_from_header() {
        let mock = MockTree::create().unwrap();
        let dev = Device::new(mock.paths().clone());
        let dir = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(dir.path().to_owned());
        let mut gaming = Profile::capture(&dev, false);
        gaming.lighting.brightness = Some(30);
        store.save("gaming", &gaming).unwrap();

        let mut app = App::new(Config { device: dev.clone(), mock: None, store: Some(store) });
        assert_eq!(app.profiles, ["gaming"]);
        app.update(Message::ProfileChosen("gaming".into()));
        assert_eq!(dev.read_lighting().brightness, Some(30));
        assert_eq!(app.lighting.brightness, 30);
        assert_eq!(app.current_profile.as_deref(), Some("gaming"));
    }
}
