#!/bin/bash
# verify-container.sh <distro>
# Smoke test: binary tồn tại, ELF hợp lệ, không thiếu shared libs.
set -euo pipefail

DISTRO="${1:-unknown}"
LOG="/tmp/vnikey-e2e-container-${DISTRO//[:\/]/-}.log"

log() { echo "$@" | tee -a "$LOG"; }
pass() { log "  ✅ PASS: $*"; }
fail() { log "  ❌ FAIL: $*"; FAILED=1; }

FAILED=0

log "=== E2E Container Smoke Test [Distro=$DISTRO] ==="

check_binary() {
  local name="$1"
  local path="$2"

  log "--- Checking: $name ($path) ---"

  # Check file exists
  if [ ! -f "$path" ]; then
    fail "$name: binary not found at $path"
    return
  fi
  pass "$name: binary exists"

  # Check ELF format
  ELF_INFO=$(file "$path")
  if echo "$ELF_INFO" | grep -q "ELF 64-bit"; then
    pass "$name: ELF 64-bit format OK"
  else
    fail "$name: unexpected format: $ELF_INFO"
  fi

  # Check no missing shared libs
  MISSING=$(ldd "$path" 2>&1 | grep "not found" || true)
  if [ -z "$MISSING" ]; then
    pass "$name: all shared libraries found"
  else
    fail "$name: missing shared libraries:\n$MISSING"
  fi
}

# Check tất cả binaries
check_binary "vnikey-x11"     "./target/release/vnikey-x11"
check_binary "vnikey-wayland" "./target/release/vnikey-wayland"
check_binary "vnikey-tray"    "./target/release/vnikey-tray"

# Check install script không bị syntax error
log "--- Checking install.sh syntax ---"
if bash -n install.sh 2>&1; then
  pass "install.sh: syntax OK"
else
  fail "install.sh: syntax error"
fi

# Không cố chạy install.sh trong container, chỉ test binary
log "--- Skipping install.sh in container (no systemd) ---"
pass "install.sh: skipped in container environment"

log ""
if [ "$FAILED" -eq 0 ]; then
  log "✅ All container smoke tests passed!"
else
  log "❌ Some container tests failed. See: $LOG"
  exit 1
fi