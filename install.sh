#!/usr/bin/env bash
# Lekhani Local Build & Install / Uninstall Helper Script
# Usage:
#   ./install.sh --fcitx5     # Install for KDE Plasma 6 / Wayland / Hyprland
#   ./install.sh --ibus       # Install for GNOME / Ubuntu
#   ./install.sh --all        # Install both IBus and Fcitx5 engines
#   ./install.sh --uninstall  # Uninstall Lekhani from system
#   ./install.sh --help       # Display help information

set -e

CHOICE="$1"

ACTUAL_USER="${SUDO_USER:-$USER}"
ACTUAL_HOME=$(getent passwd "$ACTUAL_USER" 2>/dev/null | cut -d: -f6)
ACTUAL_HOME="${ACTUAL_HOME:-$HOME}"

install_user_file() {
    local mode="$1"
    local src="$2"
    local dest="$3"
    install -Dm"$mode" "$src" "$dest"
    if [ -n "$SUDO_USER" ] && [ "$(id -u)" -eq 0 ]; then
        chown "$ACTUAL_USER:" "$dest" 2>/dev/null || true
    fi
}

install_user_dir() {
    local dir="$1"
    install -d "$dir"
    if [ -n "$SUDO_USER" ] && [ "$(id -u)" -eq 0 ]; then
        chown "$ACTUAL_USER:" "$dir" 2>/dev/null || true
    fi
}

clean_user_stale_binaries() {
    echo "--> Cleaning up stale user binaries to prevent PATH shadowing..."
    local dirs=("$ACTUAL_HOME/.cargo/bin" "$ACTUAL_HOME/.local/bin")
    if [ -n "$HOME" ] && [ "$HOME" != "$ACTUAL_HOME" ]; then
        dirs+=("$HOME/.cargo/bin" "$HOME/.local/bin")
    fi
    for d in "${dirs[@]}"; do
        if [ -d "$d" ]; then
            rm -f "$d/lekhani-gui" "$d/lekhani" "$d/ibus-lekhani" 2>/dev/null || true
        fi
    done
}

