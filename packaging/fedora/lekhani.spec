Name:           lekhani
Version:        3.0.0
Release:        1%{?dist}
Summary:        Modern pure Rust Bengali input method and desktop suite

License:        GPL-3.0-or-later
URL:            https://github.com/OpenBangla/lekhani
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  rust-packaging >= 21
BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  systemd-rpm-macros

Requires:       lekhani-common = %{version}-%{release}
Recommends:     ibus-lekhani = %{version}-%{release}
Suggests:       fcitx5-lekhani = %{version}-%{release}

%description
Lekhani is a modern, ultra-fast, 100% pure Rust Unicode Bengali input method framework for Linux.
This metapackage installs the complete Lekhani suite.

%package common
Summary:        Shared layouts, dictionaries, GUI, and CLI for Lekhani
Requires:       hicolor-icon-theme
Recommends:     google-noto-sans-bengali-fonts
Recommends:     google-noto-serif-bengali-fonts

%description common
Shared layouts, dictionaries, desktop tools, Slint TopBar GUI, and CLI utility for Lekhani.

%package -n ibus-lekhani
Summary:        IBus engine for Lekhani (Bengali input method for GNOME / IBus)
Requires:       lekhani-common = %{version}-%{release}
Requires:       ibus

%description -n ibus-lekhani
Lekhani Bengali typing engine for IBus (recommended for GNOME, Ubuntu, and Fedora Workstation).

%package -n fcitx5-lekhani
Summary:        Fcitx5 engine for Lekhani (Bengali input method for KDE Plasma 6 / Wayland)
Requires:       lekhani-common = %{version}-%{release}
Requires:       fcitx5

%description -n fcitx5-lekhani
Lekhani Bengali typing engine for Fcitx5 (recommended for KDE Plasma 6, Wayland, and Fedora KDE spin).

%prep
%autosetup -p1
%cargo_prep

%generate_buildrequires
%cargo_generate_buildrequires

%build
%cargo_build

%install
# Install binaries
install -Dpm 0755 target/release/lekhani-gui %{buildroot}%{_bindir}/lekhani-gui
install -Dpm 0755 target/release/ibus-lekhani %{buildroot}%{_bindir}/ibus-lekhani
install -Dpm 0755 target/release/fcitx5-lekhani %{buildroot}%{_bindir}/fcitx5-lekhani
install -Dpm 0755 target/release/lekhani %{buildroot}%{_bindir}/lekhani

# Install data assets & layouts
install -d %{buildroot}%{_datadir}/lekhani/layouts
install -Dpm 0644 data/layouts/*.json %{buildroot}%{_datadir}/lekhani/layouts/

install -d %{buildroot}%{_datadir}/lekhani/data
install -Dpm 0644 data/dictionaries/*.json %{buildroot}%{_datadir}/lekhani/data/

# Install icons
install -d %{buildroot}%{_datadir}/icons/hicolor/scalable/apps
install -Dpm 0644 data/icons/lekhani.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/lekhani.svg
for size in 16 22 24 32 48 64 128 256 512 1024; do
    if [ -f "data/icons/${size}.png" ]; then
        install -Dpm 0644 "data/icons/${size}.png" "%{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/lekhani.png"
    fi
done
install -d %{buildroot}%{_datadir}/lekhani/icons
install -Dpm 0644 data/icons/128.png %{buildroot}%{_datadir}/lekhani/icons/lekhani.png

# Install IBus & Fcitx5 configurations
install -Dpm 0644 data/ibus/lekhani.xml %{buildroot}%{_datadir}/ibus/component/lekhani.xml
install -Dpm 0644 data/fcitx5/addon/lekhani.conf %{buildroot}%{_datadir}/fcitx5/addon/lekhani.conf
install -Dpm 0644 data/fcitx5/inputmethod/lekhani.conf %{buildroot}%{_datadir}/fcitx5/inputmethod/lekhani.conf

# Install Desktop Entry & Metainfo
install -Dpm 0644 data/io.github.lekhani.keyboard.desktop %{buildroot}%{_datadir}/applications/io.github.lekhani.keyboard.desktop
install -Dpm 0644 data/io.github.lekhani.keyboard.metainfo.xml %{buildroot}%{_metainfodir}/io.github.lekhani.keyboard.metainfo.xml

%check
%cargo_test

%files
# Meta-package

%files common
%license LICENSE
%doc README.md
%{_bindir}/lekhani-gui
%{_bindir}/lekhani
%{_datadir}/lekhani/
%{_datadir}/applications/io.github.lekhani.keyboard.desktop
%{_metainfodir}/io.github.lekhani.keyboard.metainfo.xml
%{_datadir}/icons/hicolor/*/apps/lekhani.png

%files -n ibus-lekhani
%{_bindir}/ibus-lekhani
%{_datadir}/ibus/component/lekhani.xml

%files -n fcitx5-lekhani
%{_libdir}/fcitx5/fcitx5-lekhani.so
%{_datadir}/fcitx5/addon/lekhani.conf
%{_datadir}/fcitx5/inputmethod/lekhani.conf

%changelog
* Fri Sep 11 2026 Lekhani Contributors <openbanglateam@gmail.com> - 3.0.0-1
- Initial release with independent ibus-lekhani and fcitx5-lekhani packages
