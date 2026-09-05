#!/bin/bash

# VNIKey User-Local Installation Script

check_dependencies() {
    echo "=== Kiểm tra dependencies ==="
    local warnings=0

    # Check ibus
    if ! command -v ibus &>/dev/null; then
        echo "⚠️  CẢNH BÁO: 'ibus' không tìm thấy."
        echo "   IBus engine sẽ không hoạt động nếu bạn dùng vnikey-ibus."
        echo "   Cài: sudo apt install ibus  (Ubuntu/Debian)"
        echo "        sudo dnf install ibus  (Fedora)"
        warnings=$((warnings + 1))
    else
        echo "✅ ibus: OK"
    fi

    # Check ~/.local/bin trong PATH
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo "⚠️  CẢNH BÁO: '$HOME/.local/bin' không có trong \$PATH."
        echo "   VNIKey sẽ không tìm thấy sau khi install."
        echo "   Thêm vào ~/.bashrc hoặc ~/.zshrc:"
        echo "     export PATH=\"\$HOME/.local/bin:\$PATH\""
        warnings=$((warnings + 1))
    else
        echo "✅ ~/.local/bin trong PATH: OK"
    fi

    # Check notify-send
    if ! command -v notify-send &>/dev/null; then
        echo "⚠️  CẢNH BÁO: 'notify-send' không tìm thấy."
        echo "   Desktop notification khi toggle sẽ không hiện."
        echo "   Cài: sudo apt install libnotify-bin  (Ubuntu/Debian)"
        warnings=$((warnings + 1))
    else
        echo "✅ notify-send: OK"
    fi

    if [ $warnings -gt 0 ]; then
        echo ""
        echo "⚠️  $warnings cảnh báo. Install vẫn tiếp tục."
        echo "   Fix các vấn đề trên trước khi chạy VNIKey."
    fi
    echo ""
}

echo "Starting VNIKey installation..."

check_dependencies

# Create necessary directories
mkdir -p ~/.local/bin
mkdir -p ~/.config/autostart

# List of required files
FILES=("vnikey-wayland" "vnikey-x11" "vnikey-gui" "vnikey.sh" "vnikey-autostart.desktop")
MISSING_FILES=0

# Check if all required files exist in the current directory
for file in "${FILES[@]}"; do
    if [ ! -f "$file" ]; then
        echo "Error: Missing file '$file' in the current directory."
        MISSING_FILES=1
    fi
done

if [ $MISSING_FILES -eq 1 ]; then
    echo "Installation failed: Please run this script from the extracted VNIKey release folder."
    exit 1
fi

# Copy executable files
echo "Copying executables to ~/.local/bin/..."
cp vnikey-wayland ~/.local/bin/
cp vnikey-x11 ~/.local/bin/
cp vnikey-gui ~/.local/bin/
cp vnikey.sh ~/.local/bin/

# Make them executable
chmod +x ~/.local/bin/vnikey-wayland
chmod +x ~/.local/bin/vnikey-x11
chmod +x ~/.local/bin/vnikey-gui
chmod +x ~/.local/bin/vnikey.sh

# Install IBus engine (cho GNOME Wayland)
if [ -f "target/release/vnikey-ibus" ]; then
    echo "Installing vnikey-ibus..."
    sudo cp target/release/vnikey-ibus /usr/local/bin/vnikey-ibus
    sudo chmod +x /usr/local/bin/vnikey-ibus
fi

# Install IBus component XML
if [ -f "vnikey-ibus/component/vnikey-ibus.xml" ]; then
    echo "Installing IBus component descriptor..."
    sudo mkdir -p /usr/share/ibus/component/
    sudo cp vnikey-ibus/component/vnikey-ibus.xml /usr/share/ibus/component/
    echo "IBus component installed. Run: ibus restart"
fi

echo "Installing man pages..."
mkdir -p ~/.local/share/man/man1
cp man/vnikey.1 ~/.local/share/man/man1/

# Copy autostart file
echo "Configuring autostart..."
cp vnikey-autostart.desktop ~/.config/autostart/

echo ""
echo "=== Systemd User Service ==="
read -rp "Cài đặt systemd user service (auto-start + auto-restart khi crash)? [Y/n] " INSTALL_SERVICE
if [[ ! "$INSTALL_SERVICE" =~ ^[Nn]$ ]]; then
    SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
    mkdir -p "$SYSTEMD_USER_DIR"

    if [ -n "$WAYLAND_DISPLAY" ]; then
        cp packaging/vnikey-wayland.service "$SYSTEMD_USER_DIR/"
        systemctl --user daemon-reload
        systemctl --user enable --now vnikey-wayland.service 2>/dev/null && \
            echo "✅ vnikey-wayland.service enabled và started." || \
            echo "⚠️  Không thể start service ngay — thử logout/login lại."
    else
        cp packaging/vnikey-x11.service "$SYSTEMD_USER_DIR/"
        systemctl --user daemon-reload
        systemctl --user enable --now vnikey-x11.service 2>/dev/null && \
            echo "✅ vnikey-x11.service enabled và started." || \
            echo "⚠️  Không thể start service ngay — thử logout/login lại."
    fi

    echo "Để dừng: systemctl --user stop vnikey-wayland (hoặc vnikey-x11)"
    echo "Để xem log: journalctl --user -u vnikey-wayland -f"
else
    echo "Bỏ qua systemd service. Dùng vnikey-autostart.desktop để autostart."
fi

echo "----------------------------------------"
echo "VNIKey has been successfully installed!"
echo ""
echo "Note: The executables are installed in ~/.local/bin."
echo "Make sure ~/.local/bin is in your system PATH."
echo ""
echo "To start VNIKey manually, you can run:"
echo "  ~/.local/bin/vnikey.sh"
echo ""
echo "VNIKey will also start automatically on your next login."
echo "----------------------------------------"
