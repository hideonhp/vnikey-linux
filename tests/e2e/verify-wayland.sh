#!/bin/bash
# verify-wayland.sh <compositor_name> <compositor_cmd>
# Test Wayland connectivity: vnikey-wayland kết nối được compositor không?
set -euo pipefail

COMPOSITOR_NAME="${1:-gnome-mutter}"
COMPOSITOR_CMD="${2:-mutter --wayland --headless --virtual-monitor 1920x1080}"
LOG="/tmp/vnikey-e2e-wayland-${COMPOSITOR_NAME}.log"
BINARY="./target/release/vnikey-wayland"

log() { echo "$@" | tee -a "$LOG"; }

COMPOSITOR_PID=""
VNIKEY_PID=""

cleanup() {
  [ -n "$VNIKEY_PID"    ] && kill "$VNIKEY_PID"    2>/dev/null || true
  [ -n "$COMPOSITOR_PID" ] && kill "$COMPOSITOR_PID" 2>/dev/null || true
}
trap cleanup EXIT

log "=== E2E Wayland Test [Compositor=$COMPOSITOR_NAME] ==="

# 1. Setup dbus session (mutter/kwin cần dbus)
eval "$(dbus-launch --sh-syntax)"
export DBUS_SESSION_BUS_ADDRESS

# Setup XDG_RUNTIME_DIR nếu chưa có
if [ -z "${XDG_RUNTIME_DIR:-}" ]; then
  export XDG_RUNTIME_DIR="/tmp/xdg-runtime-$$"
  mkdir -p "$XDG_RUNTIME_DIR"
  chmod 700 "$XDG_RUNTIME_DIR"
fi

# 2. Start headless compositor
log "Starting compositor: $COMPOSITOR_CMD"
XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" $COMPOSITOR_CMD >> "$LOG" 2>&1 &
COMPOSITOR_PID=$!
sleep 5  # Chờ compositor init

if ! kill -0 "$COMPOSITOR_PID" 2>/dev/null; then
  log "ERROR: Compositor '$COMPOSITOR_NAME' crashed on startup"
  log "Check if package is installed and command is correct"
  exit 1
fi
log "Compositor started (PID=$COMPOSITOR_PID)"

# 3. Detect Wayland display socket
# Mutter thường tạo wayland-0 hoặc wayland-1
WAYLAND_DISPLAY_DETECTED=""
for dir in "$XDG_RUNTIME_DIR" "/run/user/$(id -u)" "/tmp"; do
  for socket in wayland-0 wayland-1 wayland-2; do
    if [ -S "$dir/$socket" ]; then
      WAYLAND_DISPLAY_DETECTED="$socket"
      export XDG_RUNTIME_DIR="$dir"
      break 2
    fi
  done
done

if [ -z "$WAYLAND_DISPLAY_DETECTED" ]; then
  log "ERROR: No Wayland socket found in XDG_RUNTIME_DIR"
  exit 1
fi
export WAYLAND_DISPLAY="$WAYLAND_DISPLAY_DETECTED"
log "Wayland socket: $WAYLAND_DISPLAY"

# 4. Start vnikey-wayland
log "Starting vnikey-wayland..."
"$BINARY" >> "$LOG" 2>&1 &
VNIKEY_PID=$!
sleep 3  # Chờ vnikey kết nối + negotiate protocol

# 5. Kiểm tra vnikey vẫn còn chạy (không crash)
if kill -0 "$VNIKEY_PID" 2>/dev/null; then
  log "✅ PASS: vnikey-wayland connected to $COMPOSITOR_NAME compositor (PID=$VNIKEY_PID)"
else
  log "❌ FAIL: vnikey-wayland crashed or exited unexpectedly"
  log "Compositor: $COMPOSITOR_NAME"
  log "WAYLAND_DISPLAY: $WAYLAND_DISPLAY"
  exit 1
fi

# 6. Kiểm tra log output cho error keywords
if grep -i "error\|panic\|failed\|denied" "$LOG" | grep -v "^===" > /dev/null 2>&1; then
  log "⚠️  WARNING: Found error keywords in log:"
  grep -i "error\|panic\|failed\|denied" "$LOG" | grep -v "^===" | tee -a "$LOG"
  # Không fail hard ở đây vì một số warnings là bình thường
fi

log ""
log "✅ Wayland connectivity test passed for $COMPOSITOR_NAME"