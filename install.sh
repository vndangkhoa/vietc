#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Vietnamese Input Method Installer
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'

[ "$EUID" -ne 0 ] && echo -e "${RED}Please run with sudo.${NC}" && exit 1

INSTALLING_USER="${SUDO_USER:-$USER}"

# Parse arguments
FROM_SOURCE=false
PREBUILT=false
MODE="grab"   # grab = original evdev/IBus-engine capture path; bamboo = Bamboo aux-controller
for arg in "$@"; do
    if [ "$arg" = "--from-source" ] || [ "$arg" = "--local" ]; then
        FROM_SOURCE=true
    elif [ "$arg" = "--prebuilt" ]; then
        PREBUILT=true
    elif [ "$arg" = "--bamboo" ]; then
        MODE="bamboo"
    elif [ "$arg" = "--grab" ]; then
        MODE="grab"
    fi
done

# When run from a source tree (git clone), build from source by default so the
# freshly cloned code (e.g. the rootless Wayland path) is what gets installed,
# instead of a possibly-stale prebuilt release. Pass --prebuilt to force a
# release download.
if [ "$FROM_SOURCE" != true ] && [ "$PREBUILT" != true ] && [ -f Cargo.toml ]; then
    echo -e "${YELLOW}Source tree detected — building from source.${NC}"
    echo -e "${YELLOW}(pass --prebuilt to download a release instead)${NC}"
    FROM_SOURCE=true
fi

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

    # Install build dependencies (needed to compile the daemon)
    echo "Installing build dependencies..."
    case "$DISTRO" in
        ubuntu|debian|linuxmint|mint|pop|neon|zorin|elementary)
            export DEBIAN_FRONTEND=noninteractive
            apt-get update -y
            apt-get install -y build-essential pkg-config libxkbcommon-dev \
              libx11-dev libxtst-dev libwayland-dev libevdev-dev libdbus-1-dev libssl-dev
            ;;
        fedora|rhel|centos)
            dnf install -y gcc pkgconf-pkg-config libxkbcommon-devel \
              libX11-devel libXtst-devel wayland-devel libevdev-devel dbus-devel
            ;;
        arch|manjaro|cachyos|endeavouros|garuda|artix)
            pacman -Sy --needed --noconfirm base-devel pkgconf \
              libxkbcommon wayland libevdev dbus
            ;;
    esac

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
    cp ui/target/release/vietc-tray /usr/bin/vietc-tray 2>/dev/null || true
    [ -f target/release/vietc-xrecord ] && cp target/release/vietc-xrecord /usr/bin/vietc-xrecord || true
    [ -f target/release/vietcctl ] && cp target/release/vietcctl /usr/bin/vietcctl || true
else
    cp "$BIN_DIR/vietc-daemon" /usr/bin/vietc-daemon
    cp "$BIN_DIR/vietc-cli" /usr/bin/vietc-cli
    cp "$BIN_DIR/vietc-uinputd" /usr/bin/vietc-uinputd
    cp "$BIN_DIR/vietc-tray" /usr/bin/vietc-tray 2>/dev/null || true
    [ -f "$BIN_DIR/vietc-xrecord" ] && cp "$BIN_DIR/vietc-xrecord" /usr/bin/vietc-xrecord || true
    [ -f "$BIN_DIR/vietcctl" ] && cp "$BIN_DIR/vietcctl" /usr/bin/vietcctl || true
fi
chmod 755 /usr/bin/vietc-daemon /usr/bin/vietc-cli /usr/bin/vietc-uinputd /usr/bin/vietc-tray /usr/bin/vietcctl 2>/dev/null || true

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

# XDG autostart for the tray is intentionally NOT installed: the systemd user
# service below already starts vietc-daemon (rootless). Running the tray's
# autostart too would spawn a second daemon. The tray is optional UI — run
# `vietc-tray` manually if you want the menu/status icon.

# Systemd user service (rootless: runs vietc-daemon directly, no grab/setcap)
mkdir -p /usr/lib/systemd/user
cat > /usr/lib/systemd/user/vietc.service << 'EOF'
[Unit]
Description=Viet+ Vietnamese IME Daemon (rootless)
PartOf=graphical-session.target
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/vietc-daemon
Restart=on-failure
RestartSec=3
# Only kill the daemon on stop; the IBus it respawns (IbusRestartGuard) must
# survive so input works again after vietc exits.
KillMode=process
ConditionEnvironment=DISPLAY

