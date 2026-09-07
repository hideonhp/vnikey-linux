#!/bin/bash

# VNIKey Uninstallation Script

echo "Starting VNIKey uninstallation..."

# Function to stop running processes safely
stop_process() {
    local proc_name=$1
    if pgrep -x "$proc_name" > /dev/null; then
        echo "Stopping running process: $proc_name..."
        pkill -x "$proc_name"
    fi
}

# Stop any running VNIKey processes
stop_process "vnikey-wayland"
stop_process "vnikey-x11"
stop_process "vnikey-gui"
stop_process "vnikey-ibus"

if pgrep -f "vnikey.sh" > /dev/null; then
    echo "Stopping running process: vnikey.sh..."
    pkill -f "vnikey.sh"
fi

# Stop systemd services if they exist
systemctl --user stop vnikey-wayland.service 2>/dev/null || true
systemctl --user disable vnikey-wayland.service 2>/dev/null || true
systemctl --user stop vnikey-x11.service 2>/dev/null || true
systemctl --user disable vnikey-x11.service 2>/dev/null || true
systemctl --user stop vnikey-ibus.service 2>/dev/null || true
systemctl --user disable vnikey-ibus.service 2>/dev/null || true
rm -f ~/.config/systemd/user/vnikey-wayland.service
rm -f ~/.config/systemd/user/vnikey-x11.service
rm -f ~/.config/systemd/user/vnikey-ibus.service
systemctl --user daemon-reload

# Remove executable files
echo "Removing executables..."
rm -f ~/.local/bin/vnikey-wayland
rm -f ~/.local/bin/vnikey-x11
rm -f ~/.local/bin/vnikey-gui
rm -f ~/.local/bin/vnikey.sh

if [ -f "/usr/local/bin/vnikey-ibus" ] || [ -f "/usr/share/ibus/component/vnikey-ibus.xml" ]; then
    echo "Removing system-wide IBus files (requires sudo)..."
    sudo rm -f /usr/local/bin/vnikey-ibus
    sudo rm -f /usr/local/bin/vnikey-gui
    sudo rm -f /usr/share/ibus/component/vnikey-ibus.xml
fi

# Remove autostart file
echo "Removing autostart entry..."
rm -f ~/.config/autostart/vnikey-autostart.desktop

echo "----------------------------------------"
echo "VNIKey has been successfully uninstalled!"
echo "----------------------------------------"
