#!/usr/bin/env bash
set -euo pipefail

# Ensure cargo is in PATH
if ! command -v cargo &>/dev/null; then
    if [ -f "$HOME/.cargo/bin/cargo" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
APPDIR="$SCRIPT_DIR/AppDir"
VERSION="${1:-0.1.0}"

echo "=== Building Viet+ AppImage v${VERSION} ==="

# Clean
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/doc/vietc"
mkdir -p "$APPDIR/etc/vietc"
mkdir -p "$APPDIR/usr/lib/systemd/user"
mkdir -p "$APPDIR/usr/share/metainfo"

# Build binaries
echo "[1/5] Building binaries..."
if [ ! -f "target/release/vietc" ]; then
    cargo build --release
    cd "$PROJECT_ROOT/ui" && cargo build --release && cd "$PROJECT_ROOT"
fi
echo "  Built with x11 + wayland"

# Copy binaries from deb-build if they exist, otherwise from target/release
echo "[2/5] Installing binaries..."
if [ -d "deb-build/usr/bin" ]; then
    cp -r deb-build/usr/bin/* "$APPDIR/usr/bin/"
else
    cp target/release/vietc "$APPDIR/usr/bin/"
    cp target/release/vietc-cli "$APPDIR/usr/bin/"
    [ -f ui/target/release/vietc-tray ] && cp ui/target/release/vietc-tray "$APPDIR/usr/bin/"
fi

# Bundle xclip as fallback for clipboard operations
echo "  Bundling xclip..."
if command -v xclip &>/dev/null; then
    cp "$(which xclip)" "$APPDIR/usr/bin/"
    echo "  xclip bundled"
else
    echo "  xclip not found on system, skipping"
fi

# Desktop integration
echo "[3/5] Installing desktop integration..."
if [ -f "deb-build/vietc.desktop" ]; then
    cp deb-build/vietc.desktop "$APPDIR/usr/share/applications/"
else
    cp "$SCRIPT_DIR/vietc.desktop" "$APPDIR/usr/share/applications/"
fi

# Icons
if [ -f "deb-build/vietc.svg" ]; then
    cp deb-build/vietc.svg "$APPDIR/usr/share/icons/hicolor/256x256/apps/"
    cp deb-build/vietc.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/"
    cp deb-build/vietc.png "$APPDIR/"
fi

# AppStream metadata
if [ -f "deb-build/usr/share/metainfo/io.github.anomalyco.vietc.appdata.xml" ]; then
    cp deb-build/usr/share/metainfo/io.github.anomalyco.vietc.appdata.xml "$APPDIR/usr/share/metainfo/"
else
    cat > "$APPDIR/usr/share/metainfo/io.github.anomalyco.vietc.appdata.xml" << 'XML'
<?xml version="1.0" encoding="UTF-8"?>
<component type="console-application">
  <id>io.github.anomalyco.vietc</id>
  <name>Viet+</name>
  <summary>Vietnamese Input Method for Linux</summary>
  <description>
    <p>Zero-configuration Vietnamese input method engine supporting Telex and VNI input methods. Works natively on both X11 and Wayland via evdev uinput injection.</p>
  </description>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <url type="homepage">https://github.com/anomalyco/vietc</url>
  <provides><binary>vietc</binary></provides>
  <categories><category>Utility</category></categories>
</component>
XML
fi

# Config
echo "[4/5] Installing config..."
if [ -f "deb-build/etc/vietc/config.toml" ]; then
    cp deb-build/etc/vietc/config.toml "$APPDIR/etc/vietc/"
else
    sed 's/^grab = false/grab = true/' "$PROJECT_ROOT/vietc.toml" > "$APPDIR/etc/vietc/config.toml"
fi

# Docs
if [ -f "deb-build/usr/share/doc/vietc/README.md" ]; then
    cp deb-build/usr/share/doc/vietc/README.md "$APPDIR/usr/share/doc/vietc/"
else
    cp "$PROJECT_ROOT/README.md" "$APPDIR/usr/share/doc/vietc/"
fi

# Systemd service
if [ -f "deb-build/usr/lib/systemd/user/vietc.service" ]; then
    cp deb-build/usr/lib/systemd/user/vietc.service "$APPDIR/usr/lib/systemd/user/"
else
    cp "$PROJECT_ROOT/vietc.service" "$APPDIR/usr/lib/systemd/user/"
fi

# Desktop file in AppDir root
if [ -f "deb-build/vietc.desktop" ]; then
    cp deb-build/vietc.desktop "$APPDIR/"
else
    cp "$APPDIR/usr/share/applications/vietc.desktop" "$APPDIR/"
fi

# Icon — required by appimagetool (desktop file has Icon=vietc)
# Use SVG from deb build if available, otherwise generate a keyboard icon
if [ -f "deb-build/usr/share/icons/hicolor/256x256/apps/vietc.svg" ]; then
    cp "deb-build/usr/share/icons/hicolor/256x256/apps/vietc.svg" "$APPDIR/vietc.svg"
elif [ -f "deb-build/usr/share/icons/hicolor/256x256/apps/vietc.png" ]; then
    cp "deb-build/usr/share/icons/hicolor/256x256/apps/vietc.png" "$APPDIR/vietc.png"
else
    # Generate a proper keyboard+VN icon as SVG
    cat > "$APPDIR/vietc.svg" << 'SVGEOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <rect x="20" y="60" width="216" height="140" rx="16" fill="#2d2d2d" stroke="#1a1a1a" stroke-width="4"/>
  <rect x="36" y="76" width="184" height="108" rx="8" fill="#3d3d3d"/>
  <rect x="48" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="78" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="108" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="138" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="168" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="198" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="54" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="84" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="114" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="144" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="174" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="60" y="140" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="90" y="140" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="120" y="140" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="150" y="140" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="180" y="140" width="42" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="72" y="166" width="112" height="16" rx="3" fill="#f0f0f0"/>
  <circle cx="216" cy="48" r="28" fill="#da251d"/>
  <text x="216" y="56" text-anchor="middle" fill="white" font-size="18" font-weight="bold" font-family="sans-serif">VN</text>
</svg>
SVGEOF
fi

# Convert SVG to PNG for appimagetool (it prefers PNG for the root icon)
if [ -f "$APPDIR/vietc.svg" ] && ! [ -f "$APPDIR/vietc.png" ]; then
    if command -v rsvg-convert &>/dev/null; then
        rsvg-convert -w 256 -h 256 "$APPDIR/vietc.svg" -o "$APPDIR/vietc.png"
    elif command -v inkscape &>/dev/null; then
        inkscape -w 256 -h 256 "$APPDIR/vietc.svg" --export-filename="$APPDIR/vietc.png" 2>/dev/null
    elif command -v convert &>/dev/null; then
        convert -background none "$APPDIR/vietc.svg" -resize 256x256 "$APPDIR/vietc.png" 2>/dev/null
    elif command -v python3 &>/dev/null; then
        python3 -c "
import subprocess, sys
try:
    subprocess.check_call(['rsvg-convert', '-w', '256', '-h', '256', '$APPDIR/vietc.svg', '-o', '$APPDIR/vietc.png'])
except Exception:
    pass
" 2>/dev/null
    fi
    # If no converter, appimagetool can use SVG directly
fi

# Also put icon in hicolor for system installs via AppImage
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
[ -f "$APPDIR/vietc.svg" ] && cp "$APPDIR/vietc.svg" "$APPDIR/usr/share/icons/hicolor/256x256/apps/"
[ -f "$APPDIR/vietc.png" ] && cp "$APPDIR/vietc.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/"

# Create custom AppRun script
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"

# Export our bin dir on PATH so child processes can find sibling binaries
export PATH="$HERE/usr/bin:$PATH"

# Build display env prefix for elevation commands.
# Capture from current user env (DISPLAY, XAUTHORITY, WAYLAND_DISPLAY, XDG_RUNTIME_DIR)
# so they are available inside the root daemon. Without this, xdotool/xclip/wtype
# fail silently because sudo/pkexec strip display env vars.
ENV_PREFIX="env"
[ -n "$DISPLAY" ]           && ENV_PREFIX="$ENV_PREFIX DISPLAY=$DISPLAY"
[ -n "$XAUTHORITY" ]        && ENV_PREFIX="$ENV_PREFIX XAUTHORITY=$XAUTHORITY"
[ -n "$WAYLAND_DISPLAY" ]   && ENV_PREFIX="$ENV_PREFIX WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
[ -n "$XDG_RUNTIME_DIR" ]   && ENV_PREFIX="$ENV_PREFIX XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR"

# Start daemon (kill old non-root one first if we have root)
# On X11 we can run without root (XGrabKeyboard + XTest injection needs no special permissions).
# On Wayland, evdev requires root (input group) or uinput.
NEED_ROOT=""
if [ -n "$WAYLAND_DISPLAY" ]; then
    NEED_ROOT="yes"
fi

if [ -z "$NEED_ROOT" ]; then
    # X11: no root needed
    pkill -x vietc 2>/dev/null; sleep 0.3
    "$HERE/usr/bin/vietc" >/dev/null &
    DAEMON_PID=$!
else
    # Fix Wayland env for root: sudo resets XDG_RUNTIME_DIR, breaking wtype/wl-copy.
    if [ "$(id -u)" = "0" ] && [ -z "$XDG_RUNTIME_DIR" ] && [ -n "$SUDO_USER" ]; then
        USER_UID=$(id -u "$SUDO_USER" 2>/dev/null || echo 1000)
        export XDG_RUNTIME_DIR="/run/user/$USER_UID"
        if [ -d "/run/user/$USER_UID" ] && ls "/run/user/$USER_UID/wayland-*" >/dev/null 2>&1; then
            export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"
        fi
    fi

    if command -v pkexec >/dev/null; then
        pkill -x vietc 2>/dev/null; sleep 0.5
        pkexec $ENV_PREFIX "$HERE/usr/bin/vietc" >/dev/null &
        DAEMON_PID=$!
    elif [ -n "$WAYLAND_DISPLAY" ]; then
        password=""
        if command -v kdialog >/dev/null; then
            password=$(kdialog --password "Viet+ needs root privileges to grab the keyboard.") || password=""
        elif command -v zenity >/dev/null; then
            password=$(zenity --password --title="Viet+ needs root") || password=""
        elif command -v ssh-askpass >/dev/null; then
            password=$(ssh-askpass "Viet+ needs root privileges") || password=""
        fi
        if [ -n "$password" ]; then
            pkill -x vietc 2>/dev/null; sleep 0.5
            echo "$password" | sudo -S $ENV_PREFIX "$HERE/usr/bin/vietc" >/dev/null &
            DAEMON_PID=$!
        fi
    elif command -v sudo >/dev/null; then
        pkill -x vietc 2>/dev/null; sleep 0.5
        sudo $ENV_PREFIX "$HERE/usr/bin/vietc" >/dev/null &
        DAEMON_PID=$!
    fi
fi

if [ -z "$DAEMON_PID" ] && ! pgrep -x vietc >/dev/null; then
    "$HERE/usr/bin/vietc" >/dev/null &
    DAEMON_PID=$!
fi

# Keep the AppImage alive with a tray or settings UI.
# Run as a child (not exec) so daemon cleanup works on exit.
cleanup_daemon() {
    if [ -n "$DAEMON_PID" ]; then
        kill "$DAEMON_PID" 2>/dev/null
        wait "$DAEMON_PID" 2>/dev/null
    fi
}
trap cleanup_daemon EXIT INT TERM

if [ -f "$HERE/usr/bin/vietc-tray" ]; then
    "$HERE/usr/bin/vietc-tray" "$@"
else
    echo "[vietc] Daemon running in foreground. Press Ctrl+C to stop."
    wait "$DAEMON_PID"
fi
EOF
chmod +x "$APPDIR/AppRun"

echo "[5/5] AppDir ready at: $APPDIR"
echo ""

# Auto build if appimagetool exists
if [ -f "$SCRIPT_DIR/appimagetool" ]; then
    echo "=== Running appimagetool FUSE build ==="
    ARCH=x86_64 "$SCRIPT_DIR/appimagetool" --appimage-extract-and-run "$APPDIR" "$SCRIPT_DIR/Viet+-${VERSION}-x86_64.AppImage"
elif command -v appimagetool &>/dev/null; then
    echo "=== Running system appimagetool ==="
    ARCH=x86_64 appimagetool "$APPDIR" "$SCRIPT_DIR/Viet+-${VERSION}-x86_64.AppImage"
else
    echo "To build AppImage:"
    echo "  appimagetool $APPDIR Viet+-${VERSION}-x86_64.AppImage"
fi
