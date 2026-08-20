#!/bin/bash
# create-cloud-init.sh <de> <session> <ssh_pub_key>
# Tạo cloud-init seed.iso cho QEMU VM.
set -euo pipefail

DE="$1"          # gnome | kde
SESSION="$2"     # wayland | x11
SSH_PUB_KEY="$3" # "ssh-ed25519 AAAA..."

echo "Creating cloud-init for: DE=$DE SESSION=$SESSION"

# Packages chung (build deps cho Fedora)
BASE_PACKAGES=(
  git rsync gcc pkgconf
  glib2-devel wayland-devel
  libxkbcommon-devel libxkbcommon-x11-devel
  xcb-util-wm-devel dbus-devel libXtst-devel
  dbus python3
)

# Packages theo DE
GNOME_PACKAGES=(gnome-shell mutter gnome-terminal at-spi2-core)
KDE_PACKAGES=(plasma-desktop kwin konsole)

# Packages theo session
X11_PACKAGES=(xorg-x11-server-Xvfb xdotool xclip xterm python3-gobject gtk3)
WAYLAND_PACKAGES=(ydotool)

# Tổng hợp packages
if [ "$DE" = "gnome" ]; then
  DE_PKGS=("${GNOME_PACKAGES[@]}")
else
  DE_PKGS=("${KDE_PACKAGES[@]}")
fi

if [ "$SESSION" = "x11" ]; then
  SESSION_PKGS=("${X11_PACKAGES[@]}")
else
  SESSION_PKGS=("${WAYLAND_PACKAGES[@]}")
fi

ALL_PACKAGES=("${BASE_PACKAGES[@]}" "${DE_PKGS[@]}" "${SESSION_PKGS[@]}")

# Build YAML list cho cloud-init
PKG_YAML=""
for pkg in "${ALL_PACKAGES[@]}"; do
  PKG_YAML="${PKG_YAML}  - ${pkg}\n"
done

# Tạo user-data
cat > user-data << YAML
#cloud-config
users:
  - name: tester
    groups: [wheel, video, audio, input, kvm]
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    ssh_authorized_keys:
      - ${SSH_PUB_KEY}

packages:
$(printf '%b' "$PKG_YAML")

runcmd:
  # Install Rust for tester user
  - su -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path" tester
  - echo 'export PATH=\$HOME/.cargo/bin:\$PATH' >> /home/tester/.bashrc
  - chown -R tester:tester /home/tester/.cargo /home/tester/.rustup 2>/dev/null || true

final_message: "VniKey E2E VM ready!"
YAML

# Tạo meta-data
cat > meta-data << YAML
instance-id: vnikey-e2e-$(date +%s)
local-hostname: vnikey-test
YAML

echo "Creating seed.iso..."
genisoimage \
  -output seed.iso \
  -volid cidata \
  -joliet \
  -rock \
  user-data \
  meta-data

echo "seed.iso created successfully"
ls -lh seed.iso