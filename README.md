<p align="center">
  <img src="https://img.shields.io/badge/Platform-Linux-blue?style=for-the-badge" alt="Platform">
  <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/Version-0.1.6-purple?style=for-the-badge" alt="Version">
  <img src="https://img.shields.io/badge/Tests-106_passing-brightgreen?style=for-the-badge" alt="Tests">
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

Most Vietnamese IMEs use a **pre-edit buffer** — you type into a temporary buffer with an ugly underline, and the text only becomes real Vietnamese when you commit it. This causes:

- Duplicate text (buffer + committed)
- Underline distraction
- Broken copy/paste
- Desync between engine state and what's on screen

Viet+ eliminates all of this. Keystrokes are **instantly converted to Unicode** — what you type is what you see. No buffer. No underline. No duplication.

---

## How It Works

### Data Flow: Keypress to Screen

```
Physical Keyboard
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│  Stage 1: KEY CAPTURE                                        │
│                                                              │
│  evdev: /dev/input/event* grabs keyboard (primary, reliable) │
│  X11: XRecord passive monitoring (fallback)                  │
│                                                              │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ evdev grab  │  │ X11Capture   │  │ Window switch    │   │
│  │ (libevdev)  │  │ (XRecord)    │  │ detection (250ms)│   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
└──────────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│  Stage 2: KEY ROUTING                                        │
│                                                              │
│  Modifier keys (Ctrl/Alt/Super) → forward directly           │
│  Ctrl+Space → toggle Vietnamese ON/OFF                       │
│  Backspace → replay_backspace()                              │
│  Characters → replay_and_inject(ch)                          │
│  VNI control keys → consume when no match                    │
└──────────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│  Stage 3: BAMBOO ENGINE                                      │
│                                                              │
│  Transformation model: keystrokes produce composition        │
│  changes. Marks and tones modify existing characters.        │
│  Flexible backtracking scans up to 5 chars for vowels.       │
│  Smart uo→ươ cluster with backtrack.                         │
│  Only emits Replace events when output actually changes.     │
└──────────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│  Stage 4: KEY INJECTION                                      │
│                                                              │
│  Primary: uinput injection (evdev keycodes, correct on all   │
│    display servers — routed through libinput on modern X11)  │
│  ASCII: direct Linux keycodes via /dev/uinput                │
│  Backspace: Linux keycode 14 via uinput                      │
│  Vietnamese Unicode: clipboard paste + trailing ASCII via    │
│    uinput (split only at whitespace/punctuation boundary)    │
│  uinput Ctrl+V via /dev/uinput (no X11 dependency)           │
│                                                              │
│  Fallback: X11 XTest injection (X11 keycodes = evdev + 8)    │
└──────────────────────────────────────────────────────────────┘
       │
       ▼
   Application receives keystrokes
   and renders Vietnamese text on screen
```

### Event Sourcing + Backspace-Replay

This is Viet+'s core innovation. Traditional IMEs track state incrementally — each keystroke updates an internal buffer. But this buffer can **desync** from what's actually on screen (due to focus changes, external pastes, etc.).

Viet+ uses **Event Sourcing**: every input action is recorded as a typed `InputEvent` (`KeyTyped`, `Backspace`, `Flush`, `Paste`) in an `EventStore`. On every keystroke, the entire event history is **replayed from scratch** through a fresh engine to compute the correct diff — no incremental state to desync.

```
Traditional IME:
  keystroke → update buffer → emit event → hope it matches screen
  
Viet+ (Event Sourcing):
  keystroke → append InputEvent → replay ALL events in fresh engine → compute diff
```

On every keystroke:

1. The keystroke is appended as an `InputEvent` to the `EventStore`
2. A **brand new** `Engine` is created
3. The **entire** event history is replayed through it via `Engine::replay_events()`
4. The engine's buffer is the **correct** screen output
5. Viet+ computes the diff: `Engine::replay_events_to_commands()` returns Type/Backspace commands

This means:
- **Zero state desync** — always recomputed from scratch
- **Self-healing** — if anything goes wrong, the next keystroke fixes it
- **Privacy-safe** — `EventStore::pattern_hash()` provides a sha256 of the event type sequence for pattern detection without any ability to recover original text
- **Simple** — no complex state tracking or synchronization

---

## Architecture

```
vietc/
├── engine/                  # Vietnamese composition engine (bamboo-core Rust port)
│   ├── engine.rs            # Orchestrator + replay_events(), replay_events_to_commands()
│   ├── event.rs             # Event Sourcing: InputEvent, EventStore, Command
│   ├── bamboo.rs            # Bamboo engine: transformation model, composition, tone placement
│   ├── input_method.rs      # VNI rule definitions
│   └── spelling.rs          # Vietnamese syllable validation
│
├── protocol/                # Keyboard capture & injection
│   ├── inject.rs            # KeyInjector trait
│   ├── x11_capture.rs       # XRecord keyboard capture via C helper
│   ├── x11_inject.rs        # XTest injection (fallback)
│   ├── uinput_monitor.rs    # /dev/uinput injection (primary)
│   ├── uinput_client.rs     # Unix socket client for vietc-uinputd
│   └── wayland_im.rs        # Wayland IM protocol
│
├── daemon/                  # Main daemon process
│   ├── main.rs              # Event loops, Backspace-Replay, CPU pinning
│   ├── config.rs            # TOML config loader + hot reload
│   ├── app_state.rs         # Per-app Vietnamese/English memory
│   └── display.rs           # X11/Wayland/compositor detection
│
├── uinputd/                 # Privileged uinput backspace daemon (VMK-style)
│   └── main.rs              # Unix socket server for /dev/uinput injection
│
├── ui/                      # System tray icon
│   └── main.rs              # Tray + daemon launcher
│
├── cli/                     # Interactive test harness
├── packaging/               # .deb packaging scripts
└── vietc.toml               # Default configuration
```

