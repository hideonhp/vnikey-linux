#!/bin/sh

# Kiểm tra xem môi trường hiện tại là Wayland hay X11
if [ "$XDG_SESSION_TYPE" = "wayland" ] || [ -n "$WAYLAND_DISPLAY" ]; then
    # GNOME Wayland không hỗ trợ zwp_input_method_v2 — dùng IBus engine
    # Detect GNOME bằng XDG_CURRENT_DESKTOP hoặc GNOME_SETUP_DISPLAY
    if echo "$XDG_CURRENT_DESKTOP" | grep -qi "gnome"; then
        echo "ℹ️  GNOME Wayland detected (IBus mode)."
        echo "VNIKey for GNOME is managed automatically by the IBus daemon."
        echo "You do not need to run this script or use autostart."
        echo "Simply add 'Vietnamese (VNIKey)' in Settings -> Keyboard -> Input Sources."
        exit 0
    else
        # Non-GNOME Wayland (Sway, KWin, etc.) — dùng native Wayland protocol
        echo "Starting VNIKey for Wayland (native mode)..."
        exec vnikey-wayland
    fi
else
    echo "Starting VNIKey for X11..."
    exec vnikey-x11
fi
