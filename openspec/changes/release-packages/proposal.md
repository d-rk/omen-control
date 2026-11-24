# Proposal

## Why

At the moment the only way to install omen-control is to build it with Cargo and run
`scripts/install-desktop-entry.sh`. People on Fedora, Debian and Ubuntu should be able to install it as a normal
package, and Fedora users should get updates with their regular system updates.

## What Changes

- A `.deb` built with `cargo-deb` from `Cargo.toml` metadata, and an `.rpm` built from `omen-control.spec`, which is
  the same spec file COPR uses. Both install the binary to `/usr/bin/omen-control`, the desktop entry to
  `/usr/share/applications/` and the icon to `/usr/share/icons/hicolor/scalable/apps/`. They declare the Wayland
  libraries the app loads at runtime. Targets: current Fedora, Debian stable and Ubuntu LTS, on Wayland.
- A release tarball with the binary, desktop entry, icon, install script, licence and readme, for other distributions.
- A GitHub Actions workflow that runs when a `v*` tag is pushed. It checks that the tag matches the crate version,
  builds the tarball, `.deb` and `.rpm`, and attaches them to the GitHub release.
- An `omen-control.spec` and a `.packit.yaml`. When a GitHub release is published, Packit builds the app in a Fedora
  COPR project, so users can run `dnf copr enable` and then get updates through `dnf upgrade`.
- README: install instructions for each channel, and the packaging item removed from the TODO list.

Out of scope:

- X11 sessions, openSUSE, and older distribution releases.
- Packaging the omen-rgb-keyboard kernel driver (DKMS or akmod).
- Shipping its udev rules or the `hp_wmi` blacklist.
- AppStream metainfo.
- Homebrew, Flatpak, Snap, AppImage, OBS and apt repositories.

## Capabilities

### New Capabilities
- `release-packaging`: what a release produces (packages, tarball, COPR build), where each package installs its
  files, and what it declares as dependencies.

### Modified Capabilities

None. The desktop entry and icon in `desktop-integration` are reused unchanged. `Exec=omen-control` already resolves
to `/usr/bin/omen-control` through `PATH`.

## Impact

- New files:
  - `Cargo.toml`: a `[package.metadata.deb]` section
  - `omen-control.spec`
  - `.packit.yaml`
  - `.github/workflows/release.yml`
  - README changes
- No changes to the application code.
- External setup, done once by the maintainer:
  - a Fedora account and a COPR project
  - the Packit GitHub app installed on the repository
  - the repo is published at `github.com/d-rk/omen-control`, and the COPR project is `d--rk/omen-control`
- Commits are `build:` and `ci:` (packaging is config and glue), checked by building and installing each package once.
