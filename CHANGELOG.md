# Changelog

## v0.1.0 (2026-06-26)

Initial release and major overhaul.

### Engine (major rewrite)

- **Bamboo engine port** — Replaced custom Telex/VNI state machines with a Rust port of bamboo-core's transformation model. Marks and tones are applied to characters in a composition buffer, with proper tone placement for all Vietnamese diphthongs.
- **Flexible backtracking** — Mark/tone keys scan up to 5 characters backward to find the target vowel. Type the full syllable, then add marks at the end: `nguye6n4` → `nguyễn`.
- **Smart uo→ươ cluster** — Single `w`/`7` key after a `uo` pair converts both to `ươ`, even through consonants: `chuong7` → `chương`.
- **Correct tone placement** — Fixed tone positioning for `io` (gió), `uâ` (xuất), `yê` (nguyễn), `oa`/`oe`, `uy`, `iê`, `uô`, `ươ` clusters.
- **Consume stale marks** — VNI/Telex control keys (digits, `f`/`s`/`r`/`x`/`j`/`w`) are consumed silently when they produce no change (e.g., pressing `5` on an already-toned `ạ`).
- **63 focused unit tests** covering Telex, VNI, tone placement, marks, macros, and uppercase.

### Injection (major overhaul)

- **Uinput injection** — ASCII and backspace via Linux evdev keycodes (`/dev/uinput`). Correct keycodes per keyboard hardware, no X11 keycode mismatches.
- **Vietnamese Unicode** — Clipboard paste via persistent X11 connection + XTest Ctrl+V. Text is split only at trailing whitespace/punctuation boundary (no mid-word splitting). Persistent X11 display opened once and reused.
- **Uinput daemon** (`vietc-uinputd`) — Privileged Unix socket server for `/dev/uinput` injection. VMK-style architecture with capability separation. The main daemon communicates via socket, falling back to in-process uinput.
- **X11Injector** uses `XKeysymToKeycode` for Ctrl+V keycodes, adapting to the actual keyboard layout.

### Capture

- **Evdev preferred** — Keyboard capture via `/dev/input/event*` with device grab is now the primary path. More reliable than X11 XRecord.
- **X11 XRecord fallback** — X11 passive monitoring via C helper (`vietc-xrecord`) as fallback when evdev is unavailable.

### Bug Fixes

- **Fix `Xutf8LookupString` signature** — Missing `XIC` parameter caused all keycodes to map to `\0`. Fixed by adding `*mut c_void` as first argument and passing `NULL`.
- **Fix `execute_commands` backspace count** — The X11 path incorrectly passed `grabbed=true`, subtracting 1 from every backspace. Changed to `false` so full backspace count is used.
- **Fix flush backspace overcount** — `prev_len + 1` erased one character beyond the word. Fixed to `prev_len`.
- **Fix `apply_mark` char removal** — Removed `pattern.len()` chars from composition, but the current key hadn't been appended yet. Fixed to `pattern.len() - 1`.
- **Fix mark backtrack position** — Marks were applied at the end of composition instead of at the found position. Added position-aware `apply_mark_at`.

### Packaging

- AppImage bundles `vietc-uinputd`, `vietc-xrecord`, `xclip`.
- AppRun preserves `LD_LIBRARY_PATH` with system library paths for `dlopen`.
- AppRun auto-starts `vietc-uinputd` via `pkexec`/`sudo` when available.
- Cleaned up `vietc-xrecord` compilation flags (only `-lX11 -lXtst` needed).

### Testing

- 63 focused engine tests covering Telex, VNI, marks, tones, macros, casing.
- Removed old auto-generated bulk tests (850+ tests for deprecated engine).
