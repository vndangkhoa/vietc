#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Vietnamese Input Method Uninstaller
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; NC='\033[0m'

[ "$EUID" -ne 0 ] && echo -e "${RED}Please run with sudo.${NC}" && exit 1

echo -e "${RED}=== Viet+ Uninstaller ===${NC}"

pkill -x vietc-tray 2>/dev/null || true
pkill -x vietc-daemon 2>/dev/null || true
pkill -x vietc 2>/dev/null || true

rm -f /usr/bin/vietc-daemon /usr/bin/vietc-cli /usr/bin/vietc-uinputd \
      /usr/bin/vietc-tray /usr/bin/vietc-xrecord
rm -f /usr/local/bin/vietc /usr/local/bin/vietc-daemon /usr/local/bin/vietc-cli \
      /usr/local/bin/vietc-uinputd /usr/local/bin/vietc-tray /usr/local/bin/vietc-xrecord
rm -f /etc/udev/rules.d/99-vietc.rules
rm -f /etc/vietc/config.toml
rmdir /etc/vietc 2>/dev/null || true

echo -e "${GREEN}=== Done! ===${NC}"
