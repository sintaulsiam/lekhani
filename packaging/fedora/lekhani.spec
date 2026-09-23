Name:           lekhani
Version:        1.0.0
Release:        1%{?dist}
Summary:        Modern pure Rust Bengali input method and desktop suite
License:        GPL-3.0-or-later
URL:            https://github.com/sintaulsiam/lekhani
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  cmake
BuildRequires:  gcc-c++
BuildRequires:  extra-cmake-modules
BuildRequires:  fcitx5-devel
BuildRequires:  ibus-devel
BuildRequires:  fontconfig-devel
BuildRequires:  libxkbcommon-devel

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
%autosetup -n %{name}-%{version}

%build
cargo build --release --workspace
cmake -B crates/lekhani-fcitx5/build -S crates/lekhani-fcitx5 -DCMAKE_BUILD_TYPE=Release
cmake --build crates/lekhani-fcitx5/build --config Release

%install
# Install binaries
install -Dpm 0755 target/release/lekhani-gui %{buildroot}%{_bindir}/lekhani-gui
install -Dpm 0755 target/release/ibus-lekhani %{buildroot}%{_bindir}/ibus-lekhani
install -Dpm 0755 target/release/lekhani %{buildroot}%{_bindir}/lekhani

# Install Fcitx5 shared library plugin
install -d %{buildroot}%{_libdir}/fcitx5
install -Dpm 0755 crates/lekhani-fcitx5/build/fcitx5-lekhani.so %{buildroot}%{_libdir}/fcitx5/fcitx5-lekhani.so

# Install data assets & layouts
install -d %{buildroot}%{_datadir}/lekhani/layouts
install -Dpm 0644 data/layouts/*.json %{buildroot}%{_datadir}/lekhani/layouts/

install -d %{buildroot}%{_datadir}/lekhani/data
install -Dpm 0644 data/dictionaries/*.json %{buildroot}%{_datadir}/lekhani/data/
install -Dpm 0644 data/dictionaries/*.bin %{buildroot}%{_datadir}/lekhani/data/ 2>/dev/null || true

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
install -d %{buildroot}%{_datadir}/ibus/component
install -Dpm 0644 data/ibus/lekhani.xml %{buildroot}%{_datadir}/ibus/component/lekhani.xml
install -d %{buildroot}%{_datadir}/fcitx5/addon
install -Dpm 0644 data/fcitx5/addon/lekhani.conf %{buildroot}%{_datadir}/fcitx5/addon/lekhani.conf
install -d %{buildroot}%{_datadir}/fcitx5/inputmethod
install -Dpm 0644 data/fcitx5/inputmethod/lekhani.conf %{buildroot}%{_datadir}/fcitx5/inputmethod/lekhani.conf

# Install Desktop Entry & Metainfo
install -d %{buildroot}%{_datadir}/applications
install -Dpm 0644 data/io.github.lekhani.keyboard.desktop %{buildroot}%{_datadir}/applications/io.github.lekhani.keyboard.desktop
install -d %{buildroot}%{_metainfodir}
install -Dpm 0644 data/io.github.lekhani.keyboard.metainfo.xml %{buildroot}%{_metainfodir}/io.github.lekhani.keyboard.metainfo.xml

# Install Systemd user service units
install -d %{buildroot}%{_userunitdir}
install -Dpm 0644 data/systemd/lekhani-gui.service %{buildroot}%{_userunitdir}/lekhani-gui.service
install -Dpm 0644 data/systemd/ibus-lekhani.service %{buildroot}%{_userunitdir}/ibus-lekhani.service

%check
cargo test --workspace

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
%{_userunitdir}/lekhani-gui.service
%{_datadir}/icons/hicolor/*/apps/lekhani.png
%{_datadir}/icons/hicolor/scalable/apps/lekhani.svg

%files -n ibus-lekhani
%{_bindir}/ibus-lekhani
%{_datadir}/ibus/component/lekhani.xml
%{_userunitdir}/ibus-lekhani.service

%files -n fcitx5-lekhani
%{_libdir}/fcitx5/fcitx5-lekhani.so
%{_datadir}/fcitx5/addon/lekhani.conf
%{_datadir}/fcitx5/inputmethod/lekhani.conf

%changelog
* Mon Sep 14 2026 Sintaul Mahdi Siam (Syntenieum) <sintaulsiam@gmail.com> - 1.0.0-1
- Lekhani 1.0.0 official release by Syntenieum (2026) with pure Rust core, regression-proof phonetic engine, native Fcitx5 and IBus engines
- Special thanks to Avro Keyboard (OmicronLab) and OpenBangla Keyboard
