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

# Auto-detect cargo in user directory if invoked via sudo
if ! command -v cargo &>/dev/null; then
    if [ -n "$SUDO_USER" ] && [ -x "/home/$SUDO_USER/.cargo/bin/cargo" ]; then
        export PATH="/home/$SUDO_USER/.cargo/bin:$PATH"
    elif [ -x "$HOME/.cargo/bin/cargo" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    elif [ -x "$HOME/.cargo/env" ]; then
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
fi

echo "=== Building Lekhani (লেখনী) Pure Rust Input Method ==="
cargo build --workspace --release

echo "=== Installing Lekhani Core Binaries & Desktop GUI ==="
sudo install -Dm755 target/release/lekhani-gui /usr/bin/lekhani-gui
sudo install -Dm755 target/release/lekhani /usr/bin/lekhani
# Clean up or sync any legacy binaries in ~/.local/bin to prevent $PATH shadowing
rm -f "$HOME/.local/bin/lekhani-gui" "$HOME/.local/bin/lekhani" 2>/dev/null || true


echo "=== Installing Data Assets & Layouts ==="
sudo install -d /usr/share/lekhani/layouts
sudo install -Dm644 data/layouts/*.json /usr/share/lekhani/layouts/

sudo install -d /usr/share/lekhani/data
sudo install -Dm644 data/dictionaries/*.json /usr/share/lekhani/data/

echo "=== Installing Desktop Icons (All Resolutions + SVG) ==="
for size in 16 22 24 32 48 64 128 256 512 1024; do
    if [ -f "data/icons/${size}.png" ]; then
        sudo install -Dm644 "data/icons/${size}.png" "/usr/share/icons/hicolor/${size}x${size}/apps/lekhani.png"
        install -Dm644 "data/icons/${size}.png" "$HOME/.local/share/icons/hicolor/${size}x${size}/apps/lekhani.png"
    fi
done
if [ -f "data/icons/lekhani.svg" ]; then
    sudo install -Dm644 data/icons/lekhani.svg /usr/share/icons/hicolor/scalable/apps/lekhani.svg
    install -Dm644 data/icons/lekhani.svg "$HOME/.local/share/icons/hicolor/scalable/apps/lekhani.svg"
fi
sudo install -d /usr/share/lekhani/icons
sudo install -Dm644 data/icons/128.png /usr/share/lekhani/icons/lekhani.png

echo "=== Updating Desktop Icon Caches ==="
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
kbuildsycoca6 --noincremental 2>/dev/null || kbuildsycoca5 --noincremental 2>/dev/null || true

echo "=== Installing Desktop Entry & AppStream Metadata ==="
sudo install -Dm644 data/io.github.lekhani.keyboard.desktop /usr/share/applications/io.github.lekhani.keyboard.desktop
install -Dm644 data/io.github.lekhani.keyboard.desktop "$HOME/.local/share/applications/io.github.lekhani.keyboard.desktop"
sudo install -Dm644 data/io.github.lekhani.keyboard.metainfo.xml /usr/share/metainfo/io.github.lekhani.keyboard.metainfo.xml

if [ "$CHOICE" = "--fcitx5" ] || [ "$CHOICE" = "fcitx5" ] || [ "$CHOICE" = "--all" ] || [ "$CHOICE" = "all" ]; then
    echo "=== Installing Fcitx5 Engine (KDE Plasma 6 / Wayland) ==="
    
    # Check if Fcitx5 development headers are available for compiling native shared library plugin
    if [ -d "/usr/include/Fcitx5" ] || pkg-config --exists fcitx5 2>/dev/null; then
        echo "--> Building native Fcitx5 shared library plugin (C++ / Rust FFI)..."
        cmake -B crates/lekhani-fcitx5/build -S crates/lekhani-fcitx5 -DCMAKE_BUILD_TYPE=Release
        cmake --build crates/lekhani-fcitx5/build --config Release
        
        FCITX_LIB_DIR="/usr/lib64/fcitx5"
        if [ ! -d "/usr/lib64/fcitx5" ]; then
            if [ -d "/usr/lib/x86_64-linux-gnu/fcitx5" ]; then
                FCITX_LIB_DIR="/usr/lib/x86_64-linux-gnu/fcitx5"
            else
                FCITX_LIB_DIR="/usr/lib/fcitx5"
            fi
        fi
        sudo install -d "$FCITX_LIB_DIR"
        sudo install -Dm755 crates/lekhani-fcitx5/build/fcitx5-lekhani.so "$FCITX_LIB_DIR/fcitx5-lekhani.so"
        echo "--> Installed fcitx5-lekhani.so to $FCITX_LIB_DIR/"
    else
        echo "========================================================================="
        echo "NOTE: Fcitx5 C++ headers not detected (/usr/include/Fcitx5)."
        echo "To build the native Fcitx5 plugin, install the development package:"
        echo "  Fedora / RHEL : sudo dnf install -y fcitx5-devel"
        echo "  Ubuntu / Debian: sudo apt install -y libfcitx5core-dev"
        echo "  Arch Linux    : sudo pacman -S fcitx5"
        echo "========================================================================="
    fi

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
