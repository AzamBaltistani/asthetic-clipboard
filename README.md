# Asthetic Clipboard Manager

A simple, fast, and persistent clipboard manager written in Rust.

## Features
- **Persistent History**: Saves your clipboard history to `~/.local/share/asthetic/clipboard/history.json`.
- **Pinning**: Press `p` to pin items so they don't get deleted when history is full.
- **TUI Interface**: Easy to use terminal interface (optional).
- **Wayland Support**: Designed to work on generic Linux setups (requires manual keybind).
- **Daemon**: Background process to monitor clipboard changes.

## Requirements

- **Linux** (Wayland or X11)
- **Rust Toolchain**
- **System Dependencies**:
  - `wl-clipboard` (for Wayland) OR `xclip` (for X11)
  - GTK4 development libraries (`libgtk-4-dev`)

## Installation

1. Build the project:
   ```bash
   cargo build --release
   ```
2. Binaries will be in `target/release/`.
   - `daemon`: The background monitor.
   - `asthetic-clipboard`: The main GUI client (GTK4).
   - `tui`: The terminal interface client.

## Usage

### 1. Start the Daemon
Run the daemon in the background to start recording clipboard history.
```bash
./target/release/daemon &
```
(You should add this to your startup scripts).

### 2. Open the UI
Run the client to view and select items.
```bash
./target/release/asthetic-clipboard
```

### 3. Open the TUI (Optional)
If you prefer the terminal interface:
```bash
./target/release/tui
```

### 4. Keybindings (TUI)
- **Up / Down / j / k**: Navigate list.
- **Enter**: Copy selected item to clipboard and exit.
- **p**: Pin/Unpin selected item.
- **d**: Delete selected item.
- **c**: Clear all unpinned items.
- **Esc / q**: Quit.

## Wayland Setup (Win+V)
To trigger this with `Win+V`, you must configure your Desktop Environment specifically.
- **GNOME**: Settings -> Keyboard -> View and Customize Shortcuts -> Custom Shortcuts. Add `Win+V` to run `/path/to/asthetic-clipboard`. (Note: you might need a terminal wrapper like `gnome-terminal -- /path/to/exe`).
- **Hyprland**: Add `bind = SUPER, V, exec, /path/to/asthetic-clipboard`.
