#!/bin/sh
# Viet+ Flatpak entry point
# Starts the daemon and optionally the system tray interface

HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="$HERE:$PATH"

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/vietc"
mkdir -p "$CONFIG_DIR" "$HOME/.vietc"

# Kill old processes
pkill -x vietc 2>/dev/null || true
pkill -x vietc-xrecord 2>/dev/null || true

# Start daemon in background
"$HERE/vietc" > "$CONFIG_DIR/vietc-daemon.log" 2>&1 &
DAEMON_PID=$!

cleanup() {
    if [ -n "$DAEMON_PID" ]; then
        kill "$DAEMON_PID" 2>/dev/null
        wait "$DAEMON_PID" 2>/dev/null
    fi
}
trap cleanup EXIT INT TERM

# Start tray if available
if [ -f "$HERE/vietc-tray" ]; then
    "$HERE/vietc-tray" "$@"
    exit $?
fi

# No tray: show notification if available
if command -v notify-send >/dev/null 2>&1; then
    notify-send "Viet+" "Input method running in background" -t 3000
fi

echo "[vietc] Running (PID=$DAEMON_PID). Ctrl+C to stop."
wait $DAEMON_PID