[Install]
WantedBy=graphical-session.target
EOF

# Prevent a stale user-local unit from shadowing this one (a leftover would
# point at a binary the cleanup step deletes, causing the service to crash-loop
# with status=203/EXEC).
INSTALLING_USER="${SUDO_USER:-$USER}"
USER_HOME="$(getent passwd "$INSTALLING_USER" 2>/dev/null | cut -d: -f6 || true)"
if [ -n "$USER_HOME" ] && [ "$INSTALLING_USER" != "root" ]; then
    rm -f "$USER_HOME/.config/systemd/user/vietc.service" \
          "$USER_HOME/.config/systemd/user/graphical-session.target.wants/vietc.service" \
          "$USER_HOME/.config/systemd/user/default.target.wants/vietc.service" 2>/dev/null || true
    mkdir -p "$USER_HOME/.config/systemd/user/graphical-session.target.wants"
    ln -sf /usr/lib/systemd/user/vietc.service \
          "$USER_HOME/.config/systemd/user/graphical-session.target.wants/vietc.service"
    chown -R "$INSTALLING_USER" "$USER_HOME/.config/systemd/user" 2>/dev/null || true
    # Best-effort live enable if the user's systemd is running.
    U_UID="$(id -u "$INSTALLING_USER" 2>/dev/null)"
    if command -v systemctl >/dev/null 2>&1 && [ -n "$U_UID" ]; then
        sudo -u "$INSTALLING_USER" XDG_RUNTIME_DIR="/run/user/$U_UID" \
            DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$U_UID/bus" \
            systemctl --user daemon-reload 2>/dev/null || true
        sudo -u "$INSTALLING_USER" XDG_RUNTIME_DIR="/run/user/$U_UID" \
            DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$U_UID/bus" \
            systemctl --user enable --now vietc.service 2>/dev/null || true
    fi
fi

# Run a command as the installing user with their D-Bus session, so IBus /
# GNOME settings land in the right dconf and the tray/shortcut apply to them.
run_as_user() {
    local u="${INSTALLING_USER:-$USER}"
    local uid="$(id -u "$u" 2>/dev/null)"
    sudo -u "$u" DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$uid/bus" \
        XDG_RUNTIME_DIR="/run/user/$uid" "$@"
}

# Bamboo aux-controller setup: install ibus-bamboo, preload its engines, enable
# per-app engine memory, and write a sane Bamboo config. Best-effort per distro.
setup_bamboo() {
    echo "Bamboo aux-controller mode: installing/verifying ibus-bamboo + IBus config..."
    case "$DISTRO" in
        ubuntu|debian|linuxmint|mint|pop|neon|zorin|elementary)
            if ! command -v ibus-bamboo &>/dev/null && \
               [ ! -f /usr/lib/ibus/ibus-engine-bamboo ] && \
               [ ! -f /usr/libexec/ibus/ibus-engine-bamboo ]; then
                export DEBIAN_FRONTEND=noninteractive
                add-apt-repository -y ppa:bamboo-engine/ibus-bamboo 2>/dev/null || true
                apt-get update -y 2>/dev/null || true
                apt-get install -y ibus-bamboo 2>/dev/null || \
                    echo -e "${YELLOW}Could not auto-install ibus-bamboo; install manually: https://github.com/BambooEngine/ibus-bamboo${NC}"
            fi
            ;;
        arch|manjaro|cachyos|endeavouros|garuda|artix)
            if [ ! -f /usr/lib/ibus/ibus-engine-bamboo ]; then
                (command -v yay &>/dev/null && yay -S --noconfirm ibus-bamboo) || \
                (command -v paru &>/dev/null && paru -S --noconfirm ibus-bamboo) || \
                    echo -e "${YELLOW}Install ibus-bamboo manually (AUR).${NC}"
            fi
            ;;
        *)
            echo -e "${YELLOW}Install ibus-bamboo manually for $DISTRO: https://github.com/BambooEngine/ibus-bamboo${NC}"
            ;;
    esac

    # Apply IBus settings as the real user (root's dconf is irrelevant).
    if [ -n "${INSTALLING_USER:-}" ] && [ "$INSTALLING_USER" != "root" ]; then
        run_as_user gsettings set org.freedesktop.ibus.general preload-engines "['Bamboo', 'BambooUs']" 2>/dev/null || \
            run_as_user dconf write /desktop/ibus/general/preload-engines "['Bamboo', 'BambooUs']" 2>/dev/null || true
        # Per-app engine memory: ptyxis=EN, firefox=VN, etc. (required for
        # vietc to leave Wayland-native apps alone and keep each app's engine).
        run_as_user dconf write /desktop/ibus/general/use-global-engine false 2>/dev/null || true

        BAMBOO_HOME="$(getent passwd "$INSTALLING_USER" | cut -d: -f6)/.config/ibus-bamboo"
        mkdir -p "$BAMBOO_HOME"
        if [ ! -f "$BAMBOO_HOME/ibus-bamboo.config.json" ]; then
            cat > "$BAMBOO_HOME/ibus-bamboo.config.json" << 'EOF'
{
  "InputMethod": "VNI",
  "DefaultInputMode": 2
}
EOF
            chown -R "$INSTALLING_USER" "$BAMBOO_HOME" 2>/dev/null || true
        fi
    fi
}

