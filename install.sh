#!/usr/bin/env bash
# Lekhani Local Build & Install Helper Script
# Usage:
#   ./install.sh --fcitx5   # Install for KDE Plasma 6 / Wayland / Hyprland
#   ./install.sh --ibus     # Install for GNOME / Ubuntu
#   ./install.sh --all      # Install both IBus and Fcitx5 engines

set -e

CHOICE="$1"

if [ -z "$CHOICE" ]; then
    echo "=========================================================="
    echo "       Lekhani (লেখনী) - Pure Rust Bengali Suite         "
    echo "=========================================================="
    echo "Choose which input method engine to install:"
    echo "  1) Fcitx5 only  (Recommended for KDE Plasma 6 / Wayland)"
    echo "  2) IBus only    (Recommended for GNOME / Ubuntu)"
    echo "  3) Both         (IBus & Fcitx5)"
    echo ""
    read -rp "Enter choice [1-3, default=1]: " USER_CHOICE
    case "$USER_CHOICE" in
        2) CHOICE="--ibus" ;;
        3) CHOICE="--all" ;;
        *) CHOICE="--fcitx5" ;;
    esac
fi

echo "=== Building Lekhani (লেখনী) Pure Rust Input Method ==="
cargo build --workspace --release

echo "=== Installing Lekhani Core Binaries & Desktop GUI ==="
sudo install -Dm755 target/release/lekhani-gui /usr/bin/lekhani-gui
sudo install -Dm755 target/release/lekhani /usr/bin/lekhani

echo "=== Installing Data Assets & Layouts ==="
sudo install -d /usr/share/lekhani/layouts
sudo install -Dm644 data/layouts/*.json /usr/share/lekhani/layouts/

sudo install -d /usr/share/lekhani/data
sudo install -Dm644 data/dictionaries/*.json /usr/share/lekhani/data/

echo "=== Installing Desktop Icons ==="
sudo install -Dm644 data/icons/16.png /usr/share/icons/hicolor/16x16/apps/lekhani.png
sudo install -Dm644 data/icons/32.png /usr/share/icons/hicolor/32x32/apps/lekhani.png
sudo install -Dm644 data/icons/48.png /usr/share/icons/hicolor/48x48/apps/lekhani.png
sudo install -Dm644 data/icons/128.png /usr/share/icons/hicolor/128x128/apps/lekhani.png
sudo install -Dm644 data/icons/512.png /usr/share/icons/hicolor/512x512/apps/lekhani.png
sudo install -Dm644 data/icons/1024.png /usr/share/icons/hicolor/1024x1024/apps/lekhani.png
sudo install -Dm644 data/icons/32.png /usr/share/lekhani/icons/lekhani.png

echo "=== Installing Desktop Entry & AppStream Metadata ==="
sudo install -Dm644 data/io.github.lekhani.keyboard.desktop /usr/share/applications/io.github.lekhani.keyboard.desktop
sudo install -Dm644 data/io.github.lekhani.keyboard.metainfo.xml /usr/share/metainfo/io.github.lekhani.keyboard.metainfo.xml

if [ "$CHOICE" = "--fcitx5" ] || [ "$CHOICE" = "fcitx5" ] || [ "$CHOICE" = "--all" ] || [ "$CHOICE" = "all" ]; then
    echo "=== Installing Fcitx5 Engine (KDE Plasma 6 / Wayland) ==="
    sudo install -Dm755 target/release/fcitx5-lekhani /usr/bin/fcitx5-lekhani
    sudo install -d /usr/share/fcitx5/addon /usr/share/fcitx5/inputmethod
    sudo install -Dm644 data/fcitx5/addon/lekhani.conf /usr/share/fcitx5/addon/lekhani.conf
    sudo install -Dm644 data/fcitx5/inputmethod/lekhani.conf /usr/share/fcitx5/inputmethod/lekhani.conf
fi

if [ "$CHOICE" = "--ibus" ] || [ "$CHOICE" = "ibus" ] || [ "$CHOICE" = "--all" ] || [ "$CHOICE" = "all" ]; then
    echo "=== Installing IBus Engine (GNOME / Ubuntu) ==="
    sudo install -Dm755 target/release/ibus-lekhani /usr/bin/ibus-lekhani
    sudo install -Dm644 data/ibus/lekhani.xml /usr/share/ibus/component/lekhani.xml
fi

echo ""
echo "=========================================================="
echo "    Installation Completed Successfully!                 "
echo "=========================================================="
if [ "$CHOICE" = "--fcitx5" ] || [ "$CHOICE" = "fcitx5" ]; then
    echo "To activate in KDE Plasma 6 / Fcitx5:"
    echo "  1. Run 'fcitx5 -r &' (or log out and back in)"
    echo "  2. Go to System Settings -> Input Devices -> Virtual Keyboard / Fcitx5"
    echo "  3. Add 'Bangla (Lekhani)' to your input methods"
elif [ "$CHOICE" = "--ibus" ] || [ "$CHOICE" = "ibus" ]; then
    echo "To activate in GNOME / IBus:"
    echo "  1. Run 'ibus restart'"
    echo "  2. Go to Settings -> Keyboard -> Input Sources"
    echo "  3. Add 'Bangla (Lekhani)' to your input methods"
else
    echo "Both IBus and Fcitx5 engines are installed."
fi
