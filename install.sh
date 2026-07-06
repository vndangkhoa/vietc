#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Vietnamese Input Method Installer
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'

[ "$EUID" -ne 0 ] && echo -e "${RED}Please run with sudo.${NC}" && exit 1

INSTALLING_USER="${SUDO_USER:-$USER}"

# Parse arguments
FROM_SOURCE=false
for arg in "$@"; do
    if [ "$arg" = "--from-source" ] || [ "$arg" = "--local" ]; then
        FROM_SOURCE=true
    fi
done

echo -e "${GREEN}=== Viet+ Installer ===${NC}"

# Architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ARCH="amd64" ;;
    aarch64) ARCH="arm64" ;;
    *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

# Distro
[ -f /etc/os-release ] && . /etc/os-release
DISTRO="${ID:-unknown}"
echo "Detected: $DISTRO ($ARCH)"

install_deps() {
    # Check if distro is explicitly supported
    local matched=false
    case "$DISTRO" in
        ubuntu|debian|linuxmint|mint|pop|neon|zorin|elementary|fedora|rhel|centos|arch|manjaro|cachyos|endeavouros|garuda|artix)
            matched=true
            ;;
    esac

    # Fallback to package manager detection if distro is not matched
    if [ "$matched" = false ]; then
        if command -v pacman &>/dev/null; then
            DISTRO="arch"
            matched=true
        elif command -v apt-get &>/dev/null; then
            DISTRO="ubuntu"
            matched=true
        elif command -v dnf &>/dev/null; then
            DISTRO="fedora"
            matched=true
        fi
        if [ "$matched" = true ]; then
            echo "Distro ID not explicitly recognized. Falling back to package manager: $DISTRO"
        fi
    fi

    if [ "$FROM_SOURCE" = true ]; then
        echo "Installing build and runtime dependencies..."
        case "$DISTRO" in
            ubuntu|debian|linuxmint|mint|pop|neon|zorin|elementary)
                export DEBIAN_FRONTEND=noninteractive
                apt-get update -y
                apt-get install -y build-essential pkg-config libx11-dev libxtst-dev \
                  libdbus-1-dev libevdev-dev libwayland-dev git \
                  libevdev2 libdbus-1-3 libx11-6 libxtst6 \
                  libwayland-client0 xclip wl-clipboard curl
                ;;
            fedora|rhel|centos)
                dnf groupinstall -y "Development Tools"
                dnf install -y libX11-devel libXtst-devel dbus-devel libevdev-devel wayland-devel git \
                  libevdev libX11 libXtst dbus-libs libwayland-client xclip wl-clipboard curl
                ;;
            arch|manjaro|cachyos|endeavouros|garuda|artix)
                pacman -Sy --needed --noconfirm base-devel pkgconf git \
                  libevdev libx11 libxtst dbus wayland xclip wl-clipboard curl
                ;;
            *)
                echo -e "${YELLOW}Unsupported: $DISTRO. Install deps manually.${NC}"
                ;;
        esac
    else
        echo "Installing runtime dependencies..."
        case "$DISTRO" in
            ubuntu|debian|linuxmint|mint|pop|neon|zorin|elementary)
                export DEBIAN_FRONTEND=noninteractive
                apt-get update -y
                apt-get install -y libevdev2 libdbus-1-3 libx11-6 libxtst6 \
                  libwayland-client0 xclip wl-clipboard curl
                ;;
            fedora|rhel|centos)
                dnf install -y libevdev libX11 libXtst dbus-libs libwayland-client xclip wl-clipboard curl
                ;;
            arch|manjaro|cachyos|endeavouros|garuda|artix)
                pacman -Sy --needed --noconfirm libevdev libx11 libxtst dbus \
                  wayland xclip wl-clipboard curl
                ;;
            *)
                echo -e "${YELLOW}Unsupported: $DISTRO. Install deps manually.${NC}"
                ;;
        esac
    fi
}

install_deps

