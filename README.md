# BatteryConsole

A command-line tool to check battery levels for Logitech MX devices on macOS.

## Features

- Read battery levels from Logitech MX devices connected via Bluetooth
- Uses standard BLE Battery Service (no special drivers needed)
- Fast and lightweight

## Supported Devices

- MX Keys
- MX Keys S
- MX Master 3
- MX Master 3S
- MX Anywhere 3
- MX Anywhere 3S
- Other Logitech devices with BLE Battery Service

## Installation

### From Source

Requires Rust toolchain. Install via [rustup](https://rustup.rs/).

```bash
git clone https://github.com/yourusername/batteryconsole.git
cd batteryconsole
cargo build --release
```

The binary will be at `./target/release/batteryconsole`.

### Homebrew (coming soon)

```bash
brew install batteryconsole
```

## Usage

```bash
# Check battery levels (default BLE mode)
batteryconsole

# Example output:
# MX Keys: 100%
# MX Master 3: 50%

# Verbose output
batteryconsole -v

# Use HID++ protocol (for USB receiver, requires sudo)
sudo batteryconsole --hidpp
```

## Options

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Show verbose/debug output |
| `--hidpp` | Use HID++ protocol instead of BLE (requires sudo) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## How It Works

BatteryConsole uses macOS CoreBluetooth to query the standard BLE Battery Service (UUID 0x180F) exposed by Logitech devices. This is the same method macOS System Settings uses to display battery levels.

## Known Limitations

- **macOS only** - Uses CoreBluetooth APIs specific to macOS
- **Devices report 0% while charging** - The BLE Battery Service returns 0 when the device is plugged in
- **Requires Swift runtime** - Uses Swift interpreter for CoreBluetooth access

## Requirements

- macOS 12.0 or later
- Bluetooth enabled
- Devices paired and connected via Bluetooth

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please open an issue or submit a pull request.

## Acknowledgments

- Built with Rust
- Uses CoreBluetooth via Swift for BLE communication