# User setup
INSTALLING_USER="${SUDO_USER:-$USER}"
USER_HOME="$(getent passwd "$INSTALLING_USER" 2>/dev/null | cut -d: -f6 || true)"
if [ -n "$INSTALLING_USER" ] && [ "$INSTALLING_USER" != "root" ]; then
    if command -v usermod &>/dev/null; then
        usermod -aG input "$INSTALLING_USER" 2>/dev/null || true
    elif command -v adduser &>/dev/null; then
        adduser "$INSTALLING_USER" input 2>/dev/null || true
    fi
    # grab mode keeps the legacy behaviour of dropping the user config so the
    # built-in/system defaults take over. bamboo mode KEEPS it — that is the
    # file the daemon actually reads.
    if [ "$MODE" = "grab" ]; then
        rm -f "$USER_HOME/.config/vietc/config.toml" 2>/dev/null || true
    fi
fi

# Config
mkdir -p /etc/vietc
if [ "$MODE" = "bamboo" ]; then
    setup_bamboo
    # The daemon reads ~/.config/vietc/config.toml (NOT /etc/vietc), so write
    # the controller config where it will actually be used.
    if [ -n "$USER_HOME" ]; then
        mkdir -p "$USER_HOME/.config/vietc"
        cat > "$USER_HOME/.config/vietc/config.toml" << 'EOF'
input_method = "vni"
toggle_key = "space"
start_enabled = true
grab = false
debug = false
ibus_engine = false
controller_mode = true

[auto_restore]
enabled = true
trigger_keys = ["space", "escape"]

[app_state]
enabled = true
english_apps = ["code", "jetbrains", "intellij", "pycharm", "webstorm", "vim", "nvim", "kitty", "alacritty", "foot", "ghostty"]
vietnamese_apps = ["telegram", "discord", "slack", "firefox", "chromium", "thunderbird", "gedit", "gnome-text-editor", "org.gnome.TextEditor"]
terminal_apps = ["terminal", "kitty", "alacritty", "foot", "wezterm", "konsole", "gnome-terminal", "gnome-terminal-server", "ptyxis", "kgx", "st", "urxvt", "xterm", "terminator", "tilix"]
bypass_apps = ["steam", "dota", "csgo", "minecraft", "factorio"]
terminal_input_method = "vni"

[macros]
bt = "biết"
vs = "với"
kc = "không có"
dc = "được"
ko = "không"
rd = "rất"
nk = "như"
"ko dc" = "không được"
lm = "làm"
ng = "người"
EOF
        chown -R "$INSTALLING_USER" "$USER_HOME/.config/vietc" 2>/dev/null || true
    fi
    # System copy for reference (not read by the daemon).
    cp "$USER_HOME/.config/vietc/config.toml" /etc/vietc/config.toml 2>/dev/null || true
