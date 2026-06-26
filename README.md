<p align="center">
  <img src="https://img.shields.io/badge/Platform-Linux-blue?style=for-the-badge" alt="Platform">
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/Version-0.1.0-purple?style=for-the-badge" alt="Version">
</p>

<h1 align="center">
  <br>
  Viet+
  <br>
</h1>

<p align="center">
  <b>Vietnamese Input Method for Linux</b><br>
  <sub>Zero underline &bull; Native X11 &bull; Backspace-Replay sync &bull; Built in Rust</sub>
</p>

## About Viet+

Viet+ is a modern Vietnamese input method for Linux that eliminates the **underline hell** common in other Vietnamese IMEs. Unlike traditional solutions that use pre-edit buffers with ugly underlines and duplicate text, Viet+ implements a **Direct Input** approach:

- **No pre-edit buffer** — keystrokes are instantly converted to Unicode
- **No underline** — clean, distraction-free typing
- **No text duplication** — just pure Vietnamese
- **Backspace-Replay sync** — engine state never desyncs from what's on screen

---

## Features

| Feature | Description |
|---------|-------------|
| **Direct Input Engine** | No pre-edit buffer, no underline, no text duplication |
| **Backspace-Replay** | Replays entire keystroke history through a fresh engine on every keypress — eliminates state desync |
| **Telex & VNI** | Both input methods fully supported |
| **Flexible Diacritic Placement** | Type modifiers/tone marks at end of syllable (e.g., `tranaf` -> `trần`) |
| **Auto-Restore English** | Hit space/ESC to undo accidental Vietnamese conversion |
| **ESC Undo** | Strip all tones from the current word instantly |
| **Smart App Memory** | Remembers Vietnamese/English per application |
| **Macro Expansion** | Custom shortcuts (e.g., `ko` -> `không`) |
| **Focus Reset** | Automatically clears engine state on focus change between apps |
| **Casing Preservation** | Syllable substitutions preserve your exact casing (e.g. `Saa` -> `Sả`, `SAA` -> `SẢ`) |
| **CPU Priority** | Pins daemon to P-cores + nice(-10) for low-latency input |
| **Zero Telemetry** | No keylogging, no network calls, fully FOSS |

---

## Why Viet+?

Most Vietnamese input methods on Linux suffer from **underline hell** — pre-edit buffers that duplicate text, show ugly underlines, and break your flow. Viet+ takes a different approach:

> **Direct Input** — keystrokes are instantly converted to Unicode. No pre-edit buffer. No underline. No text duplication. Just pure Vietnamese.

The **Backspace-Replay** pattern keeps the engine perfectly in sync: instead of tracking state incrementally (which can desync), Viet+ replays the entire keystroke history through a fresh engine on every keypress. The screen output is always recomputed from scratch.

---

## Quick Start

```bash
# Clone and build
git clone https://git.khoavo.myds.me/vndangkhoa/vietc.git
cd vietc
make build-all

# Test the engine interactively
cargo run --bin vietc-cli

# Run the daemon
cargo run --bin vietc

# Or download a package from the releases page
#   AppImage: ./Viet+-0.1.0-x86_64.AppImage
#   Debian:   sudo dpkg -i vietc_0.1.0-1_amd64.deb
```

---

## Input Methods

### Telex

| Key | Result | Example |
|-----|--------|---------|
| `aa` | â | `tan` -> `tân` |
| `aw` | ă | `tan` -> `tăn` |
| `ee` | ê | `men` -> `mên` |
| `oo` | ô | `to` -> `tô` |
| `ow` | ơ | `to` -> `tơ` |
| `uw` | ư | `tu` -> `tư` |
| `s` | á (sắc) | `as` -> `á` |
| `f` | à (huyền) | `af` -> `à` |
| `r` | ả (hỏi) | `ar` -> `ả` |
| `x` | ã (ngã) | `ax` -> `ã` |
| `j` | ạ (nặng) | `aj` -> `ạ` |
| `dd` | đ | `dd` -> `đ` |

### VNI

