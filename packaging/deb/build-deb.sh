#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="${1:-0.1.6}"
PACKAGE="vietc_${VERSION}-1_amd64"
STAGING="$SCRIPT_DIR/$PACKAGE"

echo "=== Building Viet+ .deb package v${VERSION} ==="

# Build binaries (all features: x11 + wayland)
echo "[1/5] Building binaries..."
cargo build --release --features "x11,wayland" --manifest-path "$PROJECT_ROOT/Cargo.toml"
(cd "$PROJECT_ROOT/ui" && cargo build --release)
echo "  Done."

# Clean and create staging
echo "[2/5] Creating staging directory..."
rm -rf "$STAGING"
mkdir -p "$STAGING/DEBIAN"
mkdir -p "$STAGING/usr/bin"
mkdir -p "$STAGING/usr/lib/systemd/user"
mkdir -p "$STAGING/etc/vietc"
mkdir -p "$STAGING/usr/share/applications"
mkdir -p "$STAGING/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$STAGING/usr/share/doc/vietc"
mkdir -p "$STAGING/usr/share/metainfo"
mkdir -p "$STAGING/etc/xdg/autostart"
mkdir -p "$STAGING/lib/udev/rules.d"


# Copy binaries
echo "[3/5] Installing binaries..."
cp "$PROJECT_ROOT/target/release/vietc" "$STAGING/usr/bin/vietc-daemon"
cp "$PROJECT_ROOT/target/release/vietc-cli" "$STAGING/usr/bin/"
cp "$PROJECT_ROOT/target/release/vietc-uinputd" "$STAGING/usr/bin/"
cp "$PROJECT_ROOT/ui/target/release/vietc-tray" "$STAGING/usr/bin/"

# Compile and bundle vietc-xrecord (C helper for X11 XRecord keyboard capture)
gcc -O2 -o "$STAGING/usr/bin/vietc-xrecord" "$SCRIPT_DIR/vietc-xrecord.c" -lX11 -lXtst

# Icons (main app icon + tray status icons)
cp "$PROJECT_ROOT/packaging/icons/vietc.svg" "$STAGING/usr/share/icons/hicolor/256x256/apps/"
cp "$PROJECT_ROOT/packaging/icons/vietc-vn.svg" "$STAGING/usr/share/icons/hicolor/256x256/apps/"
cp "$PROJECT_ROOT/packaging/icons/vietc-en.svg" "$STAGING/usr/share/icons/hicolor/256x256/apps/"

# Desktop file
cp "$SCRIPT_DIR/vietc.desktop" "$STAGING/usr/share/applications/"

# Udev rules
cp "$PROJECT_ROOT/packaging/99-vietc.rules" "$STAGING/lib/udev/rules.d/"


# Documentation
cp "$PROJECT_ROOT/README.md" "$STAGING/usr/share/doc/vietc/"
cp "$PROJECT_ROOT/LICENSE" "$STAGING/usr/share/doc/vietc/"

# Config
cp "$PROJECT_ROOT/vietc.toml" "$STAGING/etc/vietc/config.toml"

# Systemd user service — rootless: runs vietc-daemon directly (no tray autostart,
# which would otherwise spawn a second daemon).
cat > "$STAGING/usr/lib/systemd/user/vietc.service" << 'SERVICE'
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
SERVICE

# AppStream metadata
cat > "$STAGING/usr/share/metainfo/io.github.anomalyco.vietc.appdata.xml" << 'XML'
<?xml version="1.0" encoding="UTF-8"?>
<component type="console-application">
  <id>io.github.anomalyco.vietc</id>
  <name>Viet+</name>
  <summary>Vietnamese Input Method for Linux</summary>
  <description>
    <p>Zero-configuration Vietnamese input method engine supporting Telex and VNI input methods. Runs rootless as a normal user — native Wayland via zwp_input_method_v2, or the rootless X11 path (XQueryKeymap + XTEST) over XWayland. No root, setcap, or uinput required.</p>
  </description>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <url type="homepage">https://github.com/anomalyco/vietc</url>
  <provides><binary>vietc</binary></provides>
  <categories><category>Utility</category></categories>
</component>
XML

# Lintian overrides
mkdir -p "$STAGING/usr/share/lintian/overrides"
cat > "$STAGING/usr/share/lintian/overrides/vietc" << 'LINTIAN'
# Binaries are intentionally placed in /usr/bin without man pages
# as Viet+ is a modern GUI application targeting Wayland/X11.
vietc: no-manual-page *
# Init script helpers are not needed; Viet+ uses systemd --user units.
vietc: no-systemd-service-file-outside-lib
# We bundle the tray alongside the daemon for convenience.
vietc: wrong-section-according-to-package-name
LINTIAN

# Create control file
echo "[4/5] Creating control file..."
cat > "$STAGING/DEBIAN/control" << 'CONTROL'
Package: vietc
Version: VERSION_PLACEHOLDER
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.31), libevdev2 (>= 1.9.0)
Recommends: libwayland-client0 (>= 1.20), libx11-6, libxtst6, libdbus-1-3, xclip, wl-clipboard
Maintainer: Khoa Vo <vndangkhoa@gmail.com>
Description: Viet+ — Vietnamese Input Method for Linux
 Zero-configuration Vietnamese input method engine supporting
 Telex and VNI input methods. Runs rootless as a normal user —
 native Wayland (zwp_input_method_v2) or the rootless X11 path
 over XWayland. No root, setcap, or uinput required.
CONTROL
sed -i "s/VERSION_PLACEHOLDER/$VERSION/" "$STAGING/DEBIAN/control"

# Conffiles
echo "/etc/vietc/config.toml" > "$STAGING/DEBIAN/conffiles"

