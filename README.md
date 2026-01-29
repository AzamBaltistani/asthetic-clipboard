# Asthetic Clipboard Manager

A simple, fast, and persistent clipboard manager written in Rust.

## Features
- **Persistent History**: Saves your clipboard history to `~/.local/share/asthetic/clipboard/history.json`.
- **Smart Timestamp**: Shows when an item was *originally* copied. Reusing an item keeps its original time.
- **Pinning**: Press `Pin` in the menu to keep important items (they won't be deleted when history is full).
- **Image Support**: Copy and paste images directly.
- **Background Daemon**: Automatically starts on login (via systemd).
- **Theme Support**: Light & Dark mode.

## Installation

### From Source (Recommended)

1. **Clone the repository**:
   ```bash
   git clone https://github.com/AzamBaltistani/asthetic-clipboard.git
   cd asthetic-clipboard
   ```

2. **Run the installation script**:
   ```bash
   ./install.sh
   ```
   This will:
   - Build the project (requires Rust).
   - Install binaries to `~/.local/bin`.
   - Install the desktop icon.
   - **Automatically set up and start the background daemon**.

## Usage

### 1. Bind a Shortcut (Important!)
Since this is a clipboard manager, you should bind a global shortcut to open it easily.

- **Command**: `asthetic-clipboard`
- **Recommended Shortcut**: `Super+V` or `Ctrl+Alt+V`

**To set this up**:
- **GNOME**: Settings -> Keyboard -> "View and Customize Shortcuts" -> "Custom Shortcuts" -> Add (+).
- **Hyprland**: Add `bind = SUPER, V, exec, asthetic-clipboard` to your config.

### 2. Basic Usage
- **Copy**: Just copy text or images as usual (Ctrl+C).
- **Open**: Press your shortcut (or run `asthetic-clipboard`).
- **Paste**: Click an item to copy it back to your clipboard.
- **Menu**: Click the `⋮` button on an item to Pin, Delete, or Save Image.

### 3. Terminal Interface (TUI)
If you prefer the terminal:
```bash
asthetic-clipboard-tui
```
- **Navigation**: Arrow keys / j, k
- **Select**: Enter
- **Pin**: p
- **Delete**: d
- **Quit**: q / Esc

## Troubleshooting

**Daemon not running?**
The installation script sets up a systemd service. Check its status:
```bash
systemctl --user status asthetic-clipboard.service
```
If it's not running, try restarting it:
```bash
systemctl --user restart asthetic-clipboard.service
```

## Uninstallation
To remove everything:
```bash
systemctl --user stop asthetic-clipboard.service
systemctl --user disable asthetic-clipboard.service
rm ~/.config/systemd/user/asthetic-clipboard.service
rm ~/.local/bin/asthetic-clipboard*
rm ~/.local/share/applications/asthetic-clipboard.desktop
rm ~/.local/share/icons/hicolor/scalable/apps/asthetic-clipboard.svg
```
