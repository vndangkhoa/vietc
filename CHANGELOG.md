# Changelog

## v0.1.2 (2026-06-26)

### Telex & Spacing Fixes
- **ua-horn cluster fix** — Correct tone placement for `ưa` clusters (mưa, lửa).
- **Word-spacing fix** — Clipboard operations now properly preserve user's clipboard content during injection by saving and restoring via `clipboard_context`.
- **Control-key consumption fix** — VNI/Telex control keys properly consumed across all code paths.
- **Clipboard preservation** — User's clipboard is saved before daemon injection and restored after, preventing Ctrl+C/V conflicts.

### Flush & Spacing
- **Flush char forwarded as raw key** — Engine no longer includes flush char in Replace insert. Daemon forwards it as raw keycode after injection.
- **Stop retyping finished word on flush** — Characters already on screen stay, only the flush key is typed.
- **Auto-restore English words** — Common English words and invalid Vietnamese syllables are restored on space.

### Tone Placement
- **qu/gi onset glides** — Correct tone for `qu` (quý), `gi` (gió) clusters.
- **uê/uơ clusters** — Correct tone on second vowel for `uê` (thuế), `uơ` (thuở).

### Injection
- **Skip auto-repeat** — Skip 3 auto-repeat events after injection to prevent key flood.
- **Enter key** — `\n` sent as KEY_ENTER via uinput.
- **Removed xdotool** — Layout-dependent; reverted to clipboard paste.
- **Uinput daemon** improvements for clipboard-aware injection.

### AppImage
- `--quit`, `--restart`, `--update` flags.
- xdotool bundled for future use.

### Tests
- **106 tests** (72 engine + 16 CLI + 12 protocol + 5 auto-restore + 1 tone placement).

### Releases
- `vietc_0.1.2-1_amd64.deb` (975K), `Viet+-0.1.2-x86_64.AppImage` (2.2M) on GitHub + Forgejo.

---

## v0.1.1 (2026-06-26)

### Telex fixes
- Fix `r` consumed as tone key — tone keys check `has_vowel` before applying.
- Fix normal letters consumed — `is_vn_control_key` was eating `a`,`e`,`o`,`d`,`u`.
- 67 engine tests.

### Injection
- 15ms delay, persistent X11 connection, Enter key support.
- `--quit`, `--restart`, `--update` flags.

---

## v0.1.0 (2026-06-26)

Initial release — bamboo engine port, evdev capture, uinput injection.