uninstall_lekhani() {
    echo "=========================================================="
    echo "       Uninstalling Lekhani (লেখনী) from System           "
    echo "=========================================================="
    
    echo "--> Terminating running Lekhani processes..."
    pkill -f lekhani-gui 2>/dev/null || true
    pkill -f ibus-lekhani 2>/dev/null || true
    pkill -x lekhani 2>/dev/null || true

    echo "--> Removing core binaries..."
    sudo rm -f /usr/bin/lekhani-gui /usr/bin/lekhani /usr/bin/ibus-lekhani
    clean_user_stale_binaries

    echo "--> Removing system shared data assets and layouts..."
    sudo rm -rf /usr/share/lekhani

    echo "--> Removing desktop applications and AppStream metadata..."
    sudo rm -f /usr/share/applications/io.github.lekhani.keyboard.desktop
    sudo rm -f /usr/share/metainfo/io.github.lekhani.keyboard.metainfo.xml
    rm -f "$ACTUAL_HOME/.local/share/applications/io.github.lekhani.keyboard.desktop" 2>/dev/null || true
    rm -f "$ACTUAL_HOME/.local/share/metainfo/io.github.lekhani.keyboard.metainfo.xml" 2>/dev/null || true
    if [ "$HOME" != "$ACTUAL_HOME" ]; then
        rm -f "$HOME/.local/share/applications/io.github.lekhani.keyboard.desktop" 2>/dev/null || true
        rm -f "$HOME/.local/share/metainfo/io.github.lekhani.keyboard.metainfo.xml" 2>/dev/null || true
    fi

    echo "--> Removing icons..."
    for size in 16 22 24 32 48 64 128 256 512 1024; do
        sudo rm -f "/usr/share/icons/hicolor/${size}x${size}/apps/lekhani.png"
        rm -f "$ACTUAL_HOME/.local/share/icons/hicolor/${size}x${size}/apps/lekhani.png" 2>/dev/null || true
        [ "$HOME" != "$ACTUAL_HOME" ] && rm -f "$HOME/.local/share/icons/hicolor/${size}x${size}/apps/lekhani.png" 2>/dev/null || true
    done
    sudo rm -f /usr/share/icons/hicolor/scalable/apps/lekhani.svg
    rm -f "$ACTUAL_HOME/.local/share/icons/hicolor/scalable/apps/lekhani.svg" 2>/dev/null || true
    [ "$HOME" != "$ACTUAL_HOME" ] && rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/lekhani.svg" 2>/dev/null || true

    echo "--> Removing Fcitx5 addons, inputmethod configs, and plugins..."
    sudo rm -f /usr/share/fcitx5/addon/lekhani.conf
    sudo rm -f /usr/share/fcitx5/inputmethod/lekhani.conf
    sudo rm -f /usr/lib64/fcitx5/fcitx5-lekhani.so
    sudo rm -f /usr/lib/x86_64-linux-gnu/fcitx5/fcitx5-lekhani.so
    sudo rm -f /usr/lib/fcitx5/fcitx5-lekhani.so
    rm -f "$ACTUAL_HOME/.local/share/fcitx5/addon/lekhani.conf" 2>/dev/null || true
    rm -f "$ACTUAL_HOME/.local/share/fcitx5/inputmethod/lekhani.conf" 2>/dev/null || true
    rm -f "$ACTUAL_HOME/.local/lib/fcitx5/fcitx5-lekhani.so" 2>/dev/null || true
    if [ "$HOME" != "$ACTUAL_HOME" ]; then
        rm -f "$HOME/.local/share/fcitx5/addon/lekhani.conf" 2>/dev/null || true
        rm -f "$HOME/.local/share/fcitx5/inputmethod/lekhani.conf" 2>/dev/null || true
        rm -f "$HOME/.local/lib/fcitx5/fcitx5-lekhani.so" 2>/dev/null || true
    fi

    echo "--> Removing systemd user services..."
    systemctl --user stop lekhani-gui.service ibus-lekhani.service 2>/dev/null || true
    systemctl --user disable lekhani-gui.service ibus-lekhani.service 2>/dev/null || true
    sudo rm -f /usr/lib/systemd/user/lekhani-gui.service /usr/lib/systemd/user/ibus-lekhani.service
    rm -f "$ACTUAL_HOME/.config/systemd/user/lekhani-gui.service" "$ACTUAL_HOME/.config/systemd/user/ibus-lekhani.service" 2>/dev/null || true
    rm -f "$ACTUAL_HOME/.local/share/systemd/user/lekhani-gui.service" "$ACTUAL_HOME/.local/share/systemd/user/ibus-lekhani.service" 2>/dev/null || true
    if [ "$HOME" != "$ACTUAL_HOME" ]; then
        rm -f "$HOME/.config/systemd/user/lekhani-gui.service" "$HOME/.config/systemd/user/ibus-lekhani.service" 2>/dev/null || true
        rm -f "$HOME/.local/share/systemd/user/lekhani-gui.service" "$HOME/.local/share/systemd/user/ibus-lekhani.service" 2>/dev/null || true
    fi

    echo "--> Removing IBus component XML..."
    sudo rm -f /usr/share/ibus/component/lekhani.xml
    rm -f "$ACTUAL_HOME/.local/share/ibus/component/lekhani.xml" 2>/dev/null || true
    [ "$HOME" != "$ACTUAL_HOME" ] && rm -f "$HOME/.local/share/ibus/component/lekhani.xml" 2>/dev/null || true

    echo "--> Updating desktop and icon caches..."
    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    gtk-update-icon-cache -f -t "$ACTUAL_HOME/.local/share/icons/hicolor" 2>/dev/null || true
    sudo update-desktop-database -q /usr/share/applications 2>/dev/null || true
    update-desktop-database -q "$ACTUAL_HOME/.local/share/applications" 2>/dev/null || true
    kbuildsycoca6 --noincremental 2>/dev/null || kbuildsycoca5 --noincremental 2>/dev/null || true

    if [ -d "$ACTUAL_HOME/.config/lekhani" ] || [ -d "$ACTUAL_HOME/.local/share/lekhani" ]; then
        echo ""
        if [ -t 0 ]; then
            read -rp "Do you also want to delete user configurations, stats & learning history (~/.config/lekhani & ~/.local/share/lekhani)? [y/N]: " REMOVE_CONFIG
            case "$REMOVE_CONFIG" in
                [yY]|[yY][eE][sS])
                    rm -rf "$ACTUAL_HOME/.config/lekhani"
                    rm -rf "$ACTUAL_HOME/.local/share/lekhani"
                    echo "--> User configurations and statistics directories removed."
                    ;;
                *)
                    echo "--> Preserved user configurations (~/.config/lekhani) and typing statistics / learning history (~/.local/share/lekhani)."
                    ;;
            esac
        else
            echo "--> Preserved user configurations and typing statistics."
        fi
    fi

    echo ""
    echo "=========================================================="
    echo "       Lekhani has been uninstalled successfully!         "
    echo "=========================================================="
    echo "Please restart your input method framework if it was active:"
    echo "  - For Fcitx5 : fcitx5 -r &"
    echo "  - For IBus   : ibus restart"
    exit 0
}

