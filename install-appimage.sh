#!/bin/bash
# Desktop Integration Script for Asthetic Clipboard Manager AppImage
# This script integrates the AppImage with your desktop environment

set -e

APPIMAGE_PATH="$(readlink -f "$1")"
INSTALL_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"

if [ -z "$APPIMAGE_PATH" ] || [ ! -f "$APPIMAGE_PATH" ]; then
    echo "Usage: $0 <path-to-AppImage>"
    echo "Example: $0 ./Asthetic_Clipboard_Manager-x86_64.AppImage"
    exit 1
fi

echo "Installing Asthetic Clipboard Manager..."

# Create directories
mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_DIR"

# Copy AppImage to local bin
echo "  → Copying AppImage to $INSTALL_DIR"
cp "$APPIMAGE_PATH" "$INSTALL_DIR/asthetic-clipboard-manager.AppImage"
chmod +x "$INSTALL_DIR/asthetic-clipboard-manager.AppImage"

# Extract icon
echo "  → Extracting icon..."
cd /tmp
"$INSTALL_DIR/asthetic-clipboard-manager.AppImage" --appimage-extract "*.svg" 2>/dev/null || true
if [ -f "squashfs-root/asthetic-clipboard.svg" ]; then
    cp "squashfs-root/asthetic-clipboard.svg" "$ICON_DIR/"
    rm -rf squashfs-root
fi

# Create desktop entry
echo "  → Creating desktop entry..."
cat > "$DESKTOP_DIR/asthetic-clipboard.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Asthetic Clipboard Manager
GenericName=Clipboard Manager
Comment=A beautiful and persistent clipboard manager
Icon=asthetic-clipboard
Exec=${INSTALL_DIR}/asthetic-clipboard-manager.AppImage
Terminal=false
Categories=Utility;GTK;
Keywords=clipboard;history;manager;paste;
StartupNotify=true
StartupWMClass=asthetic-clipboard
EOF

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    echo "  → Updating desktop database..."
    update-desktop-database "$DESKTOP_DIR"
fi

echo ""
echo "✓ Installation complete!"
echo ""
echo "You can now:"
echo "  1. Find 'Asthetic Clipboard Manager' in your application menu"
echo "  2. Run from terminal: asthetic-clipboard-manager.AppImage"
echo "  3. Start daemon: asthetic-clipboard-manager.AppImage --daemon &"
echo ""
echo "To uninstall, run:"
echo "  rm $INSTALL_DIR/asthetic-clipboard-manager.AppImage"
echo "  rm $DESKTOP_DIR/asthetic-clipboard.desktop"
echo "  rm $ICON_DIR/asthetic-clipboard.svg"
