# omen-control

A desktop app for HP OMEN laptop keyboard lighting and fan control on Linux. It is a front end for the
[omen-rgb-keyboard](https://github.com/alessandromrc/omen-rgb-keyboard) kernel driver: it only reads and writes the
driver's sysfs files, needs no root, and ships as a single binary.

- **Lighting**: the four keyboard zones on a clickable keyboard preview, a colour picker (HSV, hex, swatches),
  brightness, brightness lock, the driver's twelve animation modes with a live preview, speed, and a gradient editor.
- **Fans**: live fan RPM and temperatures, thermal profile (silent / balanced / performance), max fan, custom curve
  switch, a drag-to-edit CPU and GPU fan curve editor, and CPU power limits (PL1/PL2). The ACPI platform profile is
  shown read-only.
- **Profiles**: save the current lighting (and optionally fan) settings under a name and switch between them from the
  header. Stored as TOML in `~/.config/omen-control/profiles/`.

## Requirements

The omen-rgb-keyboard driver must be installed and loaded. If it is missing, the app shows a setup screen instead.
In short:

1. Install the DKMS driver: `git clone https://github.com/alessandromrc/omen-rgb-keyboard.git && cd omen-rgb-keyboard && sudo ./install.sh`
2. Allow your user to write the controls without sudo: `sudo ./install-udev-rules.sh`, then log out and back in
   (this adds you to the `input` group).
3. Blacklist `hp_wmi`, which fights the driver over the same WMI interface:
   `echo "blacklist hp_wmi" | sudo tee /etc/modprobe.d/blacklist-hp.conf`
4. With Secure Boot enabled, sign the module with an enrolled MOK (`mokutil --sb-state` tells you whether it is on).

Controls whose sysfs file is missing on your model are hidden or disabled.

## Install

Packages are built for Wayland sessions on current Fedora, Debian stable and Ubuntu LTS (x86_64). They contain the
app only; install the driver as described above.

- **Fedora (COPR)**, updated with normal system updates:

  ```sh
  sudo dnf copr enable d--rk/omen-control
  sudo dnf install omen-control
  ```

- **Debian / Ubuntu**: download the `.deb` from the [latest release](https://github.com/d-rk/omen-control/releases/latest)
  and run `sudo apt install ./omen-control_*_amd64.deb`. The release also has an `.rpm` for Fedora without COPR.
- **Other distributions**: download the `omen-control-*-x86_64-linux.tar.gz` tarball, unpack it, and run
  `scripts/install-desktop-entry.sh` inside it to add the app to your menu.

If you installed a desktop entry for your user earlier (see below), remove it first with
`scripts/install-desktop-entry.sh --uninstall`. Otherwise it hides the packaged one and keeps starting the old binary.

## Build and run

Needs a recent stable Rust toolchain (1.88 or newer).

```sh
cargo build --release
./target/release/omen-control
```

Options:

- `--mock`: run against a fake driver tree in a temporary directory, with moving fan and temperature readings. Use it
  to try the app or develop it without the hardware.
- `OMEN_SYSFS_ROOT=/path`: read the driver from another directory instead of
  `/sys/devices/platform/omen-rgb-keyboard`.

### Desktop entry and icon

On Wayland the desktop finds the window's icon through a desktop entry named after the app id (`omen-control`).
Without one, the taskbar shows a generic icon. To install the entry and icon for your user, pointing at your build:

```sh
scripts/install-desktop-entry.sh                 # uses target/release/omen-control
scripts/install-desktop-entry.sh /path/to/omen-control
scripts/install-desktop-entry.sh --uninstall
```

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Specs and planned changes live in `openspec/`; see `CLAUDE.md` for the workflow.

### Releasing

1. Set the new version in `Cargo.toml` and in `Version:` of `omen-control.spec`, run `cargo build` to update
   `Cargo.lock`, and commit.
2. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`.

The release workflow checks that the tag matches both versions, then publishes the `.deb`, `.rpm` and tarball as a
GitHub release. Packit then builds the same tag in the [COPR project](https://copr.fedorainfracloud.org/coprs/d--rk/omen-control/).

## TODO

- Tray icon via `ksni` (quick profile switch, open window).
- Autostart `.desktop` entry that applies the last profile on login.
- Package the omen-rgb-keyboard driver (akmod in the COPR project), so the app package can depend on it.
- Userspace custom effects: colour from CPU temperature, audio-reactive lighting.
- Keyboard backlight on/off switch matching the Fn key toggle. The firmware switch is separate from
  `rgb_zones/brightness` and the driver doesn't expose it, so this needs a driver patch first: a `rgb_zones/backlight`
  file using `HPWMI_FOURZONE` commands 4 (read) and 5 (write) (`0xE4` on, `0x64` off in other OMEN tools; verify on
  this model). Then show the switch in the app only when the file exists.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
