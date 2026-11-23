# desktop-integration Specification

## Purpose

Makes the desktop show omen-control with its own name and icon in the taskbar, window switcher and application menu.

## Requirements

### Requirement: Application identity
The window SHALL use the application id `omen-control`, which on Wayland is how the desktop matches the window to its
desktop entry and icon. On X11 the window SHALL also carry the icon directly.

#### Scenario: Window matched to desktop entry
- **WHEN** the desktop entry and icon are installed and the app is running on Wayland
- **THEN** the taskbar shows the omen-control icon for the window

### Requirement: Desktop entry and icon
The repository SHALL ship a desktop entry `omen-control.desktop` and a scalable icon `omen-control.svg`, and a script
that installs both for the current user with the entry pointing at a given binary.

#### Scenario: Install for current user
- **WHEN** the user runs the install script with the path of a built binary
- **THEN** the entry appears in `~/.local/share/applications` with that binary as its command, and the icon in
  `~/.local/share/icons/hicolor/scalable/apps`
