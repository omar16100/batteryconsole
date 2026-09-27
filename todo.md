# BatteryConsole TODO

## Completed
- [x] Project setup with Cargo.toml
- [x] devices.rs - Logitech device definitions
- [x] hidpp.rs - HID++ protocol implementation (works with USB receivers)
- [x] ble.rs - BLE Battery Service via CoreBluetooth/Swift (works with direct Bluetooth)
- [x] main.rs - CLI with BLE as default, --hidpp flag for USB receiver mode
- [x] Unit tests for devices, hidpp, and ble modules
- [x] Integration tests for CLI
- [x] Release build
- [x] Successfully reading battery levels from MX Keys and MX Master 3
- [x] 27 Sep 2026: README fixes (clone URL, Homebrew install via omar16100/tap, removed unbacked claims), docs/index.md, docs/c4model.md, macOS CI (build + test). Plan: docs/27092026_readme_ci_plan.md

## How It Works
- Default mode: Uses Swift/CoreBluetooth to query BLE Battery Service (0x180F)
- HID++ mode (--hidpp): Uses HID++ protocol for USB receiver connected devices

## Known Caveats
- **MX Master 3 reports 0% while charging** - The BLE Battery Service returns 0 when the device is plugged in for charging

## Pending
- [ ] Detect charging state (may require different GATT characteristic)
- [ ] Add JSON output format option
- [ ] Consider adding watch mode for continuous monitoring
- [ ] Test with Logi Bolt receiver
