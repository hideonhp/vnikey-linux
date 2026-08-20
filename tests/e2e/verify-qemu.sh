#!/bin/bash
# verify-qemu.sh <de> <session>
# Chạy TRONG VM (qua SSH từ GitHub Actions runner).
set -euo pipefail

DE="${1:-gnome}"
SESSION="${2:-x11}"
LOG="/tmp/vnikey-e2e-qemu-${DE}-${SESSION}.log"
BINARY_X11="./target/release/vnikey-x11"
BINARY_WAYLAND="./target/release/vnikey-wayland"

# Ensure cargo env
source "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"

log() { echo "$@" | tee -a "$LOG"; }
pass() { log "  ✅ PASS: $*"; }
fail() { log "  ❌ FAIL: $*"; FAILED=1; }

FAILED=0
XVFB_PID=""
VNIKEY_PID=""
COMPOSITOR_PID=""
GTK_PID=""

cleanup() {
  [ -n "$VNIKEY_PID"    ] && kill "$VNIKEY_PID"    2>/dev/null || true
  [ -n "$GTK_PID"       ] && kill "$GTK_PID"       2>/dev/null || true
  [ -n "$COMPOSITOR_PID" ] && kill "$COMPOSITOR_PID" 2>/dev/null || true
  [ -n "$XVFB_PID"      ] && kill "$XVFB_PID"      2>/dev/null || true
}
trap cleanup EXIT

log "=== QEMU VM E2E Test [DE=$DE SESSION=$SESSION] ==="
log "Running on: $(uname -a)"

if [ "$SESSION" = "x11" ]; then
  # ─── X11 Session ───────────────────────────────────────────

  # Start Xvfb
  Xvfb :99 -screen 0 1280x720x24 -ac &
  XVFB_PID=$!
  export DISPLAY=:99
  sleep 1
  log "Xvfb started"

  # dbus
  eval "$(dbus-launch --sh-syntax)"
  export DBUS_SESSION_BUS_ADDRESS
  sleep 0.5

  # GTK test window
  cat > /tmp/gtk_test.py << 'PYEOF'
import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk

window = Gtk.Window(title="VniKeyE2ETest")
window.set_default_size(400, 100)
entry = Gtk.Entry()
window.add(entry)
window.show_all()

def on_key_press(widget, event):
    if event.keyval in (Gdk.KEY_Return, Gdk.KEY_KP_Enter):
        text = entry.get_text()
        with open('/tmp/vnikey-gtk-output.txt', 'w', encoding='utf-8') as f:
            f.write(text)
        Gtk.main_quit()
    return False

window.connect('key-press-event', on_key_press)
window.connect('destroy', Gtk.main_quit)
Gtk.main()
PYEOF

  python3 /tmp/gtk_test.py &
  GTK_PID=$!
  sleep 1

  WIN_ID=$(xdotool search --sync --name "VniKeyE2ETest" 2>/dev/null | head -1 || echo "")
  if [ -z "$WIN_ID" ]; then
    log "ERROR: GTK window not found"
    exit 1
  fi
  xdotool windowfocus --sync "$WIN_ID"
  sleep 0.3

  # Start vnikey-x11
  "$BINARY_X11" >> "$LOG" 2>&1 &
  VNIKEY_PID=$!
  sleep 1

  if ! kill -0 "$VNIKEY_PID" 2>/dev/null; then
    log "ERROR: vnikey-x11 crashed"
    exit 1
  fi
  log "vnikey-x11 started"

  # Run typing tests
  run_test() {
    local name="$1" keystrokes="$2" expected="$3"

    rm -f /tmp/vnikey-gtk-output.txt

    # Clear entry
    xdotool key ctrl+a; sleep 0.1
    xdotool key Delete; sleep 0.1

    # Type test input
    xdotool type --delay 60 --clearmodifiers "$keystrokes"
    sleep 0.3

    # Enter → vnikey commit → GTK saves
    xdotool key Return
    sleep 0.5

    if [ ! -f /tmp/vnikey-gtk-output.txt ]; then
      fail "$name: output file not created"
      return
    fi

    actual=$(cat /tmp/vnikey-gtk-output.txt)
    if [ "$actual" = "$expected" ]; then
      pass "$name: '$keystrokes' → '$actual'"
    else
      fail "$name: got '$actual', expected '$expected'"
    fi

    # Restart GTK
    kill "$GTK_PID" 2>/dev/null || true; sleep 0.2
    python3 /tmp/gtk_test.py &
    GTK_PID=$!; sleep 0.8
    WIN_ID=$(xdotool search --sync --name "VniKeyE2ETest" 2>/dev/null | head -1 || echo "")
    [ -n "$WIN_ID" ] && xdotool windowfocus --sync "$WIN_ID"
    sleep 0.2
  }

  log "--- Typing Tests (X11 in Fedora VM) ---"
  run_test "basic-viet"     "viet"    "việt"
  run_test "word-chao"      "chao"    "chào"
  run_test "capital-Tieng"  "Tieng"   "Tiếng"
  run_test "capital-Viet"   "Viet"    "Việt"
  run_test "word-dduong"    "dduong"  "đường"
  run_test "word-thooi"     "thooi"   "thôi"
  run_test "word-nguoi"     "nguoif"  "người"

else
  # ─── Wayland Session ────────────────────────────────────────

  eval "$(dbus-launch --sh-syntax)"
  export DBUS_SESSION_BUS_ADDRESS

  if [ "$DE" = "gnome" ]; then
    log "Starting Mutter headless..."
    mutter --wayland --headless --virtual-monitor 1920x1080 >> "$LOG" 2>&1 &
    COMPOSITOR_PID=$!
  else
    log "Starting kwin_wayland virtual..."
    kwin_wayland --virtual >> "$LOG" 2>&1 &
    COMPOSITOR_PID=$!
  fi
  sleep 3

  if ! kill -0 "$COMPOSITOR_PID" 2>/dev/null; then
    log "ERROR: $DE compositor crashed"
    exit 1
  fi
  log "$DE compositor started"

  # Detect Wayland socket
  for sock in wayland-0 wayland-1 wayland-2; do
    if [ -S "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/$sock" ]; then
      export WAYLAND_DISPLAY="$sock"
      break
    fi
  done

  if [ -z "${WAYLAND_DISPLAY:-}" ]; then
    log "ERROR: Wayland socket not found"
    exit 1
  fi
  log "Wayland socket: $WAYLAND_DISPLAY"

  # Start vnikey-wayland
  "$BINARY_WAYLAND" >> "$LOG" 2>&1 &
  VNIKEY_PID=$!
  sleep 3

  if kill -0 "$VNIKEY_PID" 2>/dev/null; then
    pass "vnikey-wayland connected to $DE compositor"
  else
    fail "vnikey-wayland crashed (DE=$DE)"
  fi

  # Bonus: test với ydotool nếu có (future enhancement)
  if command -v ydotool &>/dev/null; then
    log "ydotool available — advanced typing test: TBD"
  fi
fi

log ""
if [ "$FAILED" -eq 0 ]; then
  log "✅ All QEMU VM tests passed! [DE=$DE SESSION=$SESSION]"
else
  log "❌ Some tests failed. Log: $LOG"
  exit 1
fi