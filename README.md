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
  <sub>Zero underline &bull; Native Wayland/X11 &bull; Built in Rust</sub>
</p>

<p align="center">
  <a href="#features">Features</a> &bull;
  <a href="#quick-start">Quick Start</a> &bull;
  <a href="#input-methods">Input Methods</a> &bull;
  <a href="#configuration">Configuration</a> &bull;
  <a href="#installation">Installation</a> &bull;
  <a href="#building">Building</a>
</p>

---

## Why Viet+?

Most Vietnamese input methods on Linux suffer from **underline hell** — pre-edit buffers that duplicate text, show ugly underlines, and break your flow. Viet+ takes a different approach:

> **Direct Input** — keystrokes are instantly converted to Unicode. No pre-edit buffer. No underline. No text duplication. Just pure Vietnamese.

---

## Features

| Feature | Description |
|---------|-------------|
| **Direct Input Engine** | No pre-edit buffer, no underline, no text duplication |
| **Telex & VNI** | Both input methods fully supported |
| **Flexible Diacritic Placement** | Type modifiers/tone marks at end of syllable (e.g., `tranaf` → `trần`) |
| **Auto-Restore English** | Hit space/ESC to undo accidental Vietnamese conversion |
| **ESC Undo** | Strip all tones from the current word instantly |
| **Smart App Memory** | Remembers Vietnamese/English per application |
| **Macro Expansion** | Custom shortcuts (e.g., `ko` → `không`) |
| **Unified Injection** | Unified channel backspace and typing injection to prevent ordering race conditions |
| **Focus Buffer Auto-Reset** | Automatically clears the engine's compose buffer on focus change between apps |
| **Logging & Rotation** | Persistent logging at `~/.config/vietc/vietc.log` with automatic 10MB rotation |
| **Hot Reload** | Config changes apply without restart |
| **Casing Preservation** | Syllable substitutions preserve your exact typing casing (e.g. `Saa` → `Sả`, `SAA` → `SẢ`) |
| **High-Performance Injection** | Direct native setuid/setgid user context switching to run injection tools instantly with no slow sudo/PAM overhead |
| **Zero Telemetry** | No keylogging, no network calls, fully FOSS |

---

## Quick Start

```bash
# Clone and build
git clone https://git.khoavo.myds.me/vndangkhoa/vietc.git
cd vietc
make build

# Test the engine interactively
cargo run --bin vietc-cli

# Run the daemon (requires root for keyboard grab + uinput)
sudo make run

# Or use the AppImage
sudo ./Viet+-0.1.0-x86_64.AppImage
```

---

## Input Methods

### Telex (Default)

| Key | Result | Example |
|-----|--------|---------|
| `aa` | â | `tan` → `tân` |
| `aw` | ă | `tan` → `tăn` |
| `ee` | ê | `men` → `mên` |
| `oo` | ô | `to` → `tô` |
| `ow` | ơ | `to` → `tơ` |
| `ew` | ê | `en` → `ên` |
| `uw` | ư | `tu` → `tư` |
| `s` | á (sắc) | `as` → `á` |
| `f` | à (huyền) | `af` → `à` |
| `r` | ả (hỏi) | `ar` → `ả` |
| `x` | ã (ngã) | `ax` → `ã` |
| `j` | ạ (nặng) | `aj` → `ạ` |
| `dd` | đ | `dd` → `đ` |

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
input_method = "telex"
toggle_key = "space"
start_enabled = true
debug = false

[auto_restore]
enabled = true

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
┌──────────────┐     ┌──────────────┐     ┌────────────────┐
│  evdev       │────▶│  Viet+       │────▶│  uinput/X11    │
│  keyboard    │     │  Engine      │     │  injection     │
│  monitor     │     │  (Telex/VNI) │     │                │
└──────────────┘     └──────────────┘     └────────────────┘
                           │
                     ┌─────┴─────┐
                     │  App State │
                     │  Manager   │
                     └───────────┘
```

---

## Installation

### System Dependencies

| Component | Ubuntu/Debian | Fedora | Arch |
|-----------|--------------|--------|------|
| Core daemon | *(none)* | *(none)* | *(none)* |
| Tray icon | `libdbus-1-dev pkg-config` | `dbus-devel pkgconf` | `dbus pkgconf` |

### AppImage (recommended)

```bash
make appimage
# Requires appimagetool
```

The AppImage bundles all dependencies. Run with `sudo` for keyboard grab:

```bash
sudo ./Viet+-0.1.0-x86_64.AppImage
```

### Manual Install

```bash
sudo make install
sudo make install-tray  # optional
```

---

## Building

```bash
# Build all backends (uinput + X11 + Wayland)
make build-all

# Run tests (162+ engine tests)
make test

# Run interactive test harness
cargo run --bin vietc-cli

# Build AppImage
make appimage
```

---

## Make Targets

| Target | Description |
|--------|-------------|
| `make build-all` | Build all backends (uinput + X11 + Wayland) |
| `make test` | Run all tests |
| `make run` | Run daemon (debug, requires root) |
| `make appimage` | Build AppImage package |
| `make clean` | Clean build artifacts |
| `make fmt` | Format code |
| `make lint` | Run clippy |

---

## Project Structure

```
viet+/
├── engine/          # Core IME engine (Telex + VNI)
│   ├── src/
│   │   ├── engine.rs      # Main engine orchestrator
│   │   ├── telex.rs       # Telex state machine
│   │   ├── vni.rs         # VNI engine
│   │   ├── english.rs     # English auto-restore dictionary
│   │   └── tests.rs       # 162+ unit tests
│   └── Cargo.toml
├── protocol/        # Injection backends
│   ├── src/
│   │   ├── inject.rs              # KeyInjector trait
│   │   ├── uinput_monitor.rs      # Universal uinput+ydotool backend
│   │   ├── x11_inject.rs          # X11 XTEST fallback
│   │   └── wayland_im.rs          # Wayland IM context
│   └── Cargo.toml
├── daemon/          # Background daemon
│   ├── src/
│   │   ├── main.rs        # Evdev loop, hot-reload
│   │   ├── config.rs      # TOML config loader
│   │   ├── app_state.rs   # Per-app state manager
│   │   └── display.rs     # Display server detection
│   └── Cargo.toml
├── cli/             # Interactive test harness
├── ui/              # Tray icon application
│   ├── src/
│   │   ├── main.rs        # Tray app entry point
│   │   ├── tray.rs        # System tray icon implementation
│   │   └── config.rs      # UI config reader
│   └── Cargo.toml
├── packaging/       # Distribution packages
│   └── appimage/    # AppImage build scripts
├── vietc.toml       # Default configuration
├── vietc.service    # Systemd user service
├── Makefile         # Build targets
└── README.md
```

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Made with ❤️ for the Vietnamese Linux community</sub>
</p>
