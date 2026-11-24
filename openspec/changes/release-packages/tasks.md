# Tasks

## 1. deb package

- [x] 1.1 Set `repository` in `Cargo.toml` to `https://github.com/d-rk/omen-control`. Add `[package.metadata.deb]`: maintainer, section, the three assets plus doc files, `Depends: libxkbcommon0, libwayland-client0` and `Recommends: libvulkan1, libegl1`. Verify `cargo deb` builds, `dpkg-deb -c` lists `/usr/bin/omen-control`, the desktop entry and the icon, and `dpkg-deb -I` shows the declared dependencies.

## 2. rpm spec and COPR

- [ ] 2.1 Write `omen-control.spec`: `BuildRequires: cargo rust desktop-file-utils`, `cargo build --release --locked`, install the three files plus `%license`/`%doc`, run `desktop-file-validate` in `%check`, `Requires: libxkbcommon libwayland-client` and `Recommends: vulkan-loader mesa-libEGL`. Verify that `rpmbuild -ba` on a `git archive` tarball succeeds, and that `rpm -qlp` and `rpm -qp --requires --recommends` show the expected paths and dependencies.
- [ ] 2.2 Install the built rpm on this machine (`sudo dnf install ./…rpm`). Verify OMEN Control appears in the application menu with its icon and starts (scenario "App listed after package install"). Then uninstall it.
- [ ] 2.3 Add `.packit.yaml`: `specfile_path`, `upstream_tag_template: v{version}`, and a `copr_build` job with `trigger: release`, `targets: fedora-all-x86_64`, and `owner: d--rk`, `project: omen-control`. Verify with `packit validate-config`.
- [ ] 2.4 Finish the one-time setup. Already done: the GitHub repo, the Packit app, and the COPR project with internet access and Packit allowed. Still to do: make sure the COPR project's chroots cover the Packit targets, and complete Packit's account verification if it asks for it. Verify that the first Packit-triggered build (see 5.1) produces an rpm in `d--rk/omen-control`.

## 3. Release workflow

- [ ] 3.1 Add `.github/workflows/release.yml`, triggered on `v*` tags, with these jobs:
  - `check`: the tag matches the crate version.
  - `deb`, on `ubuntu-24.04`: build the binary, the `.deb`, and the tarball (binary, `data/omen-control.desktop`, `data/omen-control.svg`, `scripts/install-desktop-entry.sh`, LICENSE, README).
  - `rpm`, in a `fedora:latest` container: `rpmbuild -ba` with the spec file.
  - `release`: run `gh release create` with all assets.

  Verify by running it against a test tag on a fork or with `act`, and check that a mismatched tag fails before any release is created (scenario "Tag does not match version").
- [ ] 3.2 Clean-install check. Install the release `.deb` in a fresh `debian:stable` container and the release `.rpm` in a fresh `fedora:latest` container, each with the package manager, and check that the declared dependencies are pulled in (scenario "Install on a minimal system"). Start the app on a Wayland session from the installed package, without the driver loaded, and check that it shows the setup screen (scenario "Installed without the driver"). Move any library that turns out to be needed from Recommends to Requires.

## 4. Docs

- [ ] 4.1 README: add an "Install" section covering COPR (`dnf copr enable …`), the `.deb`/`.rpm` from GitHub releases, and the tarball with the install script. Note that the packages target Wayland on current Fedora, Debian and Ubuntu. Add the note to run `scripts/install-desktop-entry.sh --uninstall` before switching to a package. Add a short "Releasing" note (bump the version, tag `vX.Y.Z`, push). Remove the packaging item from the TODO list. Verify the documented commands match the files produced in groups 1–3.

## 5. First release

- [ ] 5.1 Bump the version, push tag `v0.1.0` (or the next version), and verify the GitHub release has all three assets (scenario "Release from tag"). Verify that Packit starts a COPR build for the same version, and that `dnf copr enable` followed by `dnf install omen-control` works on this machine (scenario "Update through dnf", checked fully at the next release).

## Workflow follow-up

- Commits in this change are `build:` (Cargo metadata, spec, Packit), `ci:` (workflow) and `docs:` (README). Per CLAUDE.md, none need an `OpenSpec-Change` trailer.
- Archive the change after the first release is out and a COPR update has been seen through `dnf upgrade`.
