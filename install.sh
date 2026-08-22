#!/bin/bash

# VNIKey User-Local Installation Script

echo "Starting VNIKey installation..."

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

# Copy autostart file
echo "Configuring autostart..."
cp vnikey-autostart.desktop ~/.config/autostart/

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
