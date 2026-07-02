#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Vietnamese Input Method Installer
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'

[ "$EUID" -ne 0 ] && echo -e "${RED}Please run with sudo.${NC}" && exit 1

echo -e "${GREEN}=== Viet+ Installer ===${NC}"

# Detect distro
[ -f /etc/os-release ] && . /etc/os-release
DISTRO="${ID:-unknown}"
echo "Detected: $DISTRO"

# Install dependencies
install_deps() {
    case "$DISTRO" in
        ubuntu|debian|linuxmint|mint|pop|neon|zorin|elementary)
            export DEBIAN_FRONTEND=noninteractive
            apt-get update -y
            apt-get install -y build-essential pkg-config libx11-dev libxtst-dev \
              libdbus-1-dev libevdev-dev libwayland-dev curl git
            apt-get install -y libevdev2 libdbus-1-3 libx11-6 libxtst6 \
              libwayland-client0 xclip wl-clipboard
            ;;
        fedora|rhel|centos)
            dnf install -y gcc pkgconfig libX11-devel libXtst-devel dbus-devel \
              libevdev-devel libwayland-devel curl git
            dnf install -y libevdev libX11 libXtst dbus-libs libwayland-client xclip wl-clipboard
            ;;
        arch|manjaro)
            pacman -Sy --needed --noconfirm base-devel pkgconf libx11 libxtst dbus \
              libevdev wayland curl git
            pacman -Sy --needed --noconfirm libevdev libx11 libxtst dbus \
              libwayland xclip wl-clipboard
            ;;
        *)
            echo -e "${YELLOW}Unsupported: $DISTRO. Install deps manually.${NC}"
            ;;
    esac
}

install_deps

# Install Rust if missing
if ! command -v cargo &>/dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Kill old processes
pkill -x vietc-tray 2>/dev/null || true
pkill -x vietc-daemon 2>/dev/null || true
pkill -x vietc 2>/dev/null || true

# Build
echo "Building..."
cargo build --release
(cd ui && cargo build --release)
if command -v gcc &>/dev/null && [ -f packaging/deb/vietc-xrecord.c ]; then
    gcc -O2 -o target/release/vietc-xrecord packaging/deb/vietc-xrecord.c -lX11 -lXtst 2>/dev/null || true
fi

# Install binaries
echo "Installing to /usr/bin/..."
cp target/release/vietc /usr/bin/vietc-daemon
cp target/release/vietc-cli /usr/bin/
cp target/release/vietc-uinputd /usr/bin/
cp ui/target/release/vietc-tray /usr/bin/
[ -f target/release/vietc-xrecord ] && cp target/release/vietc-xrecord /usr/bin/
chmod 755 /usr/bin/vietc-daemon /usr/bin/vietc-cli /usr/bin/vietc-uinputd /usr/bin/vietc-tray 2>/dev/null || true

# Clean old /usr/local/bin/ binaries
rm -f /usr/local/bin/vietc /usr/local/bin/vietc-daemon /usr/local/bin/vietc-cli \
      /usr/local/bin/vietc-uinputd /usr/local/bin/vietc-tray /usr/local/bin/vietc-xrecord 2>/dev/null || true

# Udev rules for uinput
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' > /etc/udev/rules.d/99-vietc.rules
udevadm control --reload-rules 2>/dev/null || true
udevadm trigger 2>/dev/null || true

# User setup
INSTALLING_USER="${SUDO_USER:-$USER}"
if [ -n "$INSTALLING_USER" ] && [ "$INSTALLING_USER" != "root" ]; then
    adduser "$INSTALLING_USER" input 2>/dev/null || true
    rm -f "$(getent passwd "$INSTALLING_USER" | cut -d: -f6)/.config/vietc/config.toml" 2>/dev/null || true
fi

# Create default config
mkdir -p /etc/vietc
cat > /etc/vietc/config.toml << 'EOF'
input_method = "vni"
toggle_key = "space"
toggle_method_key = "shift"
start_enabled = true
grab = true

[password_detection]
enabled = true
check_atspi2 = true
check_window_title = true
title_keywords = ["password", "passphrase", "secret", "mật khẩu", "sudo"]
password_apps = ["pinentry", "pinentry-gtk-2", "pinentry-qt", "kwallet"]

[app_state]
enabled = true
english_apps = ["code", "vim"]
vietnamese_apps = ["telegram", "discord", "firefox"]
bypass_apps = ["steam"]
terminal_apps = ["kitty", "alacritty", "gnome-terminal", "konsole", "foot",
  "wezterm", "st", "urxvt", "xterm"]
terminal_input_method = "vni"
EOF

echo -e "${GREEN}=== Done! ===${NC}"
echo -e "${YELLOW}Log out and log back in, then run: vietc-tray${NC}"
