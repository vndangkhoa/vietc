# Terminal VNI Input — Design & Implementation

## Goal

Make Vietnamese input work in terminal emulators without breaking TUI keyboard shortcuts.

## Approach: A + C

### A — Remove terminals from `bypass_apps`

All terminals are currently in `bypass_apps` (default config), which skips ALL engine
processing when the active window is a terminal. Removing them lets keystrokes flow
through the bamboo engine.

### C — Force VNI when terminal detected

When the active window is a terminal, the engine automatically uses VNI rules
(`1-9` for tones/marks) regardless of the global VNI/Telex setting.
This avoids key conflicts with TUI apps (vim's `j`, less's `s`, shell's `x`, etc.).

## How It Works

```
User config: input_method = "telex"
Terminal window focused  → effective method = "vni"   (forced by terminal_apps)
GUI window focused       → effective method = "telex" (user's global setting)
```

- **Engine** runs with effective method
- **Tray** shows global method (so user sees their configured setting)
- **Ctrl+LeftShift** toggles global method, recomputes effective method
- **Ctrl+Space** toggles VN/EN as before

## Config Changes

```toml
[app_state]
terminal_apps = ["kitty", "alacritty", "foot", "wezterm", "konsole",
  "gnome-terminal", "gnome-terminal-server", "kgx", "st", "urxvt", "xterm",
  "termite", "terminator", "tilix", "deepin-terminal", "pantheon-terminal"]
terminal_input_method = "vni"
```

`bypass_apps` reduced to: `["steam", "dota", "csgo", "minecraft", "factorio"]`

## Implementation

### 1. `daemon/src/config.rs`

- Add `terminal_apps` (`Vec<String>`) and `terminal_input_method` (`String`) to `AppStateConfig`
- Add `default_terminal_apps()` returning the terminal list
- Add `default_terminal_method()` returning `"vni"`
- Remove all terminal names from `default_bypass_apps()`

### 2. `daemon/src/app_state.rs`

- Add fields: `terminal_apps`, `terminal_input_method`, `global_method`, `effective_method`
- `new()` accepts `terminal_apps`, `terminal_input_method`, `global_method`
- `update_effective_method()`: if current_app matches any terminal, effective = terminal method; else effective = global method. Called on window change.
- `set_terminal_config()`: updates terminal_apps/terminal_input_method from config reload
- `set_global_method()`: updates global_method, recomputes effective
- `effective_method()` getter
- `is_terminal_app()` — checks if current_app is a terminal
- `update_with_app()` calls `update_effective_method()` internally
- `update_lists()` also handles terminal_apps

### 3. `daemon/src/main.rs`

- `Daemon::new()` — pass terminal config to `AppStateManager`, call `update_effective_method()`
- `toggle_method()` — after toggling global method, call `app_state.set_global_method()` then `engine.set_method(app_state.effective_method())`
- `check_app_change_with()` — after app change, if effective method changed from engine's current, call `engine.set_method(effective)`
- `is_vn_control_key()` calls — change from `daemon.config.input_method` to `daemon.app_state.effective_method()`
- Config reload — update `update_lists()` call to include terminal fields
- Method status file — still writes **global** method (for tray display)

### 4. `install.sh` — Update default config block

### 5. `README.md` — Update config example

### 6. `NOTES/terminal-vni.md` — This file

## Testing Checklist

### Linux Mint (X11)

- [ ] Type VNI in shell: `viet1 nam` → `viết nam`
- [ ] Type Telex in shell: `vieets nam` → `vieets nam` (Telex NOT active in terminal)
- [ ] Ctrl+Space toggles VN/EN
- [ ] Ctrl+LeftShift toggles global method (terminal unaffected, tray shows global)
- [ ] Vim insert mode: VNI works, `j`/`x`/`s` pass through as regular keys
- [ ] Gemini-cli: VNI typed text appears correctly
- [ ] sudo passwd: engine auto-disables
- [ ] Switch terminal ↔ GUI: method resets per app
- [ ] Tray icon shows global method, not terminal override

### Ubuntu 24.04+ (Wayland)

- [ ] Same VNI typing tests
- [ ] GNOME Shell D-Bus window detection
- [ ] wl-copy paste-once path for Unicode chars

## Edge Cases

| Case | Behavior |
|------|----------|
| Terminal in bypass_apps | No IME at all (configurable override for power users) |
| User wants Telex in terminals | Set `terminal_input_method = "telex"` in config |
| Multiple terminals open | Each follows the same rule |
| IDE integrated terminal | Window class is "code", not terminal. Needs manual config |
| Password prompt in terminal | Process-tree detection still disables engine regardless of method |
