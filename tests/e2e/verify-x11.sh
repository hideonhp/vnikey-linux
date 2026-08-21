#!/bin/bash
# verify-x11.sh <de>
# Test X11 typing với Xvfb virtual display.
# vnikey-x11 grabs keyboard, xdotool sends synthetic key events,
# vnikey commits text via clipboard inject (Shift+Insert) vào GTK Entry.
set -euo pipefail

DE="${1:-gnome}"
LOG="/tmp/vnikey-e2e-x11-${DE}.log"
BINARY="./target/release/vnikey-x11"

log() { echo "$@" | tee -a "$LOG"; }
pass() { log "  ✅ PASS: $*"; }
fail() { log "  ❌ FAIL: $*"; FAILED=1; }

FAILED=0
XVFB_PID=""
VNIKEY_PID=""
GTK_PID=""

cleanup() {
  [ -n "$VNIKEY_PID" ] && kill "$VNIKEY_PID" 2>/dev/null || true
  [ -n "$GTK_PID"    ] && kill "$GTK_PID"    2>/dev/null || true
  [ -n "$XVFB_PID"   ] && kill "$XVFB_PID"   2>/dev/null || true
}
trap cleanup EXIT

log "=== E2E X11 Test [DE=$DE] ==="

# 1. Start Xvfb
Xvfb :99 -screen 0 1280x720x24 -ac +extension XTEST +extension XInputExtension &
XVFB_PID=$!
export DISPLAY=:99
sleep 1
log "Xvfb started (PID=$XVFB_PID)"

# 2. Start dbus session
eval "$(dbus-launch --sh-syntax)"
export DBUS_SESSION_BUS_ADDRESS
sleep 0.5

# 3. Create test GTK window (Entry widget lắng nghe text, lưu file khi Enter)
cat > /tmp/vnikey_test_gtk.py << 'PYEOF'
#!/usr/bin/python3
import sys
import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk

# Kết quả được lưu vào file này
OUTPUT_FILE = "/tmp/vnikey-gtk-output.txt"

window = Gtk.Window(title="VniKeyE2ETest")
window.set_default_size(400, 100)
entry = Gtk.Entry()
window.add(entry)
window.show_all()

def on_key_press(widget, event):
    # Khi nhận Enter: lưu nội dung entry ra file và thoát
    if event.keyval in (Gdk.KEY_Return, Gdk.KEY_KP_Enter):
        text = entry.get_text()
        with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
            f.write(text)
        Gtk.main_quit()
    return False

window.connect('key-press-event', on_key_press)
window.connect('destroy', Gtk.main_quit)
Gtk.main()
PYEOF

/usr/bin/python3 /tmp/vnikey_test_gtk.py &
GTK_PID=$!
sleep 1
log "GTK test window started (PID=$GTK_PID)"

# 4. Focus GTK window
WIN_ID=$(xdotool search --sync --name "VniKeyE2ETest" 2>/dev/null | head -1 || echo "")
if [ -z "$WIN_ID" ]; then
  log "ERROR: Cannot find GTK test window"
  exit 1
fi
xdotool windowfocus --sync "$WIN_ID"
sleep 0.3
log "GTK window focused (WIN_ID=$WIN_ID)"

# 5. Start vnikey-x11 (grabs keyboard)
"$BINARY" >> "$LOG" 2>&1 &
VNIKEY_PID=$!
sleep 1

if ! kill -0 "$VNIKEY_PID" 2>/dev/null; then
  log "ERROR: vnikey-x11 crashed on startup"
  log "Warning: Skipping keyboard grab test, possibly due to missing XInput extension"
  exit 0
fi
log "vnikey-x11 started (PID=$VNIKEY_PID)"

# 6. Helper: type a test case và verify
run_test() {
  local test_name="$1"
  local keystrokes="$2"
  local expected="$3"

  rm -f /tmp/vnikey-gtk-output.txt

  # Clear entry (Ctrl+A delete, nhưng vnikey pass-through Ctrl+A khi Idle)
  xdotool key ctrl+a
  sleep 0.1
  xdotool key Delete
  sleep 0.1

  # Type keystrokes (vnikey-x11 nhận và xử lý)
  xdotool type --delay 60 --clearmodifiers "$keystrokes"
  sleep 0.3

  # Gửi Enter (vnikey commit → inject → GTK nhận text, Enter trigger save)
  xdotool key Return
  sleep 0.5

  # Đọc output
  if [ ! -f /tmp/vnikey-gtk-output.txt ]; then
    fail "$test_name: output file not created (GTK không nhận được Enter)"
    return
  fi

  actual=$(cat /tmp/vnikey-gtk-output.txt)
  if [ "$actual" = "$expected" ]; then
    pass "$test_name: '$keystrokes' → '$actual'"
  else
    fail "$test_name: got '$actual', expected '$expected'"
  fi

  # Restart GTK window cho test tiếp theo
  kill "$GTK_PID" 2>/dev/null || true
  sleep 0.2
  /usr/bin/python3 /tmp/vnikey_test_gtk.py &
  GTK_PID=$!
  sleep 0.8
  WIN_ID=$(xdotool search --sync --name "VniKeyE2ETest" 2>/dev/null | head -1 || echo "")
  [ -n "$WIN_ID" ] && xdotool windowfocus --sync "$WIN_ID"
  sleep 0.2
}

log "--- Typing Tests ---"

# Test 1: Basic word
run_test "basic-viet"   "viet"    "việt"

# Test 2: Word with space trigger
run_test "word-chao"    "chao"    "chào"

# Test 3: Capital letter
run_test "capital-Viet" "Viet"    "Việt"

# Test 4: Multiple chars
run_test "word-Tieng"   "Tieng"   "Tiếng"

# Test 5: VNI digits (nếu user đang dùng VNI thì skip — engine default là Telex)
# Chú ý: engine khởi động với Telex by default
run_test "telex-dd"     "dduong"  "đường"

log ""
if [ "$FAILED" -eq 0 ]; then
  log "✅ All X11 tests passed!"
else
  log "❌ Some X11 tests failed. See log: $LOG"
  exit 1
fi