TMPDIR=$(mktemp -d)
cleanup() { rm -rf "$TMPDIR"; }
trap cleanup EXIT

if [ "$FROM_SOURCE" = true ]; then
    # Install Rust if missing
    if ! command -v cargo &>/dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        export PATH="$HOME/.cargo/bin:$PATH"
        if [ -n "${SUDO_USER:-}" ] && [ -d "/home/$SUDO_USER/.cargo/bin" ]; then
            export PATH="/home/$SUDO_USER/.cargo/bin:$PATH"
        fi
    fi

    # Clone staging if not in repo
    if [ ! -f Cargo.toml ] || [ ! -d .git ]; then
        echo "Cloning staging branch to build..."
        git clone -b staging https://github.com/vndangkhoa/vietc.git "$TMPDIR/source"
        cd "$TMPDIR/source"
    fi

    echo "Building from source..."
    cargo build --release
    (cd ui && cargo build --release)
else
    echo "Fetching latest release..."
    RELEASE_JSON=$(curl -sSfL "https://api.github.com/repos/vndangkhoa/vietc/releases/latest" 2>/dev/null || echo "")
    TAG=$(echo "$RELEASE_JSON" | grep '"tag_name"' | sed 's/.*"v\(.*\)",/\1/')
    if [ -z "$TAG" ]; then
        echo -e "${RED}Failed to fetch latest release info.${NC}"
        exit 1
    fi
    echo "Latest version: v$TAG"

    # Try tarball first, then .deb
    TARBALL="vietc_${TAG}_linux_${ARCH}.tar.gz"
    TARBALL_URL="https://github.com/vndangkhoa/vietc/releases/download/v${TAG}/${TARBALL}"
    DEB="vietc_${TAG}-1_amd64.deb"
    DEB_URL="https://github.com/vndangkhoa/vietc/releases/download/v${TAG}/${DEB}"
    INSTALL_DIR="$TMPDIR/install"
    mkdir -p "$INSTALL_DIR"

    if curl -sSfL -o "$TMPDIR/$TARBALL" "$TARBALL_URL" 2>/dev/null; then
        echo "Downloading tarball..."
        tar -xzf "$TMPDIR/$TARBALL" -C "$INSTALL_DIR"
        BIN_DIR="$INSTALL_DIR/vietc_${TAG}_linux_${ARCH}/bin"
        PKG_DIR="$INSTALL_DIR/vietc_${TAG}_linux_${ARCH}"
    elif curl -sSfL -o "$TMPDIR/$DEB" "$DEB_URL" 2>/dev/null; then
        echo "Downloading .deb package..."
        if command -v dpkg-deb &>/dev/null; then
            dpkg-deb -x "$TMPDIR/$DEB" "$INSTALL_DIR"
        else
            mkdir -p "$TMPDIR/deb"
            ar x "$TMPDIR/$DEB" --output="$TMPDIR/deb" 2>/dev/null
            tar -xzf "$TMPDIR/deb/data.tar.gz" -C "$INSTALL_DIR" 2>/dev/null || \
            tar -xJf "$TMPDIR/deb/data.tar.xz" -C "$INSTALL_DIR" 2>/dev/null || \
            tar --zstd -xf "$TMPDIR/deb/data.tar.zst" -C "$INSTALL_DIR" 2>/dev/null || true
        fi
        BIN_DIR="$INSTALL_DIR/usr/bin"
        PKG_DIR="$INSTALL_DIR"
    else
        echo -e "${RED}No prebuilt binary found for v$TAG ($ARCH).${NC}"
        echo -e "${YELLOW}Visit https://github.com/vndangkhoa/vietc/releases${NC}"
        exit 1
    fi
fi

# Kill old processes
pkill -x vietc-tray 2>/dev/null || true
pkill -x vietc-daemon 2>/dev/null || true
pkill -x vietc 2>/dev/null || true

