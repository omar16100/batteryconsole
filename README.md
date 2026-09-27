# BatteryConsole

A command-line tool to check battery levels for Logitech MX devices on macOS.

## Features

- Read battery levels from Logitech MX devices connected via Bluetooth
- Uses the standard BLE Battery Service (no special drivers needed)
- Optional HID++ mode for devices behind a Logitech USB receiver

## Devices

Verified by the author with MX Keys and MX Master 3 (see [todo.md](todo.md)).

- **BLE mode (default):** tries to read every connected Bluetooth peripheral that exposes the standard Battery Service (UUID 0x180F). It does not filter by vendor, so other devices may be listed too. Results are keyed by device name (two devices with the same name show once), and devices whose read fails are left out.
- **HID++ mode (`--hidpp`):** queries Logitech (vendor ID 0x046D) HID++ interfaces. Product IDs with friendly names in [src/devices.rs](src/devices.rs): MX Keys, MX Keys S, MX Master 3, MX Master 3S, MX Anywhere 3, MX Anywhere 3S, Bolt Receiver. Other Logitech devices fall back to the product string the device reports.

## Installation

### Homebrew

```bash
brew install omar16100/tap/batteryconsole
```

The formula ([omar16100/homebrew-tap](https://github.com/omar16100/homebrew-tap/blob/main/Formula/batteryconsole.rb)) builds the `v0.1.0` tag from source, so Homebrew installs Rust as a build dependency.

### From Source

Requires Rust toolchain. Install via [rustup](https://rustup.rs/).

```bash
git clone https://github.com/omar16100/batteryconsole.git
cd batteryconsole
cargo build --release
```

The binary will be at `./target/release/batteryconsole`.

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

By default BatteryConsole pipes a small Swift program into the `swift` interpreter. That program uses macOS CoreBluetooth to list connected peripherals exposing the standard BLE Battery Service (UUID 0x180F) and reads the Battery Level characteristic (UUID 0x2A19). The Rust side parses the `name=level` lines it prints and gives up after 10 seconds.

With `--hidpp`, BatteryConsole opens Logitech HID++ interfaces through `hidapi` and asks each device for the UNIFIED_BATTERY feature (0x1004), falling back to BATTERY_STATUS (0x1000).

Architecture notes: [docs/c4model.md](docs/c4model.md).

## Known Limitations

- **macOS only:** BLE mode uses CoreBluetooth, which is specific to macOS
- **0% while charging:** MX Master 3 reports 0% over the BLE Battery Service while it is plugged in; charging state is not detected yet
- **Needs the `swift` command:** BLE mode runs the Swift interpreter (Xcode or the Xcode Command Line Tools)

## Requirements

- macOS (no minimum version has been verified)
- BLE mode: `swift` on `PATH`, Bluetooth enabled, devices paired and connected via Bluetooth
- HID++ mode: devices reachable as HID devices (for example through a Logitech USB receiver), usually with `sudo`

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please open an issue or submit a pull request.

## Acknowledgments

- Built with Rust
- Uses CoreBluetooth via Swift for BLE communication
