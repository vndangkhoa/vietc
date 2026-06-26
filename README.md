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
│  X11: XGrabKeyboard intercepts all key events                │
│  evdev: /dev/input/event* reads kernel events                │
│                                                              │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ X11Capture  │  │ evdev grab   │  │ FocusIn/FocusOut │   │
│  │ (libX11.so) │  │ (libevdev)   │  │ detection        │   │
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
└──────────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│  Stage 3: BACKSPACE-REPLAY                                   │
│                                                              │
│  keystroke_history = ['c', 'h', 'a', 'o', 's']             │
│                       │                                      │
│                       ▼                                      │
│  ┌──────────────────────────────────────────────┐           │
│  │  Create FRESH engine                         │           │
│  │  Replay ALL keystrokes through it            │           │
│  │  engine.buffer() = "cháo"  ← correct output │           │
│  └──────────────────────────────────────────────┘           │
│                       │                                      │
│                       ▼                                      │
│  screen_output = "cháo"                                     │
│  diff = backspaces(0) + type("cháo")                        │
│  (or no change if screen already shows "cháo")              │
└──────────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│  Stage 4: OUTPUT COMMANDS                                    │
│                                                              │
│  EngineEvent::Replace { backspaces: 4, insert: "cháo" }     │
│       │                                                      │
│       ▼                                                      │
│  OutputCommand::Backspace(4)                                 │
│  OutputCommand::Type("cháo")                                 │
└──────────────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│  Stage 5: KEY INJECTION                                      │
│                                                              │
│  X11 path:                                                   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  1. Ungrab keyboard (XUngrabKeyboard)               │    │
│  │  2. Send backspaces via XTestFakeKeyEvent           │    │
│  │  3. Set clipboard via XChangeProperty               │    │
│  │  4. Handle SelectionRequest events                  │    │
│  │  5. Send Ctrl+V via XTestFakeKeyEvent               │    │
│  │  6. Regrab keyboard (XGrabKeyboard)                 │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
│  uinput path (Wayland):                                      │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  1. Send backspaces via /dev/uinput (EV_KEY 14)     │    │
│  │  2. For ASCII: send keycodes via uinput              │    │
│  │  3. For Unicode: wl-copy + Ctrl+V via uinput         │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
       │
       ▼
   Application receives keystrokes
   and renders Vietnamese text on screen
```

### The Backspace-Replay Pattern

This is Viet+'s core innovation. Traditional IMEs track state incrementally — each keystroke updates an internal buffer. But this buffer can **desync** from what's actually on screen (due to focus changes, external pastes, etc.).

Viet+ solves this by **never tracking incremental state**:

```
Traditional IME:
  keystroke → update buffer → emit event → hope it matches screen
  
Viet+ (Backspace-Replay):
  keystroke → add to history → replay ALL history in fresh engine → compute diff
