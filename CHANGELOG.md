# Changelog

## v0.1.3 (2026-06-26)

### Clipboard & Injection Fixes
- **ua-horn cluster fix** — Correct tone placement for `ưa` clusters (mưa, lửa).
- **Word-spacing fix** — Clipboard operations use `clipboard_context` to save/restore user's clipboard.
- **Control-key consumption** — VNI/Telex control keys properly consumed across all code paths.
- **Clipboard preservation** — User's clipboard saved before injection and restored after, preventing Ctrl+C/V conflicts.

### Tests
- **106 tests** passing (72 engine + 16 CLI + 12 protocol + 5 auto-restore + 1 tone placement).

### Releases
- `vietc_0.1.3-1_amd64.deb` (976K), `Viet+-0.1.3-x86_64.AppImage` (2.2M) on GitHub + Forgejo.

---

## v0.1.2 (2026-06-26)

- Flush char forwarded as raw key, stop retyping finished word on flush
- Auto-restore English words, tone placement for qu/gi/uê/uơ
- Skip auto-repeat, Enter key support, removed xdotool
- 102 tests, AppImage `--quit`/`--restart`/`--update` flags

---

## v0.1.1 (2026-06-26)

- Fix `r` consumed as tone key, fix normal letters consumed in Telex
- 15ms delay, persistent X11 connection, Enter key
- 67 engine tests

---

## v0.1.0 (2026-06-26)

Initial release — bamboo engine port, evdev capture, uinput injection.
