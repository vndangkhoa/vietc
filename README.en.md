# ⌨️ Viet+ (VietC)

**Modern Zero-Underline Vietnamese Input Method for Linux (Wayland & X11) · Built with Rust 🦀**

[![Platform](https://img.shields.io/badge/Platform-Linux%20(Wayland%20%7C%20X11)-blue?style=flat-square)](https://github.com/vndangkhoa/vietc)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/badge/Version-0.1.9-purple?style=flat-square)](https://github.com/vndangkhoa/vietc)
[![Tests](https://img.shields.io/badge/Tests-151_passing-brightgreen?style=flat-square)](https://github.com/vndangkhoa/vietc)

[Overview](#-overview) • [Quick Start](#-quick-start-1-command-install) • [Features](#-features) • [Usage](#-usage) • [Configuration](#-configuration) • [Architecture](#-architecture) • [Testing](#-testing) • [🇻🇳 Tiếng Việt](README.md)

---

## 🌟 Overview

**Viet+ (VietC)** is a next-generation, high-performance Vietnamese input method engine for Linux. Written entirely in **Rust**, it eliminates the annoying pre-edit underlines and clipboard race conditions found in traditional IMEs, delivering native, zero-latency direct typing on both **Wayland** (Hyprland, Sway, GNOME, KDE Plasma) and **X11**.

```
Nguyeenx DDawng Khoa   ➔   Nguyễn Đăng Khoa
Khoong cos gif quis    ➔   Không có gì quí
search for the test    ➔   search for the test  (Intelligent Auto-Restore)
```

---

## 🚀 Quick Start (1-Command Install)

Install Viet+ with a single command on any supported Linux distribution:

```bash
curl -fsSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/install.sh | bash
```

### How to Switch Modes
- Press **`Ctrl + Shift`** to cycle: **⚪ EN (English) ➔ 🔴 VNI ➔ 🔵 TELEX ➔ ⚪ EN**
- Or click the dynamic system tray icon (**EN / VN / TLX**) anytime.
- CLI controls: `vietcctl status` | `vietcctl cycle` | `vietcctl method telex`

---

## 🐧 Supported Distros & Environments

Viet+ is tested and optimized across modern hype, gaming, and mainstream Linux distributions:

| Distro / Ecosystem | Default Desktop / WM | Display Server | Input Mechanism | Status |
| :--- | :--- | :--- | :--- | :--- |
| ⚡ **CachyOS** | KDE Plasma 6 / Hyprland | **Wayland** | `wtype` (Direct Virtual Keyboard) | ✅ **100% Optimized** |
| 🏹 **Arch Linux** | Hyprland / Sway / KDE / GNOME | **Wayland / X11** | `wtype` / `/dev/uinput` | ✅ **100% Tested** |
| 🚀 **EndeavourOS / Omarchy / Garuda** | Hyprland / KDE Plasma / i3 | **Wayland / X11** | `wtype` / `/dev/uinput` | ✅ **Fully Supported** |
| 🎩 **Fedora 40/41 / Nobara** | GNOME 46/47 / KDE Plasma | **Wayland** | Hybrid IBus + `wtype` | ✅ **Fully Supported** |
| 🌿 **Linux Mint** | Cinnamon / XFCE / MATE | **X11** | `/dev/uinput` Direct | ✅ **Fully Supported** |
| 🪐 **Pop!_OS** | COSMIC Desktop / GNOME | **Wayland / X11** | `wtype` / `/dev/uinput` | ✅ **Fully Supported** |
| 🟠 **Ubuntu 24.04+ / Debian 12** | GNOME (Mutter) / X11 | **Wayland / X11** | Hybrid IBus + AppIndicator | ✅ **Fully Supported** |
| 🦎 **Manjaro / openSUSE** | KDE Plasma / XFCE / GNOME | **Wayland / X11** | `wtype` / `/dev/uinput` | ✅ **Fully Supported** |

---

## ✨ Features

| Feature | Description |
| :--- | :--- |
| 🚀 **Zero Underline** | Types directly into the active application. No temporary pre-edit buffer, no distracting underline, no broken copy/paste. |
| ⚡ **Direct Wayland Virtual Keyboard** | Uses `wtype` (`zwp_virtual_keyboard_v1`) on Wayland / Hyprland. Injects UTF-8 Unicode directly with 0ms latency and zero clipboard conflicts. |
| 🛡️ **Hardware Device Filtering** | Automatically detects and binds strictly to `/dev/input/by-path/*-event-kbd`, completely preventing duplicate keystrokes from 2.4G wireless USB dongles and multi-interface hardware. |
| 🧠 **Intelligent English Auto-Restore** | Analyzes Vietnamese phonology and validates against a built-in English technical dictionary. Typing English words in Telex/VNI mode automatically restores clean English spelling on space/punctuation. |
| 🔄 **3-Way Instant Rotation** | Seamlessly cycles **⚪ EN ➔ 🔴 VNI ➔ 🔵 TELEX** with `Ctrl + Shift`, tray icon click, or CLI. Displays synchronized desktop OSD notifications. |
| 🎋 **Bamboo Composition Core** | Full Vietnamese diacritics (`â, ă, ê, ô, ơ, ư, đ`), smart vowel clusters (`uo ➔ ươ`, `ua ➔ ưa`), and natural tone placement. |
| 🔡 **Casing Preservation** | Accurately preserves capitalization: `Tieengs ➔ Tiếng`, `TIEENGS ➔ TIẾNG`. |
| 📝 **Custom Macro Expansion** | Built-in and user-customizable macros (`ko ➔ không`, `dc ➔ được`, `vs ➔ với`, etc.). |
| 📊 **Dynamic Tray Indicator** | StatusNotifier/AppIndicator tray icon with clear color-coded badges (**EN / VN / TLX**). |
| 🔒 **Rootless & Secure** | Runs as a standard user systemd service. No root daemon required after initial uinput udev setup. |

---

## 🎮 Usage

### 1. Typing Methods

#### **Telex Mode (🔵)**
| Input | Output | Example |
| :--- | :--- | :--- |
| `s` | Sắc (´) | `as` ➔ `á` |
| `f` | Huyền (`) | `af` ➔ `à` |
| `r` | Hỏi (̉ ) | `ar` ➔ `ả` |
| `x` | Ngã (~) | `ax` ➔ `ã` |
| `j` | Nặng (.) | `aj` ➔ `ạ` |
| `aa`, `ee`, `oo` | Â, Ê, Ô | `aa` ➔ `â`, `ee` ➔ `ê`, `oo` ➔ `ô` |
| `aw`, `ow`, `uw` | Ă, Ơ, Ư | `aw` ➔ `ă`, `ow` ➔ `ơ`, `uw` ➔ `ư` |
| `dd` | Đ | `dd` ➔ `đ`, `DD` ➔ `Đ` |
| `w` | Ươ | `chuongw` ➔ `chương` |

#### **VNI Mode (🔴)**
| Input | Output | Example |
| :--- | :--- | :--- |
| `1` | Sắc (´) | `a1` ➔ `á` |
| `2` | Huyền (`) | `a2` ➔ `à` |
| `3` | Hỏi (̉ ) | `a3` ➔ `ả` |
| `4` | Ngã (~) | `a4` ➔ `ã` |
| `5` | Nặng (.) | `a5` ➔ `ạ` |
| `6` | Â, Ê, Ô | `a6` ➔ `â`, `e6` ➔ `ê`, `o6` ➔ `ô` |
| `7` | Ơ, Ư | `o7` ➔ `ơ`, `u7` ➔ `ư` |
| `8` | Ă | `a8` ➔ `ă` |
| `9` | Đ | `d9` ➔ `đ`, `D9` ➔ `Đ` |

---

### 2. Shortcuts & Controls

| Action | Shortcut / Command |
| :--- | :--- |
| **Cycle 3 Modes** (EN ➔ VNI ➔ TELEX) | `Ctrl + Shift` (or left-click tray icon) |
| **Toggle VN / EN** | `Ctrl + Space` |
| **Check Current Status** | `vietcctl status` |
| **Switch Method via CLI** | `vietcctl method telex` / `vietcctl method vni` |
| **Restart Service** | `systemctl --user restart vietc.service` |

---

## ⚙️ Configuration

Configuration file location: `~/.config/vietc/config.toml`

```toml
input_method = "telex"          # "telex" or "vni"
toggle_key = "space"            # Ctrl+Space for VN/EN toggle
start_enabled = true            # Enable Vietnamese on startup
grab = true                     # Direct hardware evdev grab
deduplicate_keys = false        # Let engine handle consecutive letters (aa, ee, dd)

[auto_restore]
enabled = true                  # Automatically restore English words on space
trigger_keys = ["space", "escape"]

[app_state]
enabled = false                 # App-specific overrides (optional)

[macros]
"ko" = "không"
"dc" = "được"
"vs" = "với"
"ng" = "người"
```

---

## 🏗️ Architecture

```
vietc/
├── engine/                  # Bamboo-based composition core & spell checking
│   ├── bamboo.rs            # Diacritics, marks, backtracking & casing rules
│   ├── english.rs           # English vocabulary & auto-restore dictionary
│   ├── spelling.rs          # Vietnamese phonology & syllable validator
│   └── tests.rs             # Comprehensive unit benchmark suite
├── daemon/                  # Main background service daemon
│   ├── evdev_loop.rs        # Low-latency evdev poll loop
│   ├── device.rs            # by-path device discovery & hardware filtering
│   ├── daemon.rs            # Mode cycling, shortcuts & status synchronization
│   └── app_state.rs         # Application state management & overrides
├── protocol/                # Input injection & virtual keyboard protocols
│   ├── uinput_monitor.rs    # wtype (Wayland virtual keyboard) + /dev/uinput
│   └── wayland_im.rs        # Wayland input-method context
├── ui/                      # Dynamic system tray application (ksni)
│   └── tray.rs              # 3-state icon renderer (EN / VN / TLX)
├── vietcctl/                # Command-line control interface (IPC)
├── web/                     # Official interactive documentation website (React + Vite)
└── install.sh               # Universal 1-command installer script
```

---

## 🧪 Testing

The repository contains 151 automated tests covering standard Vietnamese orthography, complex vowel clusters, English auto-restore, and Wayland virtual keyboard protocols:

```bash
# Run all tests across the workspace
cargo test --workspace
```

---

## 📦 Repositories

Viet+ is mirrored across both Forgejo and GitHub:
- **GitHub**: [https://github.com/vndangkhoa/vietc](https://github.com/vndangkhoa/vietc)
- **Forgejo**: [https://git.khoavo.myds.me/vndangkhoa/vietc](https://git.khoavo.myds.me/vndangkhoa/vietc)

---

## 🤝 Contributing

Contributions are welcome!
1. Fork the repo
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing`)
5. Open a Pull Request

---

## 📄 License

Distributed under the **MIT License**. See [LICENSE](LICENSE) for details.

If you find this project useful, please [⭐ star it on GitHub](https://github.com/vndangkhoa/vietc).  
Built with ❤️ for the Vietnamese Linux community.

