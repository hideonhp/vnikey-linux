#!/bin/sh

# Kiểm tra xem môi trường hiện tại là Wayland hay X11
if [ "$XDG_SESSION_TYPE" = "wayland" ] || [ -n "$WAYLAND_DISPLAY" ]; then
    echo "Starting VNIKey for Wayland..."
    exec vnikey-wayland
else
    echo "Starting VNIKey for X11..."
    exec vnikey-x11
fi
