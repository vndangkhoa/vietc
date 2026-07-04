<p align="center">
  <img src="https://img.shields.io/badge/Platform-Linux-blue?style=for-the-badge" alt="Platform">
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/Version-0.1.18-purple?style=for-the-badge" alt="Version">
  <img src="https://img.shields.io/badge/Tests-108_passing-brightgreen?style=for-the-badge" alt="Tests">
  <img src="https://img.shields.io/badge/Event_Sourcing-✓-blueviolet?style=for-the-badge" alt="Event Sourcing">
</p>

<h1 align="center">
  <br>
  Viet+
  <br>
</h1>

<p align="center">
  <b>Vietnamese Input Method for Linux</b><br>
  <sub>Zero underline &bull; No pre-edit buffer &bull; Backspace-Replay sync &bull; Built in Rust</sub>
</p>

<p align="center">
  <a href="README.vi.md">Tiếng Việt</a>
</p>

---

## What is Viet+?

Viet+ is a Vietnamese input method for Linux that takes a fundamentally different approach from every other IME: **Direct Input**.

Most Vietnamese IMEs use a **pre-edit buffer** — you type into a temporary buffer with an ugly underline, and the text only becomes real Vietnamese when you commit it. This causes duplicate text, underline distraction, broken copy/paste, and desync between the engine state and what's on screen.

Viet+ eliminates all of this. Keystrokes are **instantly converted to Unicode** — what you type is what you see. No buffer. No underline. No duplication.

---

## Features

| Feature | How It Works |
|---------|-------------|
| **Direct Input** | No pre-edit buffer. Keystrokes instantly become text via uinput injection |
| **VNI & Telex** | Both input methods fully supported, switchable at runtime via Ctrl+Shift |
| **Bamboo Engine** | Transformation model — composition, marks, tones, flexible backtracking |
| **Smart Clusters** | `uo→ươ` with backtrack, `ua→ưa` horn placement |
| **Macro Expansion** | `ko → không`, `dc → được`, add your own |
| **Casing Preservation** | `Tieengs → Tiếng`, `TIEENGS → TIẾNG` |
| **App Memory** | Per-app Vietnamese/English state, saved to `overrides.toml` |
| **Hot Reload** | Config changes apply without restart |
| **Window-Switch Reset** | Engine clears automatically on Alt+Tab |
| **CPU Priority** | Pinned to P-cores (0-3) + nice(-10) for low-latency input |
| **Uinput Injection** | `/dev/uinput` for reliable injection on X11 and Wayland |
| **Terminal Support** | ✅ Works in all major terminals: kitty, alacritty, gnome-terminal, konsole, foot, wezterm, st, urxvt, xterm |
| **Password Auto-Detection** | 4 layers: AT-SPI2 → sudo process → window-title → window-class |
| **Tray Icon** | Shows current mode: Red VN / Blue TLX / Gray EN |
| **GNOME/Wayland** | Native GNOME Shell D-Bus integration |

---

## Input Methods

Both **VNI** and **Telex** are fully supported. Switch via **Ctrl+LeftShift** or the tray menu.

### VNI

| Key | Result | Example |
|-----|--------|---------|
| `1` | á (sắc) | `a1` → `á` |
| `2` | à (huyền) | `a2` → `à` |
| `3` | ả (hỏi) | `a3` → `ả` |
| `4` | ã (ngã) | `a4` → `ã` |
| `5` | ạ (nặng) | `a5` → `ạ` |
| `6` | â/ê/ô | `a6→â`, `e6→ê`, `o6→ô` |
| `7` | ơ/ư | `o7→ơ`, `u7→ư` |
| `8` | ă | `a8→ă` |
| `9` | đ | `d9→đ` |

### Telex

| Key | Result | Example |
|-----|--------|---------|
| `s` | á (sắc) | `as→á` |
| `f` | à (huyền) | `af→à` |
| `r` | ả (hỏi) | `ar→ả` |
| `x` | ã (ngã) | `ax→ã` |
| `j` | ạ (nặng) | `aj→ạ` |
| `aa` | â | `aa→â` |
| `ee` | ê | `ee→ê` |
| `oo` | ô | `oo→ô` |
| `ow` | ơ | `ow→ơ` |
| `aw` | ă | `aw→ă` |
| `uw` | ư | `uw→ư` |
| `dd` | đ | `dd→đ` |
| `w` | ươ | `chuongw→chương` |

---

## Key Bindings

| Combo | Action |
|-------|--------|
| **Ctrl+Space** | Toggle Vietnamese ON/OFF |
| **Ctrl+LeftShift** | Toggle VNI ↔ Telex |

---

## Password Detection

4-layer automatic detection. When a password field is detected, Vietnamese is automatically disabled:

| Layer | Method | Detects |
|-------|--------|---------|
| 1 | AT-SPI2 D-Bus (a11y role check) | Password fields in accessible apps |
| 2 | Process tree (pstree) | `sudo` / `passwd` in terminal |
| 3 | Window title keywords | `password`, `sudo` in title |
| 4 | Window class matching | pinentry, polkit, kwallet dialogs |

---

## Distro Support

| Tier | Distro | Install Method | Status |
|------|--------|---------------|--------|
| ✅ **Supported** | Ubuntu, Debian, Linux Mint, Pop!_OS, elementary OS, Zorin, Neon | `apt` (auto-detected) | Tested, one-command install |
| ✅ **Supported** | Fedora, RHEL, CentOS | `dnf` (auto-detected) | Tested, one-command install |
| ✅ **Supported** | Arch, Manjaro | `pacman` (auto-detected) | Tested, one-command install |
| ⚠️ **Might support** | openSUSE, Solus, Void | `zypper`/`eopkg`/`xbps` (manual) | Package names may differ; run install.sh and install missing deps manually if it fails |
| ❌ **Not supported** | NixOS, Alpine, Gentoo, others | N/A | No package manager entry — install deps manually, then `cargo build --release` |