| Key | Result |
|-----|--------|
| `a1` | á |
| `a2` | à |
| `a3` | ả |
| `a4` | ã |
| `a5` | ạ |
| `a6` | â |
| `a8` | ă |
| `e6` | ê |
| `o6` | ô |
| `o7` | ơ |
| `u7` | ư |

---

## Configuration

Config file: `~/.config/vietc/config.toml` or `./vietc.toml`

```toml
input_method = "vni"
toggle_key = "space"
start_enabled = false
grab = true

[auto_restore]
enabled = true
trigger_keys = ["space", "escape"]

[app_state]
enabled = true
english_apps = ["code", "vim", "kitty", "foot"]
vietnamese_apps = ["telegram", "discord", "firefox"]

[macros]
ko = "không"
dc = "được"
vs = "với"
lm = "làm"
```

---

## Architecture

```
┌──────────────────┐     ┌──────────────────┐     ┌────────────────┐
│  X11 Keyboard    │────▶│  Viet+ Daemon    │────▶│  X11/XTEST     │
│  Grab (XGrabKb)  │     │                  │     │  Injection     │
│                  │     │  Backspace-Replay│     │                │
│  FocusIn/Out     │     │  Engine          │     │  Direct        │
│  Detection       │     │  (Telex/VNI)     │     │  Clipboard     │
└──────────────────┘     └──────────────────┘     └────────────────┘
                                │
                          ┌─────┴─────┐
                          │  App State │
                          │  Manager   │
                          └───────────┘
```

### How Backspace-Replay Works

1. All keystrokes in the current word are stored in `keystroke_history`
2. On each keypress, a **fresh engine** is created and the entire history is replayed through it
3. The engine's buffer IS what should be on screen
4. Viet+ calculates the diff: backspaces to erase old text + new text to type
5. On flush (space/period/etc.), history is cleared for the next word

This eliminates the state desync bugs that plague incremental engines.

---

## Installation

### Debian/Ubuntu Package

```bash
sudo dpkg -i vietc_0.1.0-1_amd64.deb
```

Recommends: `libxtst6`, `xclip` (for clipboard injection)

### AppImage

```bash
./Viet+-0.1.0-x86_64.AppImage
```

No special permissions needed on X11 — uses XGrabKeyboard + XTest injection.

### Manual Install

```bash
make build-all
sudo make install
```

---

## Building

```bash
# Build all backends (X11 + Wayland)
make build-all

# Run tests (255+ tests)
make test

# Run interactive test harness
cargo run --bin vietc-cli

# Build packages
make deb        # .deb package
make appimage   # AppImage
```

---

## Project Structure

```
vietc/
├── engine/              # Core IME engine (Telex + VNI)
│   ├── src/
│   │   ├── engine.rs    # Main engine + replay_keystrokes()
│   │   ├── telex.rs     # Telex state machine
│   │   ├── vni.rs       # VNI engine
│   │   ├── english.rs   # English auto-restore dictionary
│   │   └── tests/       # 255+ unit tests
│   └── Cargo.toml
├── protocol/            # Injection backends
│   ├── src/
│   │   ├── inject.rs         # KeyInjector trait
│   │   ├── x11_capture.rs    # X11 keyboard capture (XGrabKeyboard)
│   │   ├── x11_inject.rs     # Direct X11 clipboard + XTest injection
│   │   └── wayland_im.rs     # Wayland IM protocol
│   └── Cargo.toml
├── daemon/              # Background daemon
│   ├── src/
│   │   ├── main.rs      # Event loop, Backspace-Replay integration
│   │   ├── config.rs    # TOML config loader
│   │   ├── app_state.rs # Per-app state manager
│   │   └── display.rs   # Display server detection
│   └── Cargo.toml
├── cli/                 # Interactive test harness
├── ui/                  # Tray icon application
├── packaging/           # Distribution packages
│   ├── appimage/        # AppImage build scripts
│   └── deb/             # .deb package build scripts
├── vietc.toml           # Default configuration
├── vietc.service        # Systemd user service
├── Makefile             # Build targets
└── README.md
```

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Made with love for the Vietnamese Linux community</sub>
</p>