# Install binaries
echo "Installing to /usr/bin/..."
if [ "$FROM_SOURCE" = true ]; then
    cp target/release/vietc /usr/bin/vietc-daemon
    cp target/release/vietc-cli /usr/bin/vietc-cli
    cp target/release/vietc-uinputd /usr/bin/vietc-uinputd
    cp ui/target/release/vietc-tray /usr/bin/vietc-tray
    [ -f target/release/vietc-xrecord ] && cp target/release/vietc-xrecord /usr/bin/vietc-xrecord || true
else
    cp "$BIN_DIR/vietc-daemon" /usr/bin/vietc-daemon
    cp "$BIN_DIR/vietc-cli" /usr/bin/vietc-cli
    cp "$BIN_DIR/vietc-uinputd" /usr/bin/vietc-uinputd
    cp "$BIN_DIR/vietc-tray" /usr/bin/vietc-tray
    [ -f "$BIN_DIR/vietc-xrecord" ] && cp "$BIN_DIR/vietc-xrecord" /usr/bin/vietc-xrecord || true
fi
chmod 755 /usr/bin/vietc-daemon /usr/bin/vietc-cli /usr/bin/vietc-uinputd /usr/bin/vietc-tray 2>/dev/null || true

# Grant cap_sys_admin so evdev grab works without full root (Linux ≥ 5.8)
# Also grant cap_dac_override for /dev/uinput access if not in input group
if command -v setcap &>/dev/null; then
    setcap cap_sys_admin,cap_dac_override+ep /usr/bin/vietc-daemon 2>/dev/null && \
        echo -e "${GREEN}setcap: vietc-daemon can grab keyboard without full root${NC}" || \
        echo -e "${YELLOW}setcap failed — run with sudo for grab${NC}"
fi

# Clean old /usr/local/bin/ binaries
rm -f /usr/local/bin/vietc /usr/local/bin/vietc-daemon /usr/local/bin/vietc-cli \
      /usr/local/bin/vietc-uinputd /usr/local/bin/vietc-tray /usr/local/bin/vietc-xrecord 2>/dev/null || true

# Clean old local user binaries & autostart to prevent shadowing the new system-wide ones
if [ -n "$INSTALLING_USER" ] && [ "$INSTALLING_USER" != "root" ]; then
    USER_HOME="$(getent passwd "$INSTALLING_USER" | cut -d: -f6 || true)"
    if [ -n "$USER_HOME" ]; then
        rm -f "$USER_HOME/.local/bin/vietc" "$USER_HOME/.local/bin/vietc-daemon" \
               "$USER_HOME/.local/bin/vietc-cli" "$USER_HOME/.local/bin/vietc-uinputd" \
               "$USER_HOME/.local/bin/vietc-tray" "$USER_HOME/.local/bin/vietc-xrecord" \
               "$USER_HOME/.local/bin/vietc-start" 2>/dev/null || true
        rm -f "$USER_HOME/.config/autostart/vietc.desktop" 2>/dev/null || true
    fi
fi

# Udev rules & Kernel module
mkdir -p /etc/modules-load.d
echo "uinput" > /etc/modules-load.d/vietc.conf
modprobe uinput 2>/dev/null || true
echo 'KERNEL=="uinput", SUBSYSTEM=="misc", GROUP="input", MODE="0660"' > /etc/udev/rules.d/99-vietc.rules
udevadm control --reload-rules 2>/dev/null || true
udevadm trigger 2>/dev/null || true

