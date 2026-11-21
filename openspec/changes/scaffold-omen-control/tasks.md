# Tasks

## 1. Project setup

- [ ] 1.1 Create the Cargo crate, dependencies, `rustfmt.toml`, `.gitignore` entries and GPL-3.0 LICENSE; verify `cargo build` succeeds
- [ ] 1.2 Add the check commands to CLAUDE.md; verify they run

## 2. sysfs backend

- [ ] 2.1 Value types with parse/format for colours, curves, gradient config, power limits, RPM and max RPM; verify the sysfs-backend value scenarios pass as unit tests
- [ ] 2.2 Paths (default, `OMEN_SYSFS_ROOT`), thermal zone lookup and platform profile; verify the root and temperature scenarios pass
- [ ] 2.3 Device read/write with optional files and file-naming errors; verify the optional-file scenarios pass
- [ ] 2.4 Mock tree; verify "Mock tree is readable as a driver" passes

## 3. Profiles

- [ ] 3.1 Profile types, name validation and TOML store; verify the profile storage and name scenarios pass
- [ ] 3.2 Applying a profile to the device; verify the applying scenarios pass

## 4. Driver detection

- [ ] 4.1 Driver presence check and Secure Boot parsing; verify the driver-detection scenarios pass
- [ ] 4.2 Driver-not-found screen with "Check again"; verify by running with `OMEN_SYSFS_ROOT=/nonexistent`

## 5. Lighting page

- [ ] 5.1 Zone hit testing, HSV conversion, hex input, preview colour functions; verify the keyboard-lighting logic scenarios pass
- [ ] 5.2 Lighting page state (pick colour, apply to all, brightness, mode, gradient editor validation, write errors); verify the page scenarios pass against the mock tree
- [ ] 5.3 Keyboard canvas, colour picker widget and page view; verify by running with `--mock`

## 6. Fans page

- [ ] 6.1 Curve editing functions (drag clamp, add, remove); verify the curve editor scenarios pass
- [ ] 6.2 Fans page state (profile, switches, apply/revert, power limits); verify the page scenarios pass against the mock tree
- [ ] 6.3 Curve editor canvas, live readings subscription and page view; verify by running with `--mock`

## 7. App shell and docs

- [ ] 7.1 App shell with theme, navigation and header profile selector; verify by running with `--mock`
- [ ] 7.2 README with driver dependency, build/run, `--mock` and the TODO list; verify the documented commands run
- [ ] 7.3 Verify `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass

## Workflow follow-up

- Commit test-first in separate commits once the user allows committing.
- Archive the change after review.
