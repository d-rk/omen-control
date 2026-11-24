Name:           omen-control
Version:        0.1.0
Release:        1%{?dist}
Summary:        Keyboard lighting and fan control for HP OMEN laptops

License:        GPL-3.0-or-later
URL:            https://github.com/d-rk/omen-control
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

ExclusiveArch:  x86_64

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  desktop-file-utils

# The binary links only libc; these are loaded with dlopen at runtime.
Requires:       libxkbcommon
Requires:       libwayland-client
Recommends:     vulkan-loader
Recommends:     mesa-libEGL

%global debug_package %{nil}

%description
Desktop UI for the omen-rgb-keyboard driver: keyboard lighting (four zones, animation modes, gradients) and fan
control (thermal profile, fan curves, power limits) for HP OMEN laptops. Needs the omen-rgb-keyboard kernel driver,
which is installed separately.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --locked

%install
install -Dpm 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -Dpm 0644 data/%{name}.desktop %{buildroot}%{_datadir}/applications/%{name}.desktop
install -Dpm 0644 data/%{name}.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/%{name}.svg

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/%{name}.desktop

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_datadir}/applications/%{name}.desktop
%{_datadir}/icons/hicolor/scalable/apps/%{name}.svg

%changelog
* Tue Nov 24 2026 Dirk Wilden <dirkwilden@gmail.com> - 0.1.0-1
- First package
