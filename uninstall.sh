#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Vietnamese Input Method Uninstaller
# Usage: curl -sSL <url> | sudo bash
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; NC='\033[0m'

# Language: auto-detect from $LANG, override with --lang vi|en
LANG_CODE="en"
case "${LANG:-}" in
  vi*|*_VN|*VN*) LANG_CODE="vi" ;;
esac
LANG_NEXT=0
for arg in "$@"; do
    if [ "$LANG_NEXT" = "1" ]; then
        case "$arg" in
            vi|vi_VN|vi-VN) LANG_CODE="vi" ;;
            en) LANG_CODE="en" ;;
        esac
        LANG_NEXT=0
    elif [ "$arg" = "--lang" ]; then
        LANG_NEXT=1
    elif [ "$arg" = "--lang=vi" ] || [ "$arg" = "--lang=vi_VN" ] || [ "$arg" = "--lang=vi-VN" ]; then
        LANG_CODE="vi"
    elif [ "$arg" = "--lang=en" ]; then
        LANG_CODE="en"
    fi
done

# t KEY — print the translated (Vietnamese) or English message.
t() {
    local key="$1"
    if [ "$LANG_CODE" = "vi" ]; then
        case "$key" in
            sudo) echo -e "${RED}Vui lòng chạy với quyền sudo.${NC}" ;;
            uninstaller_header) echo -e "${RED}=== Gỡ cài đặt Viet+ ===${NC}" ;;
            removed) echo -e "${GREEN}=== Đã gỡ cài đặt Viet+ ===${NC}" ;;
            *) echo -e "$key" ;;
        esac
    else
        case "$key" in
            sudo) echo -e "${RED}Please run with sudo.${NC}" ;;
            uninstaller_header) echo -e "${RED}=== Viet+ Uninstaller ===${NC}" ;;
            removed) echo -e "${GREEN}=== Viet+ removed ===${NC}" ;;
            *) echo -e "$key" ;;
        esac
    fi
}

[ "$EUID" -ne 0 ] && t sudo && exit 1

t uninstaller_header

# Kill running processes
pkill -x vietc-tray 2>/dev/null || true
pkill -x vietc-daemon 2>/dev/null || true
pkill -x vietc-uinputd 2>/dev/null || true
pkill -x vietcctl 2>/dev/null || true
pkill -x vietc 2>/dev/null || true

# Remove binaries
rm -f /usr/bin/vietc-daemon /usr/bin/vietc-cli /usr/bin/vietc-uinputd \
      /usr/bin/vietc-tray /usr/bin/vietc-xrecord /usr/bin/vietcctl
rm -f /usr/local/bin/vietc /usr/local/bin/vietc-daemon /usr/local/bin/vietc-cli \
      /usr/local/bin/vietc-uinputd /usr/local/bin/vietc-tray /usr/local/bin/vietc-xrecord \
      /usr/local/bin/vietcctl

# Remove udev rules
rm -f /etc/udev/rules.d/99-vietc.rules

# Remove config
rm -rf /etc/vietc

# Remove systemd service
rm -f /usr/lib/systemd/user/vietc.service

# Remove icons
rm -f /usr/share/icons/hicolor/256x256/apps/vietc*.svg

# Remove desktop file
rm -f /usr/share/applications/vietc.desktop
rm -f /etc/xdg/autostart/vietc-tray.desktop

# Remove the universal mode-toggle keybinding and per-user mode file (best effort)
INSTALLING_USER="${SUDO_USER:-}"
if [ -z "$INSTALLING_USER" ] && command -v logname &>/dev/null; then
    INSTALLING_USER="$(logname 2>/dev/null || true)"
fi
if [ -n "$INSTALLING_USER" ] && [ "$INSTALLING_USER" != "root" ]; then
    USER_HOME="$(getent passwd "$INSTALLING_USER" 2>/dev/null | cut -d: -f6 || true)"
    U_UID="$(id -u "$INSTALLING_USER" 2>/dev/null)"
    if [ -n "$USER_HOME" ]; then
        rm -f "$USER_HOME/.config/vietc/mode"
    fi
    if [ -n "$U_UID" ] && command -v gsettings &>/dev/null; then
        sudo -u "$INSTALLING_USER" XDG_RUNTIME_DIR="/run/user/$U_UID" \
            DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$U_UID/bus" \
            gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "[]" \
            2>/dev/null || true
    fi
fi

# Reload udev
udevadm control --reload-rules 2>/dev/null || true

# Reload systemd user daemon
if command -v systemctl &>/dev/null; then
    systemctl --global daemon-reload 2>/dev/null || true
fi

t removed
