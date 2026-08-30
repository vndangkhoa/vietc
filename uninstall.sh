#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Vietnamese Input Method Uninstaller
# Usage: curl -fsSL <url> | bash   OR   sudo ./uninstall.sh
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'

# Language: auto-detect from $LANG, override with --lang vi|en
LANG_CODE="vi"
case "${LANG:-}" in
  en*|C*|POSIX*) LANG_CODE="en" ;;
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

# t KEY — print translated message
t() {
    local key="$1"
    if [ "$LANG_CODE" = "vi" ]; then
        case "$key" in
            sudo) echo -e "${RED}Vui lòng chạy với quyền sudo.${NC}" ;;
            uninstaller_header) echo -e "${YELLOW}=== Bắt đầu gỡ cài đặt Viet+ (VietC) ===${NC}" ;;
            stopping_services) echo -e "Đang dừng các dịch vụ và tiến trình Viet+..." ;;
            removing_files) echo -e "Đang xóa các tệp nhị phân, cấu hình và icon..." ;;
            removed) echo -e "${GREEN}=== Đã gỡ cài đặt hoàn toàn Viet+ khỏi hệ thống! ===${NC}" ;;
            *) echo -e "$key" ;;
        esac
    else
        case "$key" in
            sudo) echo -e "${RED}Please run with sudo.${NC}" ;;
            uninstaller_header) echo -e "${YELLOW}=== Starting Viet+ (VietC) Uninstaller ===${NC}" ;;
            stopping_services) echo -e "Stopping Viet+ services and processes..." ;;
            removing_files) echo -e "Removing binaries, configuration, and icons..." ;;
            removed) echo -e "${GREEN}=== Viet+ has been completely removed from your system! ===${NC}" ;;
            *) echo -e "$key" ;;
        esac
    fi
}

# Auto-elevate with sudo if not root
if [ "$EUID" -ne 0 ]; then
    if command -v sudo >/dev/null 2>&1; then
        echo -e "${YELLOW}Viet+ uninstaller requires sudo permissions.${NC}"
        if [ -f "$0" ]; then
            exec sudo bash "$0" "$@"
        else
            exec curl -fsSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash -s -- "$@"
        fi
    else
        t sudo
        exit 1
    fi
fi

t uninstaller_header

INSTALLING_USER="${SUDO_USER:-$USER}"
if [ -z "$INSTALLING_USER" ] || [ "$INSTALLING_USER" = "root" ]; then
    if command -v logname &>/dev/null; then
        INSTALLING_USER="$(logname 2>/dev/null || true)"
    fi
fi

USER_HOME=""
U_UID=""
if [ -n "$INSTALLING_USER" ] && [ "$INSTALLING_USER" != "root" ]; then
    USER_HOME="$(getent passwd "$INSTALLING_USER" 2>/dev/null | cut -d: -f6 || true)"
    U_UID="$(id -u "$INSTALLING_USER" 2>/dev/null || true)"
fi

t stopping_services

# Stop and disable systemd user services
if [ -n "$INSTALLING_USER" ] && [ -n "$U_UID" ]; then
    sudo -u "$INSTALLING_USER" XDG_RUNTIME_DIR="/run/user/$U_UID" DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$U_UID/bus" \
        systemctl --user stop vietc.service vietc-tray.service 2>/dev/null || true
    sudo -u "$INSTALLING_USER" XDG_RUNTIME_DIR="/run/user/$U_UID" DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$U_UID/bus" \
        systemctl --user disable vietc.service vietc-tray.service 2>/dev/null || true
fi

# Kill all running processes
pkill -9 -x vietc 2>/dev/null || true
pkill -9 -x vietc-daemon 2>/dev/null || true
pkill -9 -x vietc-tray 2>/dev/null || true
pkill -9 -x vietcctl 2>/dev/null || true
pkill -9 -x vietc-uinputd 2>/dev/null || true
pkill -9 -x vietc-cli 2>/dev/null || true

t removing_files

# Remove system binaries
rm -f /usr/bin/vietc /usr/bin/vietc-daemon /usr/bin/vietcctl /usr/bin/vietc-tray \
      /usr/bin/vietc-uinputd /usr/bin/vietc-cli /usr/bin/vietc-xrecord
rm -f /usr/local/bin/vietc /usr/local/bin/vietc-daemon /usr/local/bin/vietcctl \
      /usr/local/bin/vietc-tray /usr/local/bin/vietc-uinputd /usr/local/bin/vietc-cli \
      /usr/local/bin/vietc-xrecord

# Remove user local binaries if present
if [ -n "$USER_HOME" ]; then
    rm -f "$USER_HOME/.local/bin/vietc" "$USER_HOME/.local/bin/vietc-daemon" \
          "$USER_HOME/.local/bin/vietcctl" "$USER_HOME/.local/bin/vietc-tray" \
          "$USER_HOME/.local/bin/vietc-uinputd" "$USER_HOME/.local/bin/vietc-cli"
fi

# Remove systemd service files
rm -f /usr/lib/systemd/user/vietc.service /usr/lib/systemd/user/vietc-tray.service
rm -f /etc/systemd/user/vietc.service /etc/systemd/user/vietc-tray.service
if [ -n "$USER_HOME" ]; then
    rm -f "$USER_HOME/.config/systemd/user/vietc.service" "$USER_HOME/.config/systemd/user/vietc-tray.service"
fi

# Remove udev rules
rm -f /etc/udev/rules.d/99-vietc.rules /etc/udev/rules.d/99-uinput.rules

# Remove IBus component
rm -f /usr/share/ibus/component/vietc.xml

# Remove system desktop autostart & icons
rm -f /usr/share/applications/vietc*.desktop /etc/xdg/autostart/vietc*.desktop
rm -f /usr/share/icons/hicolor/*/apps/vietc*.svg /usr/share/icons/vietc*.svg
gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true

# Remove user configs, user autostart & icons
rm -rf /etc/vietc
if [ -n "$USER_HOME" ]; then
    rm -rf "$USER_HOME/.config/vietc"
    rm -f "$USER_HOME/.config/autostart/vietc*.desktop"
    rm -f "$USER_HOME/.local/share/icons/vietc*.svg"
    rm -f "$USER_HOME/.local/share/icons/hicolor/*/apps/vietc*.svg"
fi

# Clean up status and socket files
rm -f /tmp/vietc* /tmp/vietc-status*

# Reload udev
udevadm control --reload-rules 2>/dev/null || true

# Reload systemd
if [ -n "$INSTALLING_USER" ] && [ -n "$U_UID" ]; then
    sudo -u "$INSTALLING_USER" XDG_RUNTIME_DIR="/run/user/$U_UID" DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$U_UID/bus" \
        systemctl --user daemon-reload 2>/dev/null || true
fi
if command -v systemctl &>/dev/null; then
    systemctl --global daemon-reload 2>/dev/null || true
fi

t removed

