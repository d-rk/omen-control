# Design

## Context

Greenfield repository: only OpenSpec, the commit hook and CLAUDE.md exist. The driver's sysfs ABI is documented in the
omen-rgb-keyboard README; read formats not spelled out there (`#rrggbb` zones, `(unset)`, `(cpu)`, `n/a`, `unknown`,
`unsupported`) were checked against the driver's show functions. The driver is not loaded on the development machine,
so everything is developed against the mock tree.

## Goals / Non-Goals

**Goals:**
- One crate, one binary, no root needed at runtime.
- All decision logic (parsing, validation, curve editing, profile I/O, zone hit tests) testable without a window.

**Non-Goals:**
- Tray icon, autostart, packaging, userspace effects (TODO in README).
- Pixel-exact animation preview.
- Async I/O: sysfs reads and writes are small and local.

## Decisions

**Single crate with modules** rather than a workspace: `sysfs` (paths, value types, parse/format, device read/write,
mock tree), `profiles` (TOML store), `ui` (`theme`, `app` shell, `lighting`, `fans`, `driver_missing`, and widgets
`keyboard`, `color_picker`, `curve_editor`). A workspace adds nothing until a second binary (tray daemon) exists.

**iced 0.14** with the `canvas`, `tokio` and `advanced` features. The keyboard, the colour picker and the curve editor
are `canvas::Program`s. iced_aw's colour picker is a modal overlay, not an inline HSV square, so a custom canvas widget
is used instead.

**Paths struct.** `Paths { driver, thermal, platform_profile }`. Default: `/sys/devices/platform/omen-rgb-keyboard`,
`/sys/class/thermal`, `/sys/firmware/acpi/platform_profile`. `OMEN_SYSFS_ROOT` overrides only `driver`. `--mock`
creates a `tempfile::TempDir` with `driver/`, `thermal/thermal_zoneN/{type,temp}` and `firmware/platform_profile`; the
`TempDir` is kept alive for the app's lifetime. In mock mode, RPM and temperature files are rewritten by the app's
1 s tick (small random-walk) so the live readings move.

**Optional values.** Reads return `Option<T>` per field (missing file → `None`, unparsable → `None` and a logged
warning); writes return `Result<(), Error>` with the file path in the error. Missing files drive disabled/hidden
controls.

**Pure state machines for pages.** Each page has a `State` with an `update(&mut self, msg, &Device)` returning nothing
the view must interpret; the iced `view` reads the state. Tests drive `update` against a mock tree.

**Curve editor logic** lives in plain functions over `Vec<CurvePoint>` (`drag_to`, `insert_at`, `remove`), with the
canvas only mapping pixels to temperature/percent. Chart range is 30-100 °C; points outside that range still read and
write correctly but are drawn clamped.

**Profiles** are `Profile { lighting: LightingSettings, fans: Option<FanSettings> }` serialized with serde/toml, one
file per profile named after the validated profile name, in `directories::ProjectDirs` config dir. The store takes its
directory as a parameter so tests use a temp dir.

**Theme.** A custom `iced::Theme` built from a dark palette (near-black background, OMEN-like red accent), plus
container styles for cards.

## Risks / Trade-offs

- [iced API churn between releases] → pin `iced = "0.14"` and follow the 0.14 examples.
- [Synchronous WMI-backed writes could block the UI briefly] → acceptable for a first pass; move to `Task::perform`
  if measurements show stalls.
- [Animation preview differs from hardware] → labelled as a preview; spec requires only an approximation.
- [Untestable without hardware] → mock tree mirrors the driver's read formats checked against its source.