if [ "$CHOICE" = "--help" ] || [ "$CHOICE" = "-h" ] || [ "$CHOICE" = "help" ]; then
    echo "Lekhani Build, Install & Uninstall Helper"
    echo ""
    echo "Usage:"
    echo "  ./install.sh [OPTION]"
    echo ""
    echo "Options:"
    echo "  --fcitx5, fcitx5       Build and install Lekhani Fcitx5 addon (KDE / Wayland)"
    echo "  --ibus, ibus           Build and install Lekhani IBus engine (GNOME / Ubuntu)"
    echo "  --all, all             Build and install both Fcitx5 and IBus engines"
    echo "  --uninstall, -u        Completely remove Lekhani from the system"
    echo "  --help, -h             Display this help message"
    echo ""
    echo "Running without arguments will launch the interactive menu."
    exit 0
fi

if [ "$CHOICE" = "--uninstall" ] || [ "$CHOICE" = "-u" ] || [ "$CHOICE" = "uninstall" ]; then
    uninstall_lekhani
fi

if [ -z "$CHOICE" ]; then
    echo "=========================================================="
    echo "       Lekhani (লেখনী) - Pure Rust Bengali Suite         "
    echo "=========================================================="
    echo "Choose an action:"
    echo "  1) Fcitx5 only   (Recommended for KDE Plasma 6 / Wayland)"
    echo "  2) IBus only     (Recommended for GNOME / Ubuntu)"
    echo "  3) Both          (IBus & Fcitx5)"
    echo "  4) Uninstall     (Remove Lekhani from system)"
    echo ""
    read -rp "Enter choice [1-4, default=1]: " USER_CHOICE
    case "$USER_CHOICE" in
        2) CHOICE="--ibus" ;;
        3) CHOICE="--all" ;;
        4) uninstall_lekhani ;;
        *) CHOICE="--fcitx5" ;;
    esac
fi

# Terminate running instances before updating binaries to avoid running stale memory images
echo "--> Terminating running Lekhani processes before update..."
pkill -f lekhani-gui 2>/dev/null || true
pkill -f ibus-lekhani 2>/dev/null || true
pkill -x lekhani 2>/dev/null || true

# Check for pre-compiled binaries (e.g. inside portable release tarball in bin/)
if [ -f "bin/lekhani-gui" ] && [ -f "bin/lekhani" ] && [ -f "bin/ibus-lekhani" ]; then
    echo "=== Using Pre-Compiled Lekhani Binaries from bin/ ==="
    GUI_BIN="bin/lekhani-gui"
    CLI_BIN="bin/lekhani"
    IBUS_BIN="bin/ibus-lekhani"
else
    # Auto-detect cargo in user directory if invoked via sudo
    if ! command -v cargo &>/dev/null; then
        if [ -n "$ACTUAL_USER" ] && [ -x "/home/$ACTUAL_USER/.cargo/bin/cargo" ]; then
            export PATH="/home/$ACTUAL_USER/.cargo/bin:$PATH"
        elif [ -x "$HOME/.cargo/bin/cargo" ]; then
            export PATH="$HOME/.cargo/bin:$PATH"
        elif [ -x "$HOME/.cargo/env" ]; then
            # shellcheck disable=SC1091
            source "$HOME/.cargo/env"
        fi
    fi

    echo "=== Building Lekhani (লেখনী) Pure Rust Input Method ==="
    cargo build --workspace --release
    GUI_BIN="target/release/lekhani-gui"
    CLI_BIN="target/release/lekhani"
    IBUS_BIN="target/release/ibus-lekhani"
fi

echo "=== Installing Lekhani Core Binaries & Desktop GUI ==="
sudo install -Dm755 "$GUI_BIN" /usr/bin/lekhani-gui
sudo install -Dm755 "$CLI_BIN" /usr/bin/lekhani
# Clean up any stale/legacy binaries in ~/.cargo/bin and ~/.local/bin to prevent $PATH shadowing
clean_user_stale_binaries

