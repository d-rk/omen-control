# Design

## Context

The crate builds a single binary. `ldd target/release/omen-control` shows that it links only to libc, libm and
libgcc_s. winit and wgpu open Wayland, xkbcommon, Vulkan and EGL with `dlopen` at runtime. As a result, package tools
that detect dependencies from linked libraries (cargo-deb `$auto`, rpm's automatic Requires) find nothing useful, and
the dependencies have to be declared by hand.

The desktop entry already uses `Exec=omen-control` and `Icon=omen-control`, so the files in `data/` can be installed
unchanged.

The repository is `github.com/d-rk/omen-control`, with the Packit GitHub app installed. The COPR project is
`d--rk/omen-control`. It has internet access during builds enabled and allows Packit builds from the GitHub repo.
`Cargo.toml` still names `github.com/OmenLinux/omen-control` as the repository.

## Goals / Non-Goals

**Goals:**
- A tag push is the only manual step of a release.
- One rpm definition (the spec file), used both by COPR and by the release workflow.

**Non-Goals:**
- **Old distributions.** Supported targets are current Fedora releases (whatever COPR's `fedora-all` covers) and
  current Debian stable and Ubuntu LTS. openSUSE and older releases are not.
- **X11 sessions.** The packages declare only the Wayland dependencies. The binary still contains winit's X11 code,
  because disabling it isn't worth a custom iced feature set.
- **Meeting the Fedora package review guidelines.** A COPR repository doesn't need them.
- **Signing the GitHub release assets.** COPR signs its own repository.
- **aarch64 builds.** HP OMEN laptops are x86_64.

## Decisions

**One spec file builds every rpm. cargo-generate-rpm is not used.**
- COPR needs a `.spec` file anyway.
- The release workflow builds its rpm from the same spec with `rpmbuild` in a `fedora:latest` container.
- That ties the rpm to current Fedora's glibc and package names, which is all that's supported.
- Alternative: cargo-generate-rpm on the Ubuntu runner. It would give a more portable rpm, but needs a second file list
  to keep in sync, and the portability is only needed for distributions that are out of scope.

**Dependencies are declared by hand with each distribution's package names.**
- rpm: `Requires: libxkbcommon libwayland-client`, `Recommends: vulkan-loader mesa-libEGL`.
- deb: `Depends: libxkbcommon0, libwayland-client0`, `Recommends: libvulkan1, libegl1`.
- GPU libraries are only recommended, because without them iced falls back to software rendering.

**The COPR build has network access enabled. Crates are not vendored.**
- The spec runs `cargo build --release --locked`, with the toolchain from Fedora's `cargo` and `rust` packages.
- The COPR project setting "enable internet access during builds" is turned on, so cargo can download the crates
  pinned in `Cargo.lock`. The release workflow's Fedora container has network access anyway.
- Alternative: run `cargo vendor` in Packit's `create-archive` step and build offline. That is more robust, but it
  means a custom archive script. If COPR network access ever becomes a problem, switch to this.
- `rust-version = "1.88"` is older than the Rust in every supported Fedora release, so the system toolchain is enough.

**Packit drives COPR, and is triggered by GitHub releases.**
- `.packit.yaml` defines a `copr_build` job with `trigger: release`, `owner`/`project` pointing at the maintainer's
  COPR project, and targets `fedora-all-x86_64`. That is a Packit alias for every active Fedora release plus rawhide.
  The COPR project has the matching `fedora-NN-x86_64` releases and `fedora-rawhide-x86_64` enabled. With "follow
  Fedora branching" on, COPR adds each new Fedora release from rawhide, so the two stay in step without manual edits.
- Packit builds the source RPM from the tag. `upstream_tag_template` is `v{version}`.
- Alternative: a COPR SCM webhook. It is simpler, but it rebuilds on every push unless it is filtered, and it doesn't
  report back to GitHub.

**The release workflow runs on `v*` tags and has three jobs.**
- `check`: the tag matches the `Cargo.toml` version.
- `deb`, on `ubuntu-24.04`, after `check`:
  - `cargo build --release --locked`
  - `cargo deb --no-build`
  - the tarball `omen-control-X.Y.Z-x86_64-linux.tar.gz`
- `rpm`, in a `fedora:latest` container, after `check`: `rpmbuild -ba` with the spec file on a `git archive` tarball.
- `release`, after `deb` and `rpm`: downloads both jobs' outputs and runs `gh release create` with all assets.
- The release is created last, so a failure anywhere before it publishes nothing.
- Creating the release triggers Packit's `release` job, so COPR builds the same tag.
- Ubuntu 24.04 sets the minimum glibc for the deb and tarball at 2.39. Current Debian stable and Ubuntu LTS meet it.

## Risks / Trade-offs

- [A recommended library turns out to be needed to open a window] → Install each package once in a clean container or
  VM, starting from a minimal image, before the first release (see tasks). Move a library from Recommends to Requires
  if it turns out to be needed.
- [Fedora's Rust is older than a future `rust-version` bump] → COPR and the `rpm` job fail visibly. Keep
  `rust-version` within what the oldest supported Fedora ships.
- [Packit and COPR are not set up when the first tag is pushed] → The GitHub release still succeeds, because the
  workflow doesn't depend on COPR. Start the first COPR build by hand from the Packit dashboard.

## Migration Plan

Nothing to migrate. Users who installed with `scripts/install-desktop-entry.sh` should run it with `--uninstall`
before installing the package. Otherwise their per-user entry in `~/.local/share/applications` takes precedence over
the system one and keeps pointing at the old binary. The README says this.

