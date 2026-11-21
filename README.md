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

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Specs and planned changes live in `openspec/`; see `CLAUDE.md` for the workflow.

## TODO

- Tray icon via `ksni` (quick profile switch, open window).
- Autostart `.desktop` entry that applies the last profile on login.
- Packaging: rpm (`cargo-generate-rpm`) and deb (`cargo-deb`) depending on an `omen-rgb-keyboard-dkms` package, plus an
  AppImage.
- Userspace custom effects: colour from CPU temperature, audio-reactive lighting.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
