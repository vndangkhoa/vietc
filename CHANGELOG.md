# Changelog

## v0.1.0 (2026-06-26)

Initial release.

### Engine
- Direct Input engine — no pre-edit buffer, no underline, no text duplication
- Telex and VNI input methods
- Flexible diacritic placement (tone/modifier at end of syllable)
- Auto-restore English words on space/ESC
- ESC undo (strip all tones from current word)
- Macro expansion with custom shortcuts
- Casing preservation (titlecase, uppercase)

### Injection
- X11 keyboard capture via XGrabKeyboard (no root, no /dev/input)
- Direct X11 clipboard injection (XSetSelectionOwner + XTest Ctrl+V)
- Bundled xclip + wl-copy for Wayland fallback
- Unified injection channel to prevent ordering race conditions

### Daemon
- **Backspace-Replay pattern** — replays entire keystroke history through a fresh engine on every keypress, eliminating state desync
- FocusIn/FocusOut detection for automatic engine reset
- CPU pinning to P-cores (0-3) + nice(-10) priority boost
- Hot-reload config without restart
- Smart app memory (per-application Vietnamese/English)
- Persistent logging with 10MB rotation

### Packaging
- AppImage with bundled xclip (no manual setup needed)
- Debian package with proper conffiles, maintainer scripts, and lintian overrides
- Systemd user service

### Testing
- 255+ unit tests across engine, protocol, daemon config, and replay
