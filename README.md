# ⌨️ Viet+

**Vietnamese Input Method for Linux · Hybrid IBus + Direct Input · Built in Rust — Ubuntu 24.04+ Wayland Ready**

[![Platform](https://img.shields.io/badge/Platform-Linux-blue?style=flat-square)](https://github.com/vndangkhoa/vietc)
[![Rust](https://img.shields.io/badge/Rust-1.85-000000?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/badge/Version-0.1.8-purple?style=flat-square)](https://github.com/vndangkhoa/vietc)
[![Tests](https://img.shields.io/badge/Tests-104_passing-brightgreen?style=flat-square)](https://github.com/vndangkhoa/vietc)

[Why Viet+?](#-why-viet) • [Features](#-features) • [Installation](#-installation) • [Configuration](#-configuration) • [Usage](#-usage) • [Architecture](#-architecture) • [Development](#-development) • [Contributing](#-contributing)

---

*Type Vietnamese smoothly everywhere — Hybrid engine, auto-selected: IBus preedit on Ubuntu 24.04+ Wayland (like Funput), Wayland `zwp_input_method_v2` on Hyprland/Sway, direct `evdev+uinput` on Mint/Arch/X11.*

> [!WARNING]
> This project is in active development. The legacy `evdev`/`uinput` path operates on input devices and may lock your keyboard if a bug occurs — use with caution. The default on Ubuntu GNOME Wayland is the safe `IBus` engine (no grab, no `uinput`, rootless).

---

## 🤔 Why Viet+?

Most Vietnamese IMEs use a **pre-edit buffer** — you type into a temporary buffer with an ugly underline, and the text only becomes real Vietnamese when you commit it. This causes duplicate text, underline distraction, broken copy/paste, and desync.

Viet+ is **hybrid, like Funput on Linux**:
* **Ubuntu 24.04+ Wayland (GNOME/Mutter)** → native **IBus engine** (`org.freedesktop.IBus` via D-Bus, preedit with underline, `Ctrl+Space` smooth, covers every Wayland-native app like `Firefox`/`Ptyxis`/`GNOME Text Editor` where `Mutter` lacks `zwp_input_method_v2`).
* **Hyprland/Sway** → native **Wayland `zwp_input_method_v2`** + `zwp_virtual_keyboard_v1`.
* **Mint/Arch/X11** → **Direct `evdev+uinput`** 0ms direct (grab persists, no underline).

Same `Bamboo` core, shell decides. `auto_ibus=true` (default) auto-selects the smoothest path for your session.

**What you get:**

- **Smoothness** — Correct path per desktop: IBus preedit where Wayland needs it, direct where it shines.
- **Cleanliness** — No duplication, no garbled backspace in Chrome/Firefox address bar (Firefox space-fold fix).
- **Reliability** — `IBus`/`Wayland` need no `input` group, no `setcap`; `evdev` grab persists for X11.
- **Freedom** — Open source, MIT, offline, no telemetry.

### 📖 Backstory

I built Viet+ because every Vietnamese IME on Linux annoyed me with the pre-edit underline and the broken copy/paste that came with it. The buffer approach fundamentally desyncs the engine from what's on screen.

What started as a small uinput experiment became a full Rust daemon with a Bamboo-based composition engine, per-app memory, password detection, a tray icon, and a test harness that verifies on-screen output with real synthetic keystrokes. It runs on my Linux desktop every day.

If that resonates, give it a star ⭐ — it helps others find the project.

---

## ✨ Features

| Icon | Feature | What it does |
|------|---------|--------------|
| ⚡ | **Hybrid Engine** | `IBus` on Ubuntu 24.04+ Wayland (preedit, like `Funput funput-ibus`), `zwp_v2` on Hyprland/Sway, `evdev+uinput` direct on Mint/Arch/X11 — auto (`auto_ibus`). |
| 🔤 | **VNI & Telex** | Both input methods, switchable at runtime via **Ctrl+Shift** (`IBus` also `Ctrl+Shift`, visible preedit). |
| 🎋 | **Bamboo Engine** | Composition, marks, tones, and flexible backtracking. |
| 🧩 | **Smart Clusters** | `uo→ươ` with backtrack, `ua→ưa` horn placement. |
| 📝 | **Macro Expansion** | `ko → không`, `dc → được`, `vs → với` — add your own. |
| 🔡 | **Casing Preservation** | `Tieengs → Tiếng`, `TIEENGS → TIẾNG`. |
| 🧠 | **App Memory** | Per-app Vietnamese/English state, saved to `overrides.toml`. |
| ♻️ | **Hot Reload** | Config changes apply without restart. |
| 🪟 | **Window-Switch Reset** | Engine clears automatically on Alt+Tab / `FocusIn`. |
| 🚀 | **CPU Priority** | Pinned to P-cores (0-3) + nice(-10) for low-latency input (direct path). |
| 🖱️ | **Injection** | `IBus CommitText/UpdatePreeditText` on Wayland, `/dev/uinput` + `XTEST` fallback on X11. |
| 💻 | **Terminal Support** | Works in kitty, alacritty, gnome-terminal, konsole, foot, wezterm, st, urxvt, xterm, `ptyxis` (via IBus per-app). |
| 🔐 | **Password Auto-Detection** | 4 layers: AT-SPI2 → sudo process → window-title → window-class. |
| 📊 | **Tray Icon** | Shows current mode: Red VN / Blue TLX / Gray EN (`Ctrl+Space` toggle). |
| 🐚 | **GNOME/Wayland** | `IBus` engine for Ubuntu + `zwp_input_method_v2` for wlroots + GNOME Shell D-Bus integration. |

---

## 📥 Installation

### 🚀 Quick Start (One-Command)

Works on all ✅ **Supported** distros. The script automatically detects your package manager, installs dependencies (`wtype`, `libevdev`, etc.), compiles from source or fetches binaries, configures uinput rootless permissions, and registers the system service & tray autostart.

```bash
curl -fsSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/install.sh | bash
```

**How to switch typing modes:**
* Press **`Ctrl + Shift`** to rotate: **⚪ EN ➔ 🔴 VNI ➔ 🔵 TELEX ➔ ⚪ EN**
* Or click the system tray icon to switch anytime!
* Status CLI: `vietcctl status` | Cycle: `vietcctl cycle`

**After install:** 
* **Ubuntu 24.04+ Wayland (GNOME):** `systemctl --user enable --now vietc.service` (installer does it live), then `ibus engine` → `vietc`. No logout needed for `IBus` path; `evdev` path still needs logout for `input` group. Verify: `journalctl --user -u vietc.service -f` should show `IBus engine mode auto-enabled`.
* **Mint/Arch X11:** Log out and log back in, then launch `vietc-tray` from your application menu.

### 📦 Source Repositories

The project is mirrored on GitHub and Forgejo — both stay in sync:

- **GitHub:** [https://github.com/vndangkhoa/vietc](https://github.com/vndangkhoa/vietc)
- **Forgejo:** [https://git.khoavo.myds.me/vndangkhoa/vietc](https://git.khoavo.myds.me/vndangkhoa/vietc)

### 📋 Distro Support

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
>
> `vietc-tray` is installed by `./install.sh` — run the installer first, *then* launch the tray. On Ubuntu the installer also sets up the appindicator extension automatically.

### 🗑️ Uninstall

```bash
# From GitHub
curl -sSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash

# From Forgejo
curl -sSL https://git.khoavo.myds.me/vndangkhoa/vietc/raw/branch/main/uninstall.sh | sudo bash
```

### 🔧 Manual Build & Run

```bash
# Install dependencies (note: libxkbcommon-dev needed for Wayland xkb)
sudo apt install git curl build-essential pkg-config \
  libx11-dev libxtst-dev libevdev-dev libdbus-1-dev libwayland-dev libxkbcommon-dev wl-clipboard

# Enable accessibility (Ubuntu Wayland — for password detection)
gsettings set org.gnome.desktop.a11y.applications screen-reader-enabled true

# Build
git clone https://github.com/vndangkhoa/vietc.git
cd vietc
cargo build --release
# UI tray (separate workspace)
(cd ui && cargo build --release)

# Run — hybrid auto (IBus on GNOME Wayland, evdev on X11)
./target/release/vietc
# Force IBus (test Ubuntu fix on X11)
VIETC_FORCE_IBUS=1 ./target/release/vietc
# Force direct evdev (test X11 path)
./target/release/vietc # with grab=true in config.toml + sudo if needed

# Systemd (recommended)
systemctl --user enable --now vietc.service
journalctl --user -u vietc.service -f
```

---

## ⚙️ Configuration

Config file: `~/.config/vietc/config.toml` or `./vietc.toml` (`/etc/vietc/config.toml` fallback)

| Variable | Default | Description |
|----------|---------|-------------|
| `input_method` | `"vni"` | `"vni"` or `"telex"` |
| `toggle_key` | `"space"` | Ctrl+Space to toggle VN/EN (IBus also `Ctrl+Space`) |
| `toggle_method_key` | `"shift"` | Ctrl+Shift to toggle VNI/Telex (IBus also `Ctrl+Shift`) |
| `start_enabled` | `true` | Vietnamese by default |
| `grab` | `false` | Grab keyboard (evdev) — `false` default for rootless/IBus |
| `auto_ibus` | `true` | Auto-enable `IBus` engine on Ubuntu/Debian `GNOME Wayland` (like `Funput`) |
| `ibus_engine` | `false` | Force `IBus` engine (`true` with `--ibus`) |
| `controller_mode` | `false` | Aux `Bamboo` controller (switch `Bamboo`/`BambooUs` per-app) |
| `[auto_restore].enabled` | `true` | Auto-restore English words (daemon `overrides.toml`) |
| `[password_detection].enabled` | `true` | Auto-disable in password fields |
| `[app_state].terminal_input_method` | `"vni"` | Method used inside terminal apps |

```toml
input_method = "vni"            # "vni" or "telex"
toggle_key = "space"            # Ctrl+Space to toggle VN/EN
toggle_method_key = "shift"     # Ctrl+Shift to toggle VNI/Telex
start_enabled = true            # Vietnamese by default
grab = false                    # grab keyboard (evdev) — false for IBus/Wayland
auto_ibus = true                # auto IBus on Ubuntu GNOME Wayland (Funput-style)
ibus_engine = false             # set true or use --ibus to force IBus
controller_mode = false         # true for Bamboo per-app controller

[auto_restore]
enabled = true                  # Auto-restore English words
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
# Env override: VIETC_FORCE_IBUS=1 forces IBus even on X11
```

---

## 🎮 Usage

### Input Methods

Both **VNI** and **Telex** are fully supported. Switch via **Ctrl+LeftShift** or the tray menu.

**VNI**

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

**Telex**

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

### Key Bindings

| Combo | Action |
|-------|--------|
| **Ctrl+Space** | Toggle Vietnamese ON/OFF |
| **Ctrl+LeftShift** | Toggle VNI ↔ Telex |

### Password Detection

4-layer automatic detection. When a password field is detected, Vietnamese is automatically disabled:

| Layer | Method | Detects |
|-------|--------|---------|
| 1 | AT-SPI2 D-Bus (a11y role check) | Password fields in accessible apps |
| 2 | Process tree (pstree) | `sudo` / `passwd` in terminal |
| 3 | Window title keywords | `password`, `sudo` in title |
| 4 | Window class matching | pinentry, polkit, kwallet dialogs |

### Terminal Usage

Viet+ works perfectly in terminals. When running inside a **detectable** terminal (X11/XWayland — e.g. `kitty`, `alacritty`, `gnome-terminal`), Vietnamese input is automatically disabled (English) because Bamboo's no-underline mode can't do in-place editing inside a terminal and would garble the text.

Supported/detected terminals: `kitty`, `alacritty`, `gnome-terminal`, `konsole`, `foot`, `wezterm`, `st`, `urxvt`, `xterm`, `code` (VS Code).

**Wayland-native terminals** (e.g. **ptyxis**) can't be auto-detected on this GNOME session, so they're handled via IBus **per-app engine memory**: set ptyxis once to `BambooUs` (English) and it sticks.

Type Vietnamese directly — no pre-edit buffer, no underline, no duplication. Just type VNI or Telex digits and see Unicode characters instantly!

---

## 🏗️ Architecture

Viet+ is a **hybrid** Linux daemon written in Rust — same `Bamboo` core, different shells per desktop (like `Funput` `funput`/`funput-ibus`).

| Layer | Tech | Role |
|-------|------|------|
| **Engine** | Rust + Bamboo core | Composition, marks, tones, backtracking (shared) |
| **IBus** | D-Bus `org.freedesktop.IBus` (`daemon/src/ibus_engine.rs`) | Native on Ubuntu 24.04+ Wayland (GNOME/Mutter, where `Mutter` lacks `zwp_input_method_v2`); preedit `UpdatePreeditText` + `CommitText`, `Ctrl+Space`/`Ctrl+Shift` |
| **Wayland** | `zwp_input_method_v2` + `zwp_virtual_keyboard_v1` (`daemon/src/wayland_im.rs`) | Hyprland/Sway native, rootless |
| **Capture** | `evdev` / XRecord / X11 keymap (`evdev_loop.rs`, `x11_capture.rs`) | Direct on Mint/Arch/X11; `input` group or `setcap` |
| **Injection** | `IBus` vs `/dev/uinput` vs `XTEST` (`protocol/`) | Per-path injection |
| **App State** | AT-SPI2 D-Bus + `overrides.toml` | Per-app VN/EN memory + password detection |
| **UI** | `ksni` tray | VN / TLX / EN mode indicator |
| **Config** | `TOML` (`auto_ibus`, `ibus_engine`) | Hot-reloadable, `VIETC_FORCE_IBUS=1` override |

Order probed at startup: `controller_mode` → `IBus` (`auto_ibus` on `GNOME Wayland`) → `zwp_input_method_v2` → `evdev` → `X11 keymap` → `stdin` (see `daemon/src/main.rs`, `docs/wayland-rootless.md`).

```
vietc/
├── engine/                  # Vietnamese composition engine (bamboo-core port)
├── protocol/                # Keyboard capture & injection
│   ├── uinput_monitor.rs    # /dev/uinput injection (primary)
│   ├── x11_inject.rs        # XTest injection (fallback)
│   ├── x11_capture.rs       # XRecord key capture
│   └── wayland_im.rs        # Wayland IM protocol (stub)
├── daemon/                  # Main daemon process
│   ├── main.rs              # Entry point, CLI argument parsing
│   ├── daemon.rs            # Daemon struct: process_key, toggle, replay
│   ├── config.rs            # TOML config loader + hot reload
│   ├── app_state.rs         # Per-app VN/EN memory + password detection
│   ├── event.rs             # Pure event routing functions + grab-render tests
│   ├── evdev_loop.rs        # evdev poll loop (grabbed & non-grabbed modes)
│   ├── inject.rs            # Command execution, injector creation
│   ├── stdin.rs             # Stdin mode with retry loop
│   ├── x11_capture.rs       # X11 RECORD + keymap capture paths
│   ├── device.rs            # Keyboard device discovery + permissions
│   ├── signal.rs            # SIGINT/SIGTERM handler, single-instance lock
│   ├── env.rs               # DISPLAY/DBUS env recovery from /proc
│   ├── password_detector.rs # AT-SPI2 D-Bus password field detection
│   ├── commands.rs          # OutputCommand enum
│   ├── log.rs               # Log rotation, timestamps
│   ├── display.rs           # X11/Wayland/compositor detection
│   └── tests/               # Integration test harness
│       ├── daemon_suite.rs
│       └── common/
│           ├── virtual_keyboard.rs
│           ├── clipboard.rs
│           ├── distro.rs
│           └── mod.rs
├── ui/                      # System tray icon (ksni)
│   └── tray.rs              # Tray with VN/TLX/EN mode display
├── cli/                     # Interactive test harness
└── uinputd/                 # Privileged uinput socket daemon
```

### Advantages of the Modular Architecture

The 0.1.7 refactoring split a 2151-line `main.rs` into 11 focused modules, delivering measurable improvements in maintainability, testability, and correctness:

- **Grab Persists Forever** — The grab now persists until the daemon exits, eliminating the root cause of garbled input.
- **No Double-Input** — Non-primary keyboard devices always skip the engine and forward keys directly, fixing duplicate keystrokes.
- **Testable Event Routing** — Pure functions in `event.rs` render keystrokes entirely in memory, mirroring the production evdev loop.
- **Integration Test Harness** — Spawns a real daemon, sends synthetic keystrokes via virtual uinput keyboards, and reads the clipboard to verify output across distros.
- **Regression Prevention** — Every past bug maps to a documented test scenario in `docs/testing-dictionary.md` (40+ entries).

---

## 💻 Development

```bash
# Build
cargo build
(cd ui && cargo build) # tray

# Run tests (124 passing: 78 engine + 46 daemon)
cargo test -p vietc-engine -p vietc-daemon

# Run — hybrid auto (IBus on GNOME Wayland, evdev on X11)
./target/release/vietc
# Force IBus (test Ubuntu fix on any session)
VIETC_FORCE_IBUS=1 ./target/release/vietc
# Via systemd (recommended, auto IBus on Ubuntu)
systemctl --user enable --now vietc.service
journalctl --user -u vietc.service -f # look for "IBus engine mode auto-enabled"
```

---

## 🤝 Contributing

Contributions are welcome! Here's how to help:

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing`)
5. Open a Pull Request

Please make sure to follow existing code style and add tests when possible. Writing the integration test is the first step of every bug fix.

---

## 📄 License

Distributed under the **MIT License**. See [LICENSE](LICENSE) for more information.

If you find this project useful, please [⭐ star it on GitHub](https://github.com/vndangkhoa/vietc).  
Built with ❤️ for the Vietnamese Linux community.

<p align="center">
  <sub>Made with love for the Vietnamese Linux community</sub>
</p>