echo "=== Installing Data Assets & Layouts ==="
sudo install -d /usr/share/lekhani/layouts
sudo install -Dm644 data/layouts/*.json /usr/share/lekhani/layouts/

sudo install -d /usr/share/lekhani/data
sudo install -Dm644 data/dictionaries/*.json /usr/share/lekhani/data/

echo "=== Installing Desktop Icons (All Resolutions + SVG) ==="
for size in 16 22 24 32 48 64 128 256 512 1024; do
    if [ -f "data/icons/${size}.png" ]; then
        sudo install -Dm644 "data/icons/${size}.png" "/usr/share/icons/hicolor/${size}x${size}/apps/lekhani.png"
        if [ -d "$ACTUAL_HOME" ]; then
            install_user_file 644 "data/icons/${size}.png" "$ACTUAL_HOME/.local/share/icons/hicolor/${size}x${size}/apps/lekhani.png"
        fi
    fi
done
if [ -f "data/icons/lekhani.svg" ]; then
    sudo install -Dm644 data/icons/lekhani.svg /usr/share/icons/hicolor/scalable/apps/lekhani.svg
    if [ -d "$ACTUAL_HOME" ]; then
        install_user_file 644 data/icons/lekhani.svg "$ACTUAL_HOME/.local/share/icons/hicolor/scalable/apps/lekhani.svg"
    fi
fi
sudo install -d /usr/share/lekhani/icons
sudo install -Dm644 data/icons/128.png /usr/share/lekhani/icons/lekhani.png

echo "=== Installing Desktop Entry & AppStream Metadata ==="
sudo install -Dm644 data/io.github.lekhani.keyboard.desktop /usr/share/applications/io.github.lekhani.keyboard.desktop
if [ -d "$ACTUAL_HOME" ]; then
    install_user_file 644 data/io.github.lekhani.keyboard.desktop "$ACTUAL_HOME/.local/share/applications/io.github.lekhani.keyboard.desktop"
fi
sudo install -Dm644 data/io.github.lekhani.keyboard.metainfo.xml /usr/share/metainfo/io.github.lekhani.keyboard.metainfo.xml

echo "=== Updating Desktop Icon & Application Caches ==="
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
gtk-update-icon-cache -f -t "$ACTUAL_HOME/.local/share/icons/hicolor" 2>/dev/null || true
sudo update-desktop-database -q /usr/share/applications 2>/dev/null || true
update-desktop-database -q "$ACTUAL_HOME/.local/share/applications" 2>/dev/null || true
kbuildsycoca6 --noincremental 2>/dev/null || kbuildsycoca5 --noincremental 2>/dev/null || true

echo "=== Installing Systemd User Services ==="
if [ -d "data/systemd" ]; then
    sudo install -d /usr/lib/systemd/user
    sudo install -Dm644 data/systemd/*.service /usr/lib/systemd/user/ 2>/dev/null || true
    if [ -d "$ACTUAL_HOME" ]; then
        install_user_dir "$ACTUAL_HOME/.config/systemd/user"
        for svc in data/systemd/*.service; do
            [ -f "$svc" ] && install_user_file 644 "$svc" "$ACTUAL_HOME/.config/systemd/user/$(basename "$svc")"
        done
    fi
    systemctl --user daemon-reload 2>/dev/null || true
fi

if [ "$CHOICE" = "--fcitx5" ] || [ "$CHOICE" = "fcitx5" ] || [ "$CHOICE" = "--all" ] || [ "$CHOICE" = "all" ]; then
    echo "=== Installing Fcitx5 Engine (KDE Plasma 6 / Wayland) ==="
    
    FCITX_LIB_DIR="/usr/lib64/fcitx5"
    if [ ! -d "/usr/lib64/fcitx5" ]; then
        if [ -d "/usr/lib/x86_64-linux-gnu/fcitx5" ]; then
            FCITX_LIB_DIR="/usr/lib/x86_64-linux-gnu/fcitx5"
        else
            FCITX_LIB_DIR="/usr/lib/fcitx5"
        fi
    fi

    if [ -d "/usr/include/Fcitx5" ] || pkg-config --exists fcitx5 2>/dev/null; then
        echo "--> Building native Fcitx5 shared library plugin (C++ / Rust FFI)..."
        cmake -B crates/lekhani-fcitx5/build -S crates/lekhani-fcitx5 -DCMAKE_BUILD_TYPE=Release
        cmake --build crates/lekhani-fcitx5/build --config Release
        sudo install -d "$FCITX_LIB_DIR"
        sudo install -Dm755 crates/lekhani-fcitx5/build/fcitx5-lekhani.so "$FCITX_LIB_DIR/fcitx5-lekhani.so"
        echo "--> Installed fcitx5-lekhani.so to $FCITX_LIB_DIR/"
    elif [ -f "fcitx5/fcitx5-lekhani.so" ]; then
        echo "--> Found pre-compiled fcitx5-lekhani.so..."
        sudo install -d "$FCITX_LIB_DIR"
        sudo install -Dm755 fcitx5/fcitx5-lekhani.so "$FCITX_LIB_DIR/fcitx5-lekhani.so"
        echo "--> Installed fcitx5-lekhani.so to $FCITX_LIB_DIR/"
    elif [ -f "crates/lekhani-fcitx5/build/fcitx5-lekhani.so" ]; then
        echo "--> Found existing built fcitx5-lekhani.so..."
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
    sudo install -Dm755 "$IBUS_BIN" /usr/bin/ibus-lekhani
    sudo install -Dm644 data/ibus/lekhani.xml /usr/share/ibus/component/lekhani.xml
fi

echo ""
echo "=========================================================="
echo "    Installation Completed Successfully!                 "
echo "=========================================================="
if [ "$CHOICE" = "--fcitx5" ] || [ "$CHOICE" = "fcitx5" ]; then
    echo "To activate in KDE Plasma 6 / Fcitx5:"
    echo "  1. Run 'fcitx5-remote -e && fcitx5 -d' (or log out and back in)"
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
