# 🔊 Clack

A lightweight, cross-platform mechanical keyboard sound emulator. Clack runs in the background and plays Cherry MX Blue-style sounds on every keypress — across all applications and workspaces.

**Features:**
- 🎹 Distinct sounds for different key types (spacebar, enter, escape, backspace, modifiers)
- 🎲 Subtle randomization (pitch ±3%, volume ±5%) for natural feel
- 💨 Ultra-low latency (<10ms) audio via lock-free engine
- 📦 Single binary, no runtime dependencies
- 🐧🍎🪟 Works on Linux (X11 + Wayland), macOS, and Windows

## Installation

### Linux & macOS
```bash
curl -fsSL https://raw.githubusercontent.com/ashrafpy/clack/main/install.sh | sh
```

### Windows
```powershell
irm https://raw.githubusercontent.com/ashrafpy/clack/main/install.ps1 | iex
```

### Build from Source
```bash
git clone https://github.com/ashrafpy/clack.git
cd clack
cargo build --release
# Binary at target/release/clack (or target/release/clack.exe on Windows)
```

## Usage
```bash
# Start with default settings
clack

# Adjust volume (0.0 to 1.0)
clack --volume 0.5

# Use a custom sound pack
clack --soundpack ~/my-sounds/
```

## Platform Setup

### Linux
You need to be in the `input` group for global key capture:
```bash
sudo usermod -aG input $USER
# Log out and back in for changes to take effect
```

### macOS
Grant **Accessibility** permissions when prompted, or manually:
System Settings → Privacy & Security → Accessibility → Enable for your terminal/Clack

### Windows
No additional setup needed. Run as Administrator if you want sounds in elevated windows.

## Custom Sound Packs
Create a directory with WAV files following this naming convention:
```
my-sounds/
├── generic_1.wav    # Regular keys (add _2, _3 for variations)
├── space_1.wav      # Spacebar
├── enter_1.wav      # Enter/Return
├── backspace_1.wav  # Backspace
├── escape_1.wav     # Escape
└── modifier_1.wav   # Shift, Ctrl, Alt, etc.
```

## License
MIT
