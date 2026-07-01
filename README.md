<p align="center">
  <img src="https://img.shields.io/badge/Platform-Linux-blue?style=for-the-badge" alt="Platform">
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/Version-0.1.7-purple?style=for-the-badge" alt="Version">
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

## Installation

### Build from Source

```bash
# Dependencies (Ubuntu/Debian)
sudo apt install git curl build-essential pkg-config \
  libx11-dev libxtst-dev libevdev-dev libdbus-1-dev

# Clone and build
git clone https://github.com/vndangkhoa/vietc.git
cd vietc
cargo build --release

# Add user to input group (for keyboard capture)
sudo usermod -aG input $USER
# Log out and log back in

# Run
./target/release/vietc
```

### Wayland (Ubuntu 24.04+) — Additional steps

```bash
sudo apt install wl-clipboard
gsettings set org.gnome.desktop.a11y.applications screen-reader-enabled true
```

### uinput Access (recommended)

```bash
sudo modprobe uinput
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-uinput.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
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
english_apps = ["code", "vim", "kitty", "foot"]
vietnamese_apps = ["telegram", "discord", "firefox"]
bypass_apps = ["kitty", "alacritty", "steam"]

[macros]
ko = "không"
dc = "được"
vs = "với"
```

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

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Made with love for the Vietnamese Linux community</sub>
</p>
