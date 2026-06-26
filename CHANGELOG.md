# Changelog

## v0.1.2 (2026-06-26)

### Flush & Spacing
- **Flush char forwarded as raw key** — Engine no longer includes flush char (space, enter, punctuation) in Replace insert text. Daemon forwards it as a raw keycode after injection, preventing clipboard paste from trimming trailing spaces.
- **Stop retyping finished word on flush** — Flush no longer erases and retypes the entire word. Characters already on screen stay, only the flush key is typed.
- **Auto-restore English words** — Recognizes common English words and invalid Vietnamese syllables. When typing `hello` followed by space, the word is restored if the engine incorrectly applied Vietnamese marks.

### Tone Placement
- **qu/gi onset glides** — Correct tone placement for `qu` (quý, quả) and `gi` (gió, giờ) clusters.
- **uê/uơ clusters** — Correct tone on second vowel for `uê` (thuế) and `uơ` (thuở).

### Injection Fixes
- **Skip auto-repeat pile-up** — After each injection, skip 3 auto-repeat events (value=2) to prevent `rrrrrrrr` from flooding the output during injection delay.
- **Enter key support** — `\n` character now sent as `KEY_ENTER` via uinput. Fixes Enter requiring double-press.
- **Removed clipboard save/restore** — The `xclip -o` read was leaking content into text. Simple clipboard write+paste is sufficient.
- **Removed xdotool approach** — xdotool type depends on keyboard layout and fails on US layout. Reverted to clipboard paste which is layout-independent.

### AppImage
- **`--quit` / `--restart` / `--update` flags** — CLI control over daemon lifecycle and self-updating from GitHub releases.
- **xdotool bundling** — Bundled in AppImage for future use (not active yet).

### Engine Tests
- **102 total tests** — 71 engine + 13 CLI + 12 protocol + 5 auto-restore + 1 tone placement.
- New: `tone_placement.rs` (qu/gi/gio/uê/uơ clusters), `auto_restore.rs` (5 tests).

### DEB & AppImage
- `vietc_0.1.2-1_amd64.deb` (975K), `Viet+-0.1.2-x86_64.AppImage` (2.2M) published on GitHub and Forgejo.

---

## v0.1.1 (2026-06-26)

### Telex fixes
- **Fix `r` consumed as tone key** — Telex tone keys (`f`,`s`,`r`,`x`,`j`) now only activate when the composition has a vowel.
- **Fix normal letters consumed** — `is_vn_control_key` was consuming `a`,`e`,`o`,`d`,`u` in Telex mode.
- **Tone key context check** — Tone keys check `has_vowel` before applying.

### Injection
- **15ms delay** between clipboard paste and trailing uinput ASCII.
- **Persistent X11 connection** for Ctrl+V via `std::sync::Once`.
- **Enter key** sends `KEY_ENTER` via uinput.

### AppImage
- `--quit`, `--restart`, `--update` flags. GUI quit dialog.
- 67 engine tests.

---

## v0.1.0 (2026-06-26)

Initial release and major overhaul.
