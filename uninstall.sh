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
# Need to use -f for bash scripts sometimes, or just pkill by string
if pgrep -f "vnikey.sh" > /dev/null; then
    echo "Stopping running process: vnikey.sh..."
    pkill -f "vnikey.sh"
fi

# Remove executable files
echo "Removing executables from ~/.local/bin/..."
rm -f ~/.local/bin/vnikey-wayland
rm -f ~/.local/bin/vnikey-x11
rm -f ~/.local/bin/vnikey-gui
rm -f ~/.local/bin/vnikey.sh

# Remove autostart file
echo "Removing autostart entry..."
rm -f ~/.config/autostart/vnikey-autostart.desktop

echo "----------------------------------------"
echo "VNIKey has been successfully uninstalled!"
echo "----------------------------------------"