```

On every keystroke:

1. The keystroke is appended to `keystroke_history`
2. A **brand new** `Engine` is created
3. The **entire** history is replayed through it
4. The engine's buffer is the **correct** screen output
5. Viet+ computes the diff: how many backspaces to erase old text, what new text to type

This means:
- **Zero state desync** — always recomputed from scratch
- **Self-healing** — if anything goes wrong, the next keystroke fixes it
- **Simple** — no complex state tracking or synchronization

---

## Architecture

```
vietc/
├── engine/                  # Core Vietnamese composition engine
│   ├── engine.rs            # Orchestrator + replay_keystrokes()
│   ├── telex.rs             # Telex state machine (688 lines)
│   ├── vni.rs               # VNI state machine (593 lines)
│   ├── english.rs           # English auto-restore dictionary
│   └── spelling.rs          # Vietnamese syllable validation
│
├── protocol/                # Keyboard capture & injection
│   ├── inject.rs            # KeyInjector trait
│   ├── x11_capture.rs       # XGrabKeyboard + XNextEvent loop
│   ├── x11_inject.rs        # XTest injection + direct clipboard
│   └── wayland_im.rs        # Wayland IM protocol
│
├── daemon/                  # Main daemon process
│   ├── main.rs              # Event loops, Backspace-Replay, CPU pinning
│   ├── config.rs            # TOML config loader + hot reload
│   ├── app_state.rs         # Per-app Vietnamese/English memory
│   └── display.rs           # X11/Wayland/compositor detection
│
├── ui/                      # System tray icon
│   └── main.rs              # Tray + daemon launcher
│
├── cli/                     # Interactive test harness
├── packaging/               # AppImage + deb build scripts
└── vietc.toml               # Default configuration
```

### Component Interaction

```
┌─────────────────────────────────────────────────────────────┐
│                      vietc-tray                             │
│  (System tray icon, daemon launcher, password prompt)       │
└───────────────────────┬─────────────────────────────────────┘
                        │ starts
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                      vietc (daemon)                          │
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
│                    │ X11: grab   │                           │
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
│  │  TelexEngine / VniEngine / EnglishDict / Spelling     │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │               vietc-protocol                           │ │
│  │  X11Capture / X11Injector / UinputInjector / Wayland  │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Input Methods

### Telex

| Key | Result | Example |
|-----|--------|---------|
| `aa` | â | `tan` → `tân` |
| `aw` | ă | `tan` → `tăn` |
| `ee` | ê | `men` → `mên` |
| `oo` | ô | `to` → `tô` |
| `ow` | ơ | `to` → `tơ` |
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

## Features

| Feature | How It Works |
|---------|-------------|
| **Direct Input** | No pre-edit buffer. Keystrokes instantly become Unicode via XTest/uinput injection |
| **Backspace-Replay** | Replays entire keystroke history in a fresh engine on every keypress — zero state desync |
| **Flexible Placement** | Type tone/modifier at end of syllable (`tranaf` → `trần`) — engine scans backward to find the vowel |
| **Smart Clusters** | `uo` → `ươ`, `ươ` + `o` → `uô`, shape modifier overriding (â↔ă, ô↔ơ) |
| **Auto-Restore** | ~250 English words recognized — typing `hello` won't become Vietnamese. Triggered on space/ESC |
| **ESC Undo** | Strip all tones from current word instantly |
| **Macro Expansion** | `ko` → `không`, `dc` → `được`, custom shortcuts |
| **Casing Preservation** | `SATS` → `SÁT`, `Saa` → `Sả` — matches your typing pattern |
| **App Memory** | Per-app Vietnamese/English state, saved to `overrides.toml` |
| **Hot Reload** | Config changes apply without restart (polls mtime every 1.5s) |
| **Focus Reset** | FocusIn/FocusOut clears engine state — no stale injection on window switch |
| **CPU Priority** | Pins daemon to P-cores (0-3) + nice(-10) for low-latency input |

---

## Installation

### AppImage (recommended)

```bash
./Viet+-0.1.0-x86_64.AppImage
```

Includes daemon + tray + CLI + xclip. No special permissions needed on X11.

### Debian/Ubuntu

```bash
sudo dpkg -i vietc_0.1.0-1_amd64.deb
```

Recommends: `libxtst6`, `xclip`

### Manual

```bash
git clone https://git.khoavo.myds.me/vndangkhoa/vietc.git
cd vietc
make build-all
sudo make install
```

---

## Configuration

Config file: `~/.config/vietc/config.toml` or `./vietc.toml`

```toml
input_method = "vni"       # "vni" or "telex"
toggle_key = "space"       # Ctrl+Space to toggle
start_enabled = false      # English by default
grab = true                # grab keyboard (AppImage)

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

## Building

```bash
make build-all     # Build with X11 + Wayland
make test          # Run 255+ tests
make deb           # Build .deb package
make appimage      # Build AppImage
```

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Made with love for the Vietnamese Linux community</sub>
</p>
