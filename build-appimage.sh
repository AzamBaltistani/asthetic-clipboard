#!/bin/bash
set -e

echo "=================================="
echo "Building Asthetic Clipboard AppImage"
echo "=================================="

# Constants
APPDIR="AppDir"
APPNAME="Asthetic_Clipboard_Manager"
ARCH="x86_64"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo_step() {
    echo -e "${BLUE}==>${NC} $1"
}

echo_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Step 1: Build release binaries
echo_step "Building release binaries..."
cargo build --release
echo_success "Binaries built successfully"

# Step 2: Create AppDir structure
echo_step "Creating AppDir structure..."
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"

# Step 3: Copy binaries
echo_step "Copying binaries..."
cp target/release/asthetic-clipboard "${APPDIR}/usr/bin/"
cp target/release/daemon "${APPDIR}/usr/bin/"
cp target/release/tui "${APPDIR}/usr/bin/"
echo_success "Binaries copied"

# Step 4: Copy desktop file and icon
echo_step "Copying desktop entry and icon..."
cp asthetic-clipboard.desktop "${APPDIR}/usr/share/applications/"
cp assets/icon.svg "${APPDIR}/usr/share/icons/hicolor/scalable/apps/asthetic-clipboard.svg"

# Create PNG icon from SVG for better compatibility
if command -v inkscape &> /dev/null; then
    echo_step "Converting icon to PNG..."
    inkscape assets/icon.svg -w 256 -h 256 -o "${APPDIR}/usr/share/icons/hicolor/256x256/apps/asthetic-clipboard.png" &> /dev/null
    echo_success "Icon converted to PNG"
elif command -v convert &> /dev/null; then
    echo_step "Converting icon to PNG with ImageMagick..."
    convert -background none assets/icon.svg -resize 256x256 "${APPDIR}/usr/share/icons/hicolor/256x256/apps/asthetic-clipboard.png"
    echo_success "Icon converted to PNG"
else
    echo "Warning: Neither inkscape nor ImageMagick found. Skipping PNG icon generation."
    echo "AppImage will use SVG icon only."
fi

# Copy icon to AppDir root (required by AppImage)
cp assets/icon.svg "${APPDIR}/asthetic-clipboard.svg"
if [ -f "${APPDIR}/usr/share/icons/hicolor/256x256/apps/asthetic-clipboard.png" ]; then
    cp "${APPDIR}/usr/share/icons/hicolor/256x256/apps/asthetic-clipboard.png" "${APPDIR}/asthetic-clipboard.png"
fi

# Copy desktop file to AppDir root (required by AppImage)
cp asthetic-clipboard.desktop "${APPDIR}/"

# Step 5: Copy AppRun script
echo_step "Copying AppRun script..."
cp appimage/AppRun "${APPDIR}/"
chmod +x "${APPDIR}/AppRun"
echo_success "AppRun script installed"

# Step 6: Download linuxdeploy if not present
if [ ! -f "linuxdeploy-${ARCH}.AppImage" ]; then
    echo_step "Downloading linuxdeploy..."
    wget -q "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-${ARCH}.AppImage"
    chmod +x "linuxdeploy-${ARCH}.AppImage"
    echo_success "linuxdeploy downloaded"
fi

# Download GTK plugin if not present
if [ ! -f "linuxdeploy-plugin-gtk.sh" ]; then
    echo_step "Downloading linuxdeploy GTK plugin..."
    wget -q "https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh"
    chmod +x "linuxdeploy-plugin-gtk.sh"
    echo_success "GTK plugin downloaded"
fi

# Step 7: Run linuxdeploy to bundle dependencies
echo_step "Bundling dependencies with linuxdeploy..."
export DEPLOY_GTK_VERSION=4
export OUTPUT="${APPNAME}-${ARCH}.AppImage"

./linuxdeploy-${ARCH}.AppImage \
    --appdir="${APPDIR}" \
    --plugin=gtk \
    --output=appimage \
    --executable="${APPDIR}/usr/bin/asthetic-clipboard" \
    --desktop-file="${APPDIR}/asthetic-clipboard.desktop" \
    --icon-file="${APPDIR}/asthetic-clipboard.svg"

echo_success "AppImage created successfully!"
echo ""
echo "=================================="
echo "✓ Build Complete!"
echo "=================================="
echo "AppImage location: ./${OUTPUT}"
echo ""
echo "To run the AppImage:"
echo "  chmod +x ${OUTPUT}"
echo "  ./${OUTPUT}                  # Launch GUI"
echo "  ./${OUTPUT} --daemon         # Run daemon"
echo "  ./${OUTPUT} --tui            # Run TUI"
echo ""
