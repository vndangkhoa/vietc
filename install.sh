#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Vietnamese Input Method Installer
set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0;69m' # No Color
NC='\033[0m'

echo -e "${BLUE}=== Viet+ Installation Script ===${NC}"

# Check for root privilege requirement
check_root() {
    if [ "$EUID" -ne 0 ]; then
        echo -e "${RED}Error: This script must be run as root (or with sudo) to install system files and update permissions.${NC}"
        echo -e "Please re-run as: ${YELLOW}sudo $0${NC}"
        exit 1
    fi
}

# Detect distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO=$ID
        DISTRO_LIKE=${ID_LIKE:-""}
    else
        echo -e "${RED}Error: Cannot detect Linux distribution (/etc/os-release missing).${NC}"
        exit 1
    fi
    echo -e "Detected OS: ${GREEN}${DISTRO}${NC}"
}

# Install dependencies using package manager
install_dependencies() {
    local mode=$1 # "build" or "run"
    echo -e "Installing ${mode} dependencies..."
    
    case "$DISTRO" in
        ubuntu|debian|raspbian|pop|mint|linuxmint|neon|zorin|elementary)
            export DEBIAN_FRONTEND=noninteractive
            apt-get update -y
            if [ "$mode" = "build" ]; then
                apt-get install -y build-essential pkg-config libx11-dev libxtst-dev libdbus-1-dev libevdev-dev libwayland-dev curl
            fi
            apt-get install -y libevdev2 libdbus-1-3 libx11-6 libxtst6 xclip wl-clipboard libwayland-client0 curl
            ;;
        fedora|rhel|centos)
            if [ "$mode" = "build" ]; then
                dnf install -y gcc pkgconfig libX11-devel libXtst-devel dbus-devel libevdev-devel
            fi
            dnf install -y libevdev libX11 libXtst dbus-libs xclip wl-clipboard
            ;;
        arch|manjaro|arco)
            if [ "$mode" = "build" ]; then
                pacman -Sy --needed --noconfirm base-devel pkgconf libx11 libxtst dbus libevdev
            fi
            pacman -Sy --needed --noconfirm libevdev libx11 libxtst dbus xclip wl-clipboard
            ;;
        *)
            if [[ "$DISTRO_LIKE" == *"ubuntu"* || "$DISTRO_LIKE" == *"debian"* ]]; then
                export DEBIAN_FRONTEND=noninteractive
                apt-get update -y
                if [ "$mode" = "build" ]; then
                    apt-get install -y build-essential pkg-config libx11-dev libxtst-dev libdbus-1-dev libevdev-dev libwayland-dev curl
                fi
                apt-get install -y libevdev2 libdbus-1-3 libx11-6 libxtst6 xclip wl-clipboard libwayland-client0 curl
            else
                echo -e "${YELLOW}Warning: Unsupported distribution '${DISTRO}'. Please make sure you have the following packages installed manually:${NC}"
                echo -e "  - libevdev, libdbus-1, libx11, libxtst, xclip, wl-clipboard"
                if [ "$mode" = "build" ]; then
                    echo -e "  - gcc, pkg-config, and development headers for the above libraries."
                fi
            fi
            ;;
    esac
}

# Check for Rust compiler (needed only for source build)
check_rust() {
    if [ -n "${SUDO_USER:-}" ]; then
        local user_home
        user_home="$(getent passwd "$SUDO_USER" 2>/dev/null | cut -d: -f6 || echo "/home/$SUDO_USER")"
        export CARGO_HOME="$user_home/.cargo"
        export RUSTUP_HOME="$user_home/.rustup"
        export PATH="$CARGO_HOME/bin:$PATH"
    fi

    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${YELLOW}Rust toolchain not found.${NC}"
        read -p "Would you like to install Rust toolchain now? [Y/n] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            echo -e "${RED}Error: Cargo is required to build from source.${NC}"
            exit 1
        fi
        echo "Installing Rust via rustup.rs..."
        # Run rustup installer as the original non-root user if SUDO_USER is set
        if [ -n "${SUDO_USER:-}" ]; then
            su - "$SUDO_USER" -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
        else
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            export PATH="$HOME/.cargo/bin:$PATH"
        fi
    fi
    echo -e "Rust version: ${GREEN}$(rustc --version)${NC}"
}

# Determine if we are installing from source tree or prebuilt binaries
check_root # Need root to perform OS checks and dependencies install
detect_distro

SOURCE_DIR=""
if [ -f "./Cargo.toml" ] && [ -d "./engine" ] && [ -d "./ui" ]; then
    SOURCE_DIR=$(pwd)
    echo -e "Status: Running from ${GREEN}Source Tree${NC}"
else
    echo -e "Status: Running from ${GREEN}Release Package${NC}"
fi

# 1. Compile or Stage Binaries
if [ -n "$SOURCE_DIR" ]; then
    install_dependencies "build"
    check_rust
    
    echo -e "=== Compiling Viet+ ==="
    # Build core components
    cargo build --release
    # Build UI
    (cd ui && cargo build --release)
    # Compile C helper
    gcc -O2 -o target/release/vietc-xrecord packaging/deb/vietc-xrecord.c -lX11 -lXtst
    
    BIN_DAEMON="target/release/vietc"
    BIN_CLI="target/release/vietc-cli"
    BIN_UINPUTD="target/release/vietc-uinputd"
    BIN_TRAY="ui/target/release/vietc-tray"
    BIN_XRECORD="target/release/vietc-xrecord"
    
    FILE_RULES="packaging/99-vietc.rules"
    FILE_DESKTOP="packaging/deb/vietc.desktop"
    FILE_CONFIG="vietc.toml"
    DIR_ICONS="packaging/icons"
