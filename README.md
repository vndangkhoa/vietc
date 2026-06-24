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

Inspired by [Gõ Nhanh](https://github.com/nickel-lang/nickel)'s brilliant UX, rebuilt native for Linux.

---

## Features

| Feature | Description |
|---------|-------------|
| **Direct Input Engine** | No pre-edit buffer, no underline, no text duplication |
| **Telex & VNI** | Both input methods fully supported |
| **Auto-Restore English** | Hit space/ESC to undo accidental Vietnamese conversion |
| **ESC Undo** | Strip all tones from the current word instantly |
| **Smart App Memory** | Remembers Vietnamese/English per application |
| **Macro Expansion** | Custom shortcuts (e.g., `ko` → `không`) |
| **Triple Backend** | uinput (universal), X11 XTEST, Wayland zwp_input_method_v2 |
| **Hot Reload** | Config changes apply without restart |
| **Settings UI** | GTK4/Libadwaita GUI (optional) |
| **System Tray** | KStatusNotifierItem tray app |
| **Zero Telemetry** | No keylogging, no disk writes, fully FOSS |

---

## Quick Start

```bash
# Clone and build
git clone https://github.com/vietplus/vietplus.git
cd vietplus
make build

# Test the engine interactively
make test-cli

# Run the daemon (requires root for evdev/uinput)
sudo make run

# Or install system-wide
sudo make install
```

---

## Input Methods

### Telex (Default)

| Key | Result | Example |
|-----|--------|---------|
| `aa` | ă | `dan` → `dăn` |
| `ee` | ê | `men` → `mên` |
| `oo` | ô | `to` → `tô` |
| `aw` | â | `an` → `ân` |
| `ow` | ô | `on` → `ôn` |
| `ew` | ê | `en` → `ên` |
| `uw` | ư | `un` → `ưn` |
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
| `a6` | ă |
| `a7` | â |
| `e8` | ê |
| `o9` | ô |
| `o0` | ơ |
| `u0` | ư |

---

## Configuration

Config file: `~/.config/vietc/config.toml` or `./vietc.toml`

```toml
input_method = "telex"
toggle_key = "space"
start_enabled = true

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
| Settings UI | `libgtk-4-dev libadwaita-1-dev` | `gtk4-devel libadwaita-devel` | `gtk4 libadwaita` |
| Tray icon | `libdbus-1-dev pkg-config` | `dbus-devel pkgconf` | `dbus pkgconf` |

### Debian/Ubuntu

```bash
make deb
sudo dpkg -i packaging/deb/vietc_0.1.0_amd64.deb
sudo apt-get install -f
```

### AppImage

```bash
make appimage
# Requires appimagetool
appimagetool packaging/appimage/AppDir Viet+-0.1.0-x86_64.AppImage
```

### Arch Linux (AUR)

```bash
cd packaging/aur
makepkg -si
```

### Flatpak

```bash
flatpak-builder --user --install --force-clean build-dir \
  packaging/flatpak/io.github.vietc.VietPlus.json
```

### Manual Install

```bash
sudo make install
sudo make install-ui    # optional
sudo make install-tray  # optional
```

---

## Building

```bash
# Build core (daemon + CLI)
make build

# Build with X11 support
make build-x11

# Build with Wayland IM protocol
make build-wayland

# Build with all backends
make build-all

# Build settings UI (requires GTK4)
make build-ui

# Build tray icon (requires libdbus-1-dev)
make build-tray

# Run tests
make test

# Run interactive test harness
make test-cli
```

---

## Make Targets

| Target | Description |
|--------|-------------|
| `make build` | Build core crates |
| `make build-x11` | Build with X11 support |
| `make build-wayland` | Build with Wayland IM protocol |
| `make build-all` | Build with all backends |
| `make build-ui` | Build settings UI |
| `make build-tray` | Build tray icon app |
| `make test` | Run all tests |
| `make test-cli` | Interactive test harness |
| `make run` | Run daemon (debug) |
| `make install` | Install to /usr/local/bin |
| `make install-x11` | Install with X11 |
| `make install-wayland` | Install with Wayland IM |
| `make install-ui` | Install settings UI |
| `make install-tray` | Install tray icon |
| `make install-all-ui` | Install both UI + tray |
| `make install-config` | Install default config |
| `make appimage` | Build AppImage package |
| `make deb` | Build .deb package |
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
│   │   └── tests.rs       # 124 unit tests
│   └── Cargo.toml
├── protocol/        # Injection backends
│   ├── src/
│   │   ├── inject.rs              # KeyInjector trait
│   │   ├── uinput_monitor.rs      # Universal uinput backend
│   │   ├── x11_inject.rs          # X11 XTEST backend
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
├── ui/              # Settings UI + tray (GTK4/Libadwaita)
│   ├── src/
│   │   ├── main.rs        # Settings app
│   │   ├── window.rs      # Settings window
│   │   ├── tray.rs        # System tray icon
│   │   └── config.rs      # UI config reader
│   └── Cargo.toml
├── packaging/       # Distribution packages
│   ├── aur/         # Arch Linux PKGBUILD
│   ├── flatpak/     # Flatpak manifest
│   ├── appimage/    # AppImage build scripts
│   └── deb/         # Debian package
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
