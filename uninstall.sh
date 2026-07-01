#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Vietnamese Input Method Uninstaller
set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${RED}=== Viet+ Uninstallation Script ===${NC}"

if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root (or with sudo).${NC}"
    exit 1
fi

echo "Stopping any running Viet+ processes..."
pkill -x vietc-tray 2>/dev/null || true
pkill -x vietc-daemon 2>/dev/null || true
pkill -x vietc-uinputd 2>/dev/null || true
pkill -x vietc 2>/dev/null || true

echo "Removing binaries..."
rm -f /usr/bin/vietc-daemon /usr/bin/vietc-cli /usr/bin/vietc-uinputd /usr/bin/vietc-tray /usr/bin/vietc-xrecord
rm -f /usr/local/bin/vietc-daemon /usr/local/bin/vietc-cli /usr/local/bin/vietc-uinputd /usr/local/bin/vietc-tray /usr/local/bin/vietc-xrecord /usr/local/bin/vietc

echo "Removing icons..."
rm -f /usr/share/icons/hicolor/256x256/apps/vietc.svg
rm -f /usr/share/icons/hicolor/256x256/apps/vietc-vn.svg
rm -f /usr/share/icons/hicolor/256x256/apps/vietc-en.svg

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor/ >/dev/null 2>&1 || true
fi

echo "Removing desktop files and autostart..."
rm -f /usr/share/applications/vietc.desktop
rm -f /etc/xdg/autostart/vietc-tray.desktop

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
fi

echo "Removing systemd service..."
rm -f /usr/lib/systemd/user/vietc.service

echo "Removing udev rules..."
rm -f /etc/udev/rules.d/99-vietc.rules
if command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules >/dev/null 2>&1 || true
    udevadm trigger --subsystem-match=misc >/dev/null 2>&1 || true
fi

echo "Removing global configuration..."
rm -rf /etc/vietc

if command -v systemctl >/dev/null 2>&1; then
    systemctl --global daemon-reload >/dev/null 2>&1 || true
fi

echo -e "\n${GREEN}=== Uninstallation Completed Successfully! ===${NC}"
