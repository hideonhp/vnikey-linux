#!/bin/bash

# VNIKey Installation Script
#
# Usage:
#   ./install.sh                  — Auto-detect environment and install appropriate profile
#   ./install.sh --ibus-only      — Force install only vnikey-ibus (for GNOME Wayland / IBus users)
#   ./install.sh --full           — Install all frontends (Wayland, X11, IBus, GUI)

set -e

IBUS_ONLY=0
FULL_INSTALL=0

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        --ibus-only)
            IBUS_ONLY=1
            ;;
        --full)
            FULL_INSTALL=1
            ;;
        --help|-h)
            echo "VNIKey Install Script"
            echo ""
            echo "Usage:"
            echo "  ./install.sh               Auto-detect environment"
            echo "  ./install.sh --ibus-only   Install only vnikey-ibus (GNOME Wayland)"
            echo "  ./install.sh --full        Install all frontends (Wayland + X11 + IBus)"
            exit 0
            ;;
    esac
done

# --- Detect environment ---
IS_GNOME_WAYLAND=0
if echo "$XDG_CURRENT_DESKTOP" | grep -qi "gnome" && [ -n "$WAYLAND_DISPLAY" ]; then
    IS_GNOME_WAYLAND=1
fi

# Auto-set IBUS_ONLY when on GNOME Wayland and --full not specified
if [ $IS_GNOME_WAYLAND -eq 1 ] && [ $FULL_INSTALL -eq 0 ]; then
    IBUS_ONLY=1
fi

echo "=== VNIKey Installer ==="
if [ $IBUS_ONLY -eq 1 ]; then
    echo "Profile: IBus-only (GNOME Wayland / IBus)"
elif [ $FULL_INSTALL -eq 1 ]; then
    echo "Profile: Full install (Wayland + X11 + IBus)"
else
    echo "Profile: Auto-detect"
fi
echo ""

# --- Check dependencies ---
check_dependencies() {
    echo "=== Kiểm tra dependencies ==="
    local warnings=0

    # IBus is required for IBus-only profile
    if [ $IBUS_ONLY -eq 1 ]; then
        if ! command -v ibus &>/dev/null; then
            echo "❌ LỖI: 'ibus' không tìm thấy — bắt buộc cho profile IBus-only."
            echo "   Cài: sudo apt install ibus  (Ubuntu/Debian)"
            echo "        sudo dnf install ibus  (Fedora)"
            exit 1
        else
            echo "✅ ibus: OK"
        fi
    else
        if ! command -v ibus &>/dev/null; then
            echo "⚠️  CẢNH BÁO: 'ibus' không tìm thấy."
            echo "   IBus engine sẽ không hoạt động nếu bạn dùng vnikey-ibus."
            echo "   Cài: sudo apt install ibus  (Ubuntu/Debian)"
            echo "        sudo dnf install ibus  (Fedora)"
            warnings=$((warnings + 1))
        else
            echo "✅ ibus: OK"
        fi
    fi

    # Check ~/.local/bin in PATH (only needed for non-ibus-only)
    if [ $IBUS_ONLY -eq 0 ]; then
        if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
            echo "⚠️  CẢNH BÁO: '$HOME/.local/bin' không có trong \$PATH."
            echo "   VNIKey sẽ không tìm thấy sau khi install."
            echo "   Thêm vào ~/.bashrc hoặc ~/.zshrc:"
            echo "     export PATH=\"\$HOME/.local/bin:\$PATH\""
            warnings=$((warnings + 1))
        else
            echo "✅ ~/.local/bin trong PATH: OK"
        fi
    fi

    # notify-send (optional, only for X11/Wayland frontends)
    if [ $IBUS_ONLY -eq 0 ]; then
        if ! command -v notify-send &>/dev/null; then
            echo "⚠️  CẢNH BÁO: 'notify-send' không tìm thấy."
            echo "   Desktop notification khi toggle sẽ không hiện."
            echo "   Cài: sudo apt install libnotify-bin  (Ubuntu/Debian)"
            echo "        sudo dnf install libnotify       (Fedora)"
            warnings=$((warnings + 1))
        else
            echo "✅ notify-send: OK"
        fi
    fi

    if [ $warnings -gt 0 ]; then
        echo ""
        echo "⚠️  $warnings cảnh báo. Install vẫn tiếp tục."
    fi
    echo ""
}

check_dependencies

# --- IBus-Only Profile ---
install_ibus_only() {
    echo "=== Cài đặt vnikey-ibus (IBus-only profile) ==="

    IBUS_INSTALLED=0

    # Find vnikey-ibus binary
    if [ -f "vnikey-ibus" ]; then
        IBUS_BIN="vnikey-ibus"
    elif [ -f "target/release/vnikey-ibus" ]; then
        IBUS_BIN="target/release/vnikey-ibus"
    else
        echo "❌ LỖI: Không tìm thấy binary 'vnikey-ibus'."
        echo "   Build từ source: cargo build --release -p vnikey-ibus"
        exit 1
    fi

    echo "Installing $IBUS_BIN → /usr/local/bin/vnikey-ibus"
    sudo install -Dm755 "$IBUS_BIN" /usr/local/bin/vnikey-ibus
    echo "✅ vnikey-ibus installed."

    IBUS_INSTALLED=1

    # Install IBus component XML
    install_ibus_xml

    # Man page
    install_man_page

    echo ""
    echo "=== Hoàn tất (IBus-only) ==="
    echo "IBus đã được cấu hình để dùng VNIKey."
    echo ""
    echo "Các bước tiếp theo:"
    echo "  1. Vào Settings → Keyboard → Input Sources"
    echo "  2. Thêm 'Vietnamese (VNIKey)'"
    echo "  3. IBus sẽ tự động khởi động vnikey-ibus khi bạn switch sang VNIKey."
    echo ""
    echo "💡 Không cần systemd service hay autostart — IBus tự quản lý lifecycle."
    echo "💡 Để cấu hình: dùng ibus-setup hoặc GNOME Settings."
}