### Component Interaction

```
┌─────────────────────────────────────────────────────────────┐
│                      vietc-tray                             │
│  (System tray icon, daemon launcher)                        │
└───────────────────────┬─────────────────────────────────────┘
                        │ starts
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                      vietc-daemon                            │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Config       │  │ App State    │  │ Display          │  │
│  │ (hot reload) │  │ (per-app)    │  │ (X11/Wayland)    │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │             │
│         └─────────────────┼────────────────────┘             │
│                           │                                  │
│                    ┌──────▼──────┐                           │
│                    │ Event Loop  │                           │
│                    │             │                           │
│                    │ evdev: grab │                           │
│                    │ keyboard    │                           │
│                    │             │                           │
│                    │ Process     │                           │
│                    │ keystroke   │                           │
│                    │             │                           │
│                    │ Replay all  │                           │
│                    │ history     │                           │
│                    │             │                           │
│                    │ Inject      │                           │
│                    │ diff        │                           │
│                    └─────────────┘                           │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                   vietc-engine                         │ │
│  │  VniEngine / EnglishDict / Spelling                    │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │               vietc-protocol                           │ │
│  │  UinputInjector / X11Injector / X11Capture / Wayland  │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Input Methods

### VNI (default, Telex coming in next version)

| Key | Result | Example |
|-----|--------|---------|
| `1` | á (sắc) | `a1` → `á` |
| `2` | à (huyền) | `a2` → `à` |
| `3` | ả (hỏi) | `a3` → `ả` |
| `4` | ã (ngã) | `a4` → `ã` |
| `5` | ạ (nặng) | `a5` → `ạ` |
| `6` | â/ê/ô | `a6` → `â`, `e6` → `ê`, `o6` → `ô` |
| `7` | ơ/ư | `o7` → `ơ`, `u7` → `ư` |
| `8` | ă | `a8` → `ă` |
| `9` | đ | `d9` → `đ` |

Flexible typing: type the full syllable, then add marks/tone keys at the end. Example: `nguye6n4` → `nguyễn`. The engine scans backward up to 5 characters to find the target vowel.

---

## Features

| Feature | How It Works |
|---------|-------------|
| **Direct Input** | No pre-edit buffer. Keystrokes instantly become text via uinput injection |
| **Bamboo Engine** | Transformation model ported from bamboo-core — composition, marks, tones, flexible backtracking |
| **Flexible Backtrack** | Type tone/modifier at end of syllable (`tran5` → `trạn`). Scans up to 5 chars backward |
| **Smart Clusters** | `uo` → `ươ` with backtrack (`chuong7` → `chương`) |
| **Tone Placement** | Correct tone positioning for all Vietnamese diphthongs (io→gió, uâ→xuất, yê→nguyễn) |
| **Macro Expansion** | `ko` → `không`, `dc` → `được`, custom shortcuts |
| **Casing Preservation** | `Tieengs` → `Tiếng`, `TIEENGS` → `TIẾNG` |
| **App Memory** | Per-app Vietnamese/English state, saved to `overrides.toml` |
| **Hot Reload** | Config changes apply without restart (polls mtime every 1.5s) |
| **Window-Switch Reset** | Active window ID verified on every keystroke — Alt+Tab instantly clears engine state. No stale composition across apps |
| **CPU Priority** | Pins daemon to P-cores (0-3) + nice(-10) for low-latency input |
| **Uinput Injection** | Uses `/dev/uinput` for reliable keyboard injection without X11 dependency. Falls back to XTest on systems without uinput access |

---

## Installation

### Debian Package (recommended)

System tray icon + daemon + desktop entry. Requires user to be in the `input` group for keyboard capture.

```bash
# Install
sudo dpkg -i vietc_0.1.6-1_amd64.deb

# Log out and log back in (for input group membership to take effect)
# Then launch "Viet+" from your application menu
```

The post-install script will:
- Kill any running tray/daemon processes
- Remove stale binaries from `/usr/local/bin/`
- Add your user to the `input` group
- Prompt you to log out and back in

### Build from Source

```bash
git clone https://github.com/vndangkhoa/vietc.git
cd vietc
make deb
sudo dpkg -i packaging/deb/vietc_0.1.6-1_amd64.deb
```

Requires Rust toolchain, `pkg-config`, `libx11-dev`, `libxtst-dev`, `libevdev-dev`. See `packaging/deb/build-deb.sh` for details.

---

## Configuration

Config file: `~/.config/vietc/config.toml` or `./vietc.toml`

```toml
input_method = "vni"       # "vni" or "telex"
toggle_key = "space"       # Ctrl+Space to toggle
start_enabled = true       # Vietnamese by default
grab = true                # grab keyboard (evdev)

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

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Made with love for the Vietnamese Linux community</sub>
</p>