> **⚠️ Tray icon note:** GNOME (Ubuntu) and Cinnamon (Mint) need a StatusNotifier watcher for the tray to appear:
> - Ubuntu: `sudo apt install gnome-shell-extension-appindicator`
> - Mint: pre-installed; works out of the box

---

## Installation

### One-Command Install

Works on all ✅ **Supported** distros above. The script auto-detects your package manager:

**From GitHub (recommended):**
```bash
git clone https://github.com/vndangkhoa/vietc.git /tmp/vietc \
  && cd /tmp/vietc && sudo ./install.sh
```

**From Forgejo (self-hosted):**
```bash
git clone https://git.khoavo.myds.me/vndangkhoa/vietc.git /tmp/vietc \
  && cd /tmp/vietc && sudo ./install.sh
```

The script installs dependencies, compiles, installs to `/usr/bin/`, sets up uinput udev rules, and adds your user to the `input` group.

**After install:** Log out and log back in, then launch `vietc-tray` from your application menu.

### One-Command Uninstall

**From GitHub:**
```bash
curl -sSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash
```

**From Forgejo:**
```bash
curl -sSL https://git.khoavo.myds.me/vndangkhoa/vietc/raw/branch/main/uninstall.sh | sudo bash
```

### Manual Build & Run

```bash
# Install dependencies
sudo apt install git curl build-essential pkg-config \
  libx11-dev libxtst-dev libevdev-dev libdbus-1-dev libwayland-dev wl-clipboard

# Enable accessibility (Ubuntu Wayland — for password detection)
gsettings set org.gnome.desktop.a11y.applications screen-reader-enabled true

# Build
git clone https://github.com/vndangkhoa/vietc.git
cd vietc
cargo build --release

# Run (Mint — no sudo needed for uinput)
./target/release/vietc

# Run (Ubuntu — needs sudo for keyboard grab)
sudo ./target/release/vietc
```

---

## Configuration

Config file: `~/.config/vietc/config.toml` or `./vietc.toml`

```toml
input_method = "vni"            # "vni" or "telex"
toggle_key = "space"            # Ctrl+Space to toggle VN/EN
toggle_method_key = "shift"     # Ctrl+Shift to toggle VNI/Telex
start_enabled = true            # Vietnamese by default
grab = true                     # grab keyboard (evdev)

[auto_restore]
enabled = false                 # Auto-restore English words (defaults to false)
trigger_keys = ["space", "escape"]

[password_detection]
enabled = true
check_atspi2 = true
check_window_title = true
title_keywords = ["password", "passphrase", "secret", "mật khẩu", "sudo"]
password_apps = ["pinentry", "pinentry-gtk-2", "pinentry-qt",
  "lxqt-sudo", "kdesudo", "gksudo",
  "polkit-gnome-authentication-agent-1",
  "kwallet", "gnome-keyring", "ssh-askpass"]

[app_state]
enabled = true
english_apps = ["code", "vim"]
vietnamese_apps = ["telegram", "discord", "firefox"]
bypass_apps = ["steam"]
terminal_apps = ["kitty", "alacritty", "gnome-terminal", "konsole", "foot",
  "wezterm", "st", "urxvt", "xterm"]
terminal_input_method = "vni"   # Automatically switch to VNI when running in a terminal app

[macros]
ko = "không"
dc = "được"
vs = "với"
```

### Terminal Usage

Viet+ works perfectly in terminals. When running inside a terminal (e.g., gnome-terminal, kitty), Vietnamese input is automatically enabled using the input method specified by `terminal_input_method` under `[app_state]`.

Supported terminals: `kitty`, `alacritty`, `gnome-terminal`, `konsole`, `foot`, `wezterm`, `st`, `urxvt`, `xterm`

Type Vietnamese directly — no pre-edit buffer, no underline, no duplication. Just type VNI or Telex digits and see Unicode characters instantly!

---

## Architecture

```
vietc/
├── engine/                  # Vietnamese composition engine (bamboo-core port)
├── protocol/                # Keyboard capture & injection
│   ├── uinput_monitor.rs    # /dev/uinput injection (primary)
│   ├── x11_inject.rs        # XTest injection (fallback)
│   ├── x11_capture.rs       # XRecord key capture
│   └── wayland_im.rs        # Wayland IM protocol (stub)
├── daemon/                  # Main daemon process
│   ├── main.rs              # Event loops, grab, signal handling
│   ├── config.rs            # TOML config loader + hot reload
│   ├── app_state.rs         # Per-app VN/EN memory + password detection
│   ├── password_detector.rs # AT-SPI2 D-Bus password field detection
│   └── display.rs           # X11/Wayland/compositor detection
├── ui/                      # System tray icon (ksni)
│   └── tray.rs              # Tray with VN/TLX/EN mode display
├── cli/                     # Interactive test harness
└── uinputd/                 # Privileged uinput socket daemon
```

---

## Roadmap

### v0.1.19
- [ ] Wayland input method protocol (`zwp_input_method_v2`) — eliminates clipboard + backspace race, fixes missing spaces permanently
- [ ] Event-based AT-SPI2 focus monitoring (subscribe to a11y focus events, no polling)

### v0.1.20
- [ ] GitHub Actions CI for automated .deb builds
- [ ] Flatpak re-add for immutable distros

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Made with love for the Vietnamese Linux community</sub>
</p>