# Icons
if [ "$FROM_SOURCE" = true ]; then
    mkdir -p /usr/share/icons/hicolor/256x256/apps
    cp packaging/icons/*.svg /usr/share/icons/hicolor/256x256/apps/ 2>/dev/null || true
else
    if [ -d "$PKG_DIR/icons" ]; then
        mkdir -p /usr/share/icons/hicolor/256x256/apps
        cp "$PKG_DIR/icons"/*.svg /usr/share/icons/hicolor/256x256/apps/ 2>/dev/null || true
    elif [ -d "$INSTALL_DIR/usr/share/icons" ]; then
        cp -r "$INSTALL_DIR/usr/share/icons/"* /usr/share/icons/ 2>/dev/null || true
    fi
fi

# Desktop file
if [ "$FROM_SOURCE" = true ]; then
    mkdir -p /usr/share/applications
    cp packaging/deb/vietc.desktop /usr/share/applications/
else
    if [ -f "$PKG_DIR/desktop/vietc.desktop" ]; then
        mkdir -p /usr/share/applications
        cp "$PKG_DIR/desktop/vietc.desktop" /usr/share/applications/
    elif [ -f "$INSTALL_DIR/usr/share/applications/vietc.desktop" ]; then
        cp "$INSTALL_DIR/usr/share/applications/vietc.desktop" /usr/share/applications/
    fi
fi

# XDG autostart
mkdir -p /etc/xdg/autostart
cat > /etc/xdg/autostart/vietc-tray.desktop << 'EOF'
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
EOF

# Systemd user service
mkdir -p /usr/lib/systemd/user
cat > /usr/lib/systemd/user/vietc.service << 'EOF'
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
EOF

# User setup
INSTALLING_USER="${SUDO_USER:-$USER}"
if [ -n "$INSTALLING_USER" ] && [ "$INSTALLING_USER" != "root" ]; then
    if command -v usermod &>/dev/null; then
        usermod -aG input "$INSTALLING_USER" 2>/dev/null || true
    elif command -v adduser &>/dev/null; then
        adduser "$INSTALLING_USER" input 2>/dev/null || true
    fi
    rm -f "$(getent passwd "$INSTALLING_USER" | cut -d: -f6)/.config/vietc/config.toml" 2>/dev/null || true
fi

# Config
mkdir -p /etc/vietc
if [ "$FROM_SOURCE" = true ]; then
    cp vietc.toml /etc/vietc/config.toml
else
    if [ -f "$PKG_DIR/config/config.toml" ]; then
        cp "$PKG_DIR/config/config.toml" /etc/vietc/config.toml
    elif [ -f "$INSTALL_DIR/etc/vietc/config.toml" ]; then
        cp "$INSTALL_DIR/etc/vietc/config.toml" /etc/vietc/config.toml
    fi
fi
if [ ! -f /etc/vietc/config.toml ]; then
    cat > /etc/vietc/config.toml << 'EOF'
input_method = "vni"
toggle_key = "space"
start_enabled = true
grab = true

[app_state]
enabled = true
english_apps = ["code", "vim"]
vietnamese_apps = ["telegram", "discord", "firefox"]
EOF
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Viet+ installed successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

if command -v setcap &>/dev/null && getcap /usr/bin/vietc-daemon 2>/dev/null | grep -q cap_sys_admin; then
    echo -e "${GREEN}✓ CAP_SYS_ADMIN granted — daemon can grab keyboard without root${NC}"
    echo ""
    echo -e "Start the tray now:  ${GREEN}vietc-tray${NC}"
    echo -e "Or test directly:     ${GREEN}sudo -u $INSTALLING_USER vietc-daemon &${NC}"
else
    echo -e "${YELLOW}⚠  Daemon needs root for keyboard grab${NC}"
    echo ""
    echo -e "Start the daemon:     ${GREEN}sudo vietc-daemon${NC}"
    echo -e "Then run the tray:    ${GREEN}vietc-tray${NC}"
    echo ""
    echo -e "Or configure passwordless sudo:"
    echo -e "  ${GREEN}echo \"$INSTALLING_USER ALL=(ALL) NOPASSWD: /usr/bin/vietc-daemon\" | sudo tee /etc/sudoers.d/vietc${NC}"
fi

echo ""
echo -e "Test: type in Vietnamese in any app."
echo -e "Toggle VN/EN: ${GREEN}Ctrl+Space${NC}  Switch VNI/Telex: ${GREEN}Ctrl+Shift${NC}"
echo ""
echo -e "See ${GREEN}vietc.toml${NC} for configuration."
