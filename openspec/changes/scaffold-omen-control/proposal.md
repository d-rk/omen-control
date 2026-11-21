# Proposal

## Why

The omen-rgb-keyboard driver exposes keyboard lighting and fan control only as sysfs files, so users have to `echo`
into `/sys` by hand. omen-control gives them a desktop app for it. This first change picks the stack and lays down the
backend, the main pages and the repository skeleton, so later work (tray, packaging, custom effects) has a base.

## What Changes

- New Rust crate `omen-control` (single binary, GPL-3.0) built on iced 0.14.
- Typed sysfs client for the driver's `rgb_zones/` and `fan/` files, the thermal zones and the ACPI platform profile,
  under a configurable root (`OMEN_SYSFS_ROOT`) and a `--mock` mode that builds a fake tree in a temp dir.
- Lighting page: 4-zone keyboard preview with selection and approximate animation preview, colour picker, brightness,
  animation mode and speed, gradient editor.
- Fans page: live RPM and temperatures, thermal profile, max fan, curve switch, drag-to-edit CPU/GPU curves, power
  limits.
- Named profiles stored as TOML in the user's config directory, with a quick selector in the header.
- A "driver not found" screen with setup steps when the sysfs root is missing.
- Repository files: README, LICENSE, rustfmt config, `.gitignore` entries for Cargo; CLAUDE.md gets the check commands.

## Capabilities

### New Capabilities

- `sysfs-backend`: reading and writing the driver's sysfs values, value formats, optional files, root selection and the
  mock tree.
- `keyboard-lighting`: the lighting page behaviour (zone selection, colour, brightness, animation, gradient, preview).
- `fan-control`: the fans page behaviour (live readings, profiles, curve editing, power limits).
- `profiles`: saving, loading and deleting named settings profiles.
- `driver-detection`: detecting a missing driver and guiding the user through setup.

### Modified Capabilities

None.

## Impact

- New code under `src/`, `Cargo.toml`, `Cargo.lock`.
- New dependencies: iced, serde, toml, directories, thiserror, tempfile (mock mode and tests).
- No driver code is copied; the app is a userspace client of the driver's documented sysfs ABI and needs no root.
- Out of scope, kept as TODO in the README: tray icon via ksni, autostart entry, packaging, userspace custom effects.