else
    # Install from prebuilt release folder
    install_dependencies "run"
    
    BIN_DAEMON="bin/vietc-daemon"
    BIN_CLI="bin/vietc-cli"
    BIN_UINPUTD="bin/vietc-uinputd"
    BIN_TRAY="bin/vietc-tray"
    BIN_XRECORD="bin/vietc-xrecord"
    
    FILE_RULES="udev/99-vietc.rules"
    FILE_DESKTOP="desktop/vietc.desktop"
    FILE_CONFIG="config/config.toml"
    DIR_ICONS="icons"
    
    # Validation
    if [ ! -f "$BIN_DAEMON" ] && [ -f "bin/vietc" ]; then
        BIN_DAEMON="bin/vietc" # Fallback if not renamed in build
    fi
fi

# Kill running instances before installing new files
echo "Stopping any running Viet+ processes..."
pkill -x vietc-tray 2>/dev/null || true
pkill -x vietc-daemon 2>/dev/null || true
pkill -x vietc-uinputd 2>/dev/null || true
pkill -x vietc 2>/dev/null || true

# 2. Install Binaries
echo "Installing binaries to /usr/bin/..."
cp "$BIN_DAEMON" /usr/bin/vietc-daemon
cp "$BIN_CLI" /usr/bin/
cp "$BIN_UINPUTD" /usr/bin/
cp "$BIN_TRAY" /usr/bin/
cp "$BIN_XRECORD" /usr/bin/
chmod 755 /usr/bin/vietc-daemon /usr/bin/vietc-cli /usr/bin/vietc-uinputd /usr/bin/vietc-tray /usr/bin/vietc-xrecord

# Remove old local path binaries to prevent shadows
rm -f /usr/local/bin/vietc-tray /usr/local/bin/vietc /usr/local/bin/vietc-daemon \
      /usr/local/bin/vietc-cli /usr/local/bin/vietc-uinputd /usr/local/bin/vietc-xrecord 2>/dev/null || true

# 3. Install Icons
echo "Installing icons..."
mkdir -p /usr/share/icons/hicolor/256x256/apps
cp "$DIR_ICONS/vietc.svg" /usr/share/icons/hicolor/256x256/apps/
cp "$DIR_ICONS/vietc-vn.svg" /usr/share/icons/hicolor/256x256/apps/
cp "$DIR_ICONS/vietc-en.svg" /usr/share/icons/hicolor/256x256/apps/
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor/ >/dev/null 2>&1 || true
fi

# 4. Install Desktop File
echo "Installing desktop launcher..."
mkdir -p /usr/share/applications
cp "$FILE_DESKTOP" /usr/share/applications/

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
fi

# 5. Install Systemd User Service
echo "Installing systemd user service..."
mkdir -p /usr/lib/systemd/user
cat > /usr/lib/systemd/user/vietc.service << 'SERVICE'
[Unit]
Description=Viet+ Vietnamese IME Tray
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/vietc-tray
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
SERVICE

# 6. Install XDG Autostart
echo "Setting up autostart..."
mkdir -p /etc/xdg/autostart
cat > /etc/xdg/autostart/vietc-tray.desktop << 'AUTOSTART'
[Desktop Entry]
Type=Application
Name=Viet+ Tray
Comment=Vietnamese Input Method Tray
Exec=vietc-tray
Icon=vietc
Terminal=false
Categories=Utility;
StartupNotify=false
NoDisplay=true
AUTOSTART

# 7. Install default config
echo "Installing default configuration..."
mkdir -p /etc/vietc
cp "$FILE_CONFIG" /etc/vietc/config.toml
chmod 644 /etc/vietc/config.toml

# 8. Configure permissions (uinput)
echo "Installing udev rules for uinput access..."
mkdir -p /etc/udev/rules.d
cp "$FILE_RULES" /etc/udev/rules.d/99-vietc.rules
chmod 644 /etc/udev/rules.d/99-vietc.rules

echo "Reloading udev rules..."
if command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules >/dev/null 2>&1 || true
    udevadm trigger --subsystem-match=misc >/dev/null 2>&1 || true
fi

# Add active users to input group
INSTALLING_USER="${SUDO_USER:-${USER:-}}"
if [ -n "$INSTALLING_USER" ] && [ "$INSTALLING_USER" != "root" ]; then
    echo "Adding user '$INSTALLING_USER' to group 'input'..."
    if ! groups "$INSTALLING_USER" 2>/dev/null | grep -qw input; then
        if command -v usermod >/dev/null 2>&1; then
            usermod -aG input "$INSTALLING_USER" || true
        elif command -v adduser >/dev/null 2>&1; then
            adduser "$INSTALLING_USER" input || true
        fi
    fi
    
    # Remove any old conflicting configurations in user home
    user_home="$(getent passwd "$INSTALLING_USER" 2>/dev/null | cut -d: -f6 || true)"
    if [ -n "$user_home" ]; then
        rm -f "$user_home/.config/vietc/config.toml" 2>/dev/null || true
        rm -f "$user_home/.config/vietc/overrides.toml" 2>/dev/null || true
        rm -f "$user_home/.config/vietc/.first-launch-done" 2>/dev/null || true
    fi
fi

if command -v systemctl >/dev/null 2>&1; then
    systemctl --global daemon-reload >/dev/null 2>&1 || true
fi

echo -e "\n${GREEN}=== Installation Completed Successfully! ===${NC}"
echo -e "Please ${YELLOW}LOG OUT and LOG BACK IN${NC} to activate group permissions."
echo -e "Once logged back in, you can start Viet+ from your application launcher menu."