# Maintainer scripts
cat > "$STAGING/DEBIAN/postinst" << 'POSTINST'
#!/bin/sh
set -e

show_popup() {
  local user="$1" msg="$2"
  local display="${DISPLAY:-:0}"
  local xauth=""
  if [ -n "$user" ]; then
    local home
    home="$(getent passwd "$user" 2>/dev/null | cut -d: -f6 || true)"
    if [ -n "$home" ]; then
      xauth="$home/.Xauthority"
    fi
  fi
  # Try zenity (modal dialog)
  if command -v zenity >/dev/null 2>&1 && [ -n "$user" ]; then
    su "$user" -c "DISPLAY='$display' XAUTHORITY='$xauth' \
      zenity --info --title='Viet+' --text='$msg' --width=400" 2>/dev/null || true
  fi
  # Also try notify-send (desktop notification)
  if command -v notify-send >/dev/null 2>&1 && [ -n "$user" ]; then
    su "$user" -c "DISPLAY='$display' XAUTHORITY='$xauth' \
      notify-send 'Viet+' '$msg' -t 10000 -i vietc" 2>/dev/null || true
  fi
}

cleanup_old_install() {
  # Remove old binaries from /usr/local/bin/ (shadowed the new /usr/bin/ ones)
  rm -f /usr/local/bin/vietc-tray /usr/local/bin/vietc /usr/local/bin/vietc-daemon \
        /usr/local/bin/vietc-cli /usr/local/bin/vietc-uinputd /usr/local/bin/vietc-xrecord 2>/dev/null || true

  # Clean old local user binaries & autostart to prevent shadowing the new system-wide ones
  local installing_user="${SUDO_USER:-${USER:-}}"
  if [ -n "$installing_user" ] && [ "$installing_user" != "root" ]; then
    local user_home
    user_home="$(getent passwd "$installing_user" 2>/dev/null | cut -d: -f6 || true)"
    if [ -n "$user_home" ]; then
      rm -f "$user_home/.local/bin/vietc" "$user_home/.local/bin/vietc-daemon" \
             "$user_home/.local/bin/vietc-cli" "$user_home/.local/bin/vietc-uinputd" \
             "$user_home/.local/bin/vietc-tray" "$user_home/.local/bin/vietc-xrecord" \
             "$user_home/.local/bin/vietc-start" 2>/dev/null || true
      rm -f "$user_home/.config/autostart/vietc.desktop" 2>/dev/null || true
    fi
  fi
}

case "$1" in
  configure)
    # Kill old running daemon/tray so new binaries take effect
    pkill -x vietc-tray 2>/dev/null || true
    pkill -x vietc-daemon 2>/dev/null || true
    pkill -x vietc 2>/dev/null || true

    # Remove old /usr/local/bin/ binaries that shadowed the new ones
    cleanup_old_install

    # Reload systemd
    if command -v systemctl >/dev/null 2>&1; then
      systemctl --global daemon-reload >/dev/null 2>&1 || true
    fi

    # Add installing user to input group (needed for /dev/uinput access)
    INSTALLING_USER="${SUDO_USER:-${USER:-}}"
    if [ -n "$INSTALLING_USER" ] && [ "$INSTALLING_USER" != "root" ]; then
      if ! groups "$INSTALLING_USER" 2>/dev/null | grep -qw input; then
        adduser "$INSTALLING_USER" input 2>/dev/null || true
      fi
      # Remove stale user config from previous installs
      USER_HOME="$(getent passwd "$INSTALLING_USER" 2>/dev/null | cut -d: -f6 || true)"
      if [ -n "$USER_HOME" ]; then
        rm -f "$USER_HOME/.config/vietc/config.toml" 2>/dev/null || true
        rm -f "$USER_HOME/.config/vietc/overrides.toml" 2>/dev/null || true
        rm -f "$USER_HOME/.config/vietc/.first-launch-done" 2>/dev/null || true
      fi

      # Show popup
      show_popup "$INSTALLING_USER" \
        "Viet+ installed! Please LOG OUT and LOG BACK IN to start typing Vietnamese."
    fi

    # Update icon cache so the app icon appears in the menu
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
      gtk-update-icon-cache -f /usr/share/icons/hicolor/ >/dev/null 2>&1 || true
    fi

    # Reload udev rules to apply the new uinput rule
    if command -v udevadm >/dev/null 2>&1; then
      udevadm control --reload-rules >/dev/null 2>&1 || true
      udevadm trigger --subsystem-match=misc >/dev/null 2>&1 || true
    fi

    ;;
esac
POSTINST
chmod 755 "$STAGING/DEBIAN/postinst"

cat > "$STAGING/DEBIAN/prerm" << 'PRERM'
#!/bin/sh
set -e
case "$1" in
  remove|upgrade|deconfigure)
    if command -v systemctl >/dev/null 2>&1; then
      systemctl --global daemon-reload >/dev/null 2>&1 || true
    fi
    ;;
esac
PRERM
chmod 755 "$STAGING/DEBIAN/prerm"

cat > "$STAGING/DEBIAN/postrm" << 'POSTRM'
#!/bin/sh
set -e
case "$1" in
  purge)
    rm -rf /etc/vietc 2>/dev/null || true
    ;;
esac
POSTRM
chmod 755 "$STAGING/DEBIAN/postrm"

# Build .deb
echo "[5/5] Building .deb package..."
fakeroot dpkg-deb --build "$STAGING" "$SCRIPT_DIR/vietc_${VERSION}-1_amd64.deb"
echo ""
echo "=== Package built: packaging/deb/vietc_${VERSION}-1_amd64.deb ==="