# --- Full Profile ---
install_full() {
    echo "=== Cài đặt đầy đủ (Full profile) ==="

    mkdir -p ~/.local/bin ~/.config/autostart

    # Check required files
    FILES=("vnikey-wayland" "vnikey-x11" "vnikey-gui" "vnikey.sh" "vnikey-autostart.desktop")
    MISSING=0
    for file in "${FILES[@]}"; do
        if [ ! -f "$file" ]; then
            echo "❌ Thiếu file '$file'."
            MISSING=1
        fi
    done
    if [ $MISSING -eq 1 ]; then
        echo "Hãy chạy script này từ thư mục release của VNIKey."
        exit 1
    fi

    echo "Copying executables to ~/.local/bin/..."
    install -Dm755 vnikey-wayland ~/.local/bin/vnikey-wayland
    install -Dm755 vnikey-x11     ~/.local/bin/vnikey-x11
    install -Dm755 vnikey-gui     ~/.local/bin/vnikey-gui
    install -Dm755 vnikey.sh      ~/.local/bin/vnikey.sh
    echo "✅ Executables installed."

    # IBus engine (optional in full profile)
    IBUS_INSTALLED=0
    if [ -f "vnikey-ibus" ]; then
        echo "Installing vnikey-ibus (IBus engine for GNOME Wayland)..."
        sudo install -Dm755 vnikey-ibus /usr/local/bin/vnikey-ibus
        IBUS_INSTALLED=1
        echo "✅ vnikey-ibus installed."
    elif [ -f "target/release/vnikey-ibus" ]; then
        sudo install -Dm755 target/release/vnikey-ibus /usr/local/bin/vnikey-ibus
        IBUS_INSTALLED=1
        echo "✅ vnikey-ibus installed from build output."
    else
        echo "ℹ️  vnikey-ibus không tìm thấy — bỏ qua IBus engine install."
    fi

    install_ibus_xml
    install_man_page

    # Autostart
    cp vnikey-autostart.desktop ~/.config/autostart/
    echo "✅ Autostart configured."

    # Systemd user service
    echo ""
    read -rp "Cài đặt systemd user service (auto-start + auto-restart)? [Y/n] " INSTALL_SERVICE
    if [[ ! "$INSTALL_SERVICE" =~ ^[Nn]$ ]]; then
        install_systemd_service
    fi

    echo ""
    echo "=== Hoàn tất (Full install) ==="
    echo "Để chạy VNIKey thủ công: ~/.local/bin/vnikey.sh"
    echo "VNIKey sẽ tự khởi động ở lần đăng nhập tiếp theo."
}

# --- Helpers ---
install_ibus_xml() {
    if [ -f "vnikey-ibus.xml" ]; then
        echo "Installing IBus component descriptor..."
        sudo mkdir -p /usr/share/ibus/component/
        sudo cp vnikey-ibus.xml /usr/share/ibus/component/vnikey-ibus.xml
        echo "✅ IBus component XML installed."
    elif [ -f "vnikey-ibus/component/vnikey-ibus.xml" ]; then
        echo "Installing IBus component descriptor (from source tree)..."
        sudo mkdir -p /usr/share/ibus/component/
        sudo cp vnikey-ibus/component/vnikey-ibus.xml /usr/share/ibus/component/
        echo "✅ IBus component XML installed."
    fi

    if command -v ibus &>/dev/null; then
        ibus restart 2>/dev/null || true
        echo "✅ IBus restarted."
    fi
}

install_man_page() {
    mkdir -p ~/.local/share/man/man1
    if [ -f "man/vnikey.1" ]; then
        cp man/vnikey.1 ~/.local/share/man/man1/
        echo "✅ Man page installed."
    elif [ -f "vnikey.1" ]; then
        cp vnikey.1 ~/.local/share/man/man1/
        echo "✅ Man page installed."
    fi
}

install_systemd_service() {
    SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
    mkdir -p "$SYSTEMD_USER_DIR"

    if [ -n "$WAYLAND_DISPLAY" ]; then
        SERVICE_FILE="packaging/vnikey-wayland.service"
        SERVICE_NAME="vnikey-wayland"
    else
        SERVICE_FILE="packaging/vnikey-x11.service"
        SERVICE_NAME="vnikey-x11"
    fi

    if [ -f "$SERVICE_FILE" ]; then
        cp "$SERVICE_FILE" "$SYSTEMD_USER_DIR/"
        systemctl --user daemon-reload
        systemctl --user enable --now "${SERVICE_NAME}.service" 2>/dev/null && \
            echo "✅ ${SERVICE_NAME}.service enabled và started." || \
            echo "⚠️  Không thể start service ngay — thử logout/login lại."
    else
        echo "⚠️  Không tìm thấy $SERVICE_FILE — bỏ qua systemd service."
    fi
    echo "Để dừng: systemctl --user stop $SERVICE_NAME"
    echo "Để xem log: journalctl --user -u $SERVICE_NAME -f"
}

# --- Main ---
if [ $IBUS_ONLY -eq 1 ]; then
    install_ibus_only
else
    install_full
fi
