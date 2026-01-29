#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting installation of Asthetic Clipboard Manager...${NC}"

# 1. Check for Cargo
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo (Rust) is not installed. Please install Rust first."
    exit 1
fi

# 2. Build the project
echo -e "${BLUE}Building project (release mode)...${NC}"
cargo build --release

# 3. Create install directories
mkdir -p "$HOME/.local/bin"
mkdir -p "$HOME/.local/share/applications"
mkdir -p "$HOME/.local/share/icons/hicolor/scalable/apps"
mkdir -p "$HOME/.config/systemd/user"

# 4. Install Binaries
echo -e "${BLUE}Installing binaries...${NC}"
cp target/release/asthetic-clipboard "$HOME/.local/bin/"
cp target/release/daemon "$HOME/.local/bin/asthetic-clipboard-daemon"
cp target/release/tui "$HOME/.local/bin/asthetic-clipboard-tui"

# 5. Install Icon
echo -e "${BLUE}Installing icon...${NC}"
cp assets/icon.svg "$HOME/.local/share/icons/hicolor/scalable/apps/asthetic-clipboard.svg"

# 6. Install Desktop Entry
echo -e "${BLUE}Installing desktop entry...${NC}"
cp asthetic-clipboard.desktop "$HOME/.local/share/applications/"
update-desktop-database "$HOME/.local/share/applications" || true

# 7. Create and Install Systemd Service
echo -e "${BLUE}Configuring background daemon (systemd)...${NC}"
SERVICE_FILE="$HOME/.config/systemd/user/asthetic-clipboard.service"

cat > "$SERVICE_FILE" << EOF
[Unit]
Description=Asthetic Clipboard Manager Daemon
After=graphical-session.target

[Service]
ExecStart=%h/.local/bin/asthetic-clipboard-daemon
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

# 8. Reload and Start Service
systemctl --user daemon-reload
systemctl --user enable asthetic-clipboard.service
systemctl --user restart asthetic-clipboard.service

echo -e "${GREEN}Installation Complete!${NC}"
echo "------------------------------------------------"
echo "The daemon is running in the background."
echo "You can launch the GUI from your application menu or by running 'asthetic-clipboard'."
echo "You can launch the TUI by running 'asthetic-clipboard-tui'."