else
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
grab = false

[app_state]
enabled = true
english_apps = ["code", "vim"]
vietnamese_apps = ["telegram", "discord", "firefox"]
EOF
    fi
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Viet+ installed successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Tray icon + universal mode shortcut
if command -v vietc-tray &>/dev/null; then
    mkdir -p /etc/xdg/autostart
    cat > /etc/xdg/autostart/vietc-tray.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Viet+ Tray
Comment=Viet+ input method status indicator
Exec=/usr/bin/vietc-tray
X-GNOME-Autostart-enabled=true
X-GNOME-Autostart-Delay=2
NoDisplay=false
EOF

    # On GNOME, the tray needs the appindicator extension to be visible.
    if [ "$DISTRO" = "ubuntu" ] && command -v gnome-shell &>/dev/null; then
        apt-get install -y gnome-shell-extension-appindicator 2>/dev/null || true
        if [ -n "${INSTALLING_USER:-}" ] && [ "$INSTALLING_USER" != "root" ]; then
            run_as_user gnome-extensions enable appindicator@rgcjonas.gmail.com 2>/dev/null || true
        fi
    fi

    # Universal mode toggle: Left Ctrl+Space -> vietcctl cycle (EN -> VNI -> TELEX).
    # Works on Wayland because it is a desktop shortcut, not a grabbed key.
    if [ "$MODE" = "bamboo" ] && [ -n "${INSTALLING_USER:-}" ] && [ "$INSTALLING_USER" != "root" ]; then
        KEYPATH="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/vietc-cycle/"
        SCHEMA="org.gnome.settings-daemon.plugins.media-keys.custom-keybinding"
        run_as_user gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "['$KEYPATH']"
        run_as_user gsettings set "$SCHEMA:$KEYPATH" name 'Viet+ cycle input mode'
        run_as_user gsettings set "$SCHEMA:$KEYPATH" command '/usr/bin/vietcctl cycle'
        run_as_user gsettings set "$SCHEMA:$KEYPATH" binding '<Primary>space'
        echo -e "${GREEN}✓ Left Ctrl+Space${NC} now cycles EN -> VNI -> TELEX (via vietcctl)."
    fi
    echo -e "${GREEN}✓ Tray icon${NC} will start on login (autostart entry installed)."
fi

if [ "$MODE" = "bamboo" ]; then
    echo -e "${GREEN}✓ Bamboo aux-controller mode${NC}"
    echo -e "  vietc switches the Bamboo IBus engine per focused app; Wayland-native"
    echo -e "  apps (ptyxis, firefox, gedit) are left to their own per-app IBus engine."
    echo -e "  One-time: focus ptyxis -> set IBus engine to BambooUs (English);"
    echo -e "  focus firefox/gedit -> Bamboo (Vietnamese)."
    echo -e "  Cycle typing style anywhere with ${GREEN}Left Ctrl+Space${NC}."
elif [ "$MODE" = "grab" ]; then
    echo -e "${GREEN}✓ Installed${NC} vietc-daemon runs as a normal user (rootless)."
    echo -e "  It uses zwp_input_method_v2 when available, else the rootless X11 path"
    echo -e "  (XQueryKeymap + XTEST over XWayland). No setcap/uinput required."
    echo ""
    echo -e "Enable auto-start (as the user, not root):"
    echo -e "  ${GREEN}systemctl --user daemon-reload${NC}"
    echo -e "  ${GREEN}systemctl --user enable --now vietc.service${NC}"
    echo ""
    echo -e "vietc will auto-start on login, stop IBus, and take over input."
    echo -e "On stop it restarts IBus. Optional UI: run ${GREEN}vietc-tray${NC} manually."
    echo ""
    echo -e "Test: type in Vietnamese in any app."
    echo -e "Toggle VN/EN: ${GREEN}Ctrl+Space${NC}  Switch VNI/Telex: ${GREEN}Ctrl+Shift${NC}"
fi
echo ""
echo -e "See ${GREEN}vietc.toml${NC} for configuration."
echo -e "Privileged fallback (evdev/uinput) is still available if neither v2 nor"
echo -e "X11/XWayland is present — see docs/wayland-rootless.md."
