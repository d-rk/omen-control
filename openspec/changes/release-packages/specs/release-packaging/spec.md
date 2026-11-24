# Spec Delta

## Purpose

Lets users install omen-control as a system package (deb, rpm, or from a Fedora COPR repository) or from a release
tarball, and lets maintainers publish all of these by pushing a version tag.

## ADDED Requirements

### Requirement: Package file layout
The deb and rpm packages SHALL install:

- the binary as `/usr/bin/omen-control`
- the desktop entry as `/usr/share/applications/omen-control.desktop`
- the icon as `/usr/share/icons/hicolor/scalable/apps/omen-control.svg`
- the licence and readme in the distribution's documentation and licence directories

#### Scenario: App listed after package install
- **WHEN** the user installs the deb or rpm package
- **THEN** OMEN Control appears in the desktop's application menu with its own icon, and launching it starts
  `/usr/bin/omen-control`

### Requirement: Runtime dependencies declared
The packages SHALL require the libraries the app loads at runtime on Wayland: Wayland client and xkbcommon. They SHALL
recommend, but not require, the GPU (Vulkan/EGL) libraries, because without them the app falls back to software
rendering. X11 sessions are not supported by the packages.

#### Scenario: Install on a minimal system
- **WHEN** the package is installed with the package manager on a system without libxkbcommon
- **THEN** the package manager installs libxkbcommon along with it, and the app starts in a Wayland session

### Requirement: Driver not bundled
The packages SHALL NOT install the kernel driver, udev rules or modprobe configuration, and SHALL NOT require a driver
package. Without the driver, the app shows its existing setup screen.

#### Scenario: Installed without the driver
- **WHEN** the package is installed on a machine where the omen-rgb-keyboard driver is not loaded
- **THEN** the package installs successfully and the app opens on the setup screen

### Requirement: Tagged release artifacts
Pushing a tag `vX.Y.Z` SHALL publish a GitHub release with three assets: a `.deb` for current Debian and Ubuntu, an
`.rpm` for current Fedora, and a tarball. The
tarball contains the binary, desktop entry, icon, desktop-entry install script, licence and readme. If the tag does
not match the crate version, the release SHALL fail and publish nothing.

#### Scenario: Release from tag
- **WHEN** a maintainer pushes tag `v0.2.0` and `Cargo.toml` has version `0.2.0`
- **THEN** the GitHub release `v0.2.0` has the `.deb`, `.rpm` and tarball for version 0.2.0 attached

#### Scenario: Tag does not match version
- **WHEN** a maintainer pushes tag `v0.2.0` and `Cargo.toml` has version `0.1.0`
- **THEN** the release workflow fails and attaches no assets

### Requirement: COPR repository updates
Publishing a GitHub release SHALL trigger a build of that version in the project's Fedora COPR repository for the
supported Fedora releases. Users who have enabled the repository then get the new version through normal system
updates.

#### Scenario: Update through dnf
- **WHEN** the user has run `dnf copr enable` for the project and installed omen-control, and a newer release is
  published
- **THEN** after the COPR build finishes, `dnf upgrade` installs the newer version
