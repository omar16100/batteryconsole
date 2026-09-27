# Architecture (C4)

Source of truth for how batteryconsole is put together. Update it with every architecture change.

## Context

A person on macOS runs `batteryconsole` in a terminal to see battery levels of Logitech MX keyboards and mice. In BLE mode it reads devices already paired and connected over Bluetooth; in HID++ mode it reads devices the HID stack enumerates (for example behind a Logitech USB receiver). It has no network access, no config file and no stored state.

External systems:

- **macOS CoreBluetooth**, reached through the `swift` interpreter (Xcode or Command Line Tools).
- **macOS HID stack**, reached through the `hidapi` crate (HID++ mode, usually needs `sudo`).
- **Logitech devices**: Bluetooth peripherals exposing the Battery Service (0x180F), or HID++ devices behind a USB receiver.

## Containers

One container: the `batteryconsole` CLI binary (Rust, single crate, `tokio` runtime). In BLE mode it spawns a short-lived `swift -` child process.

## Components

| File | Role |
|---|---|
| `src/main.rs` | `clap` argument parsing (`-v`, `--hidpp`), logger setup, dispatch to BLE or HID++ and printing |
| `src/ble.rs` | Embedded Swift program, child process spawn, 10 s timeout, parsing of `name=level` lines |
| `src/hidpp.rs` | HID++ 2.0 requests (short reports), feature discovery via the root feature, UNIFIED_BATTERY (0x1004) then BATTERY_STATUS (0x1000) |
| `src/devices.rs` | Logitech vendor ID and product ID to friendly-name table |
| `tests/integration.rs` | CLI checks via `cargo run -- --help` / `--version` |

## Data flow

BLE mode (default):

1. `main` calls `ble::get_ble_batteries`.
2. The embedded Swift source is written to `swift -` on stdin.
3. Swift uses CoreBluetooth `retrieveConnectedPeripherals(withServices: [180F])`, connects, reads characteristic 0x2A19 and prints `name=level` per device. Results are keyed by peripheral name, so same-named devices collapse to one line, and failed reads are omitted.
4. Rust parses stdout into `BleBatteryInfo` and prints `name: level%`.

HID++ mode (`--hidpp`):

1. `hidapi` enumerates HID devices; keep vendor 0x046D on usage pages 0xFF00 / 0xFF43, deduplicated by (vendor, product, serial).
2. For device indices 0x01, 0x02, 0xFF: discover UNIFIED_BATTERY, else BATTERY_STATUS, and read level plus status.
3. Print `name: level% (status)`.

## Decisions

- BLE access goes through the Swift interpreter instead of a Rust CoreBluetooth binding. Cost: needs `swift` on `PATH` and pays interpreter start-up on every run.
- BLE mode does not filter by vendor, so any connected peripheral with a readable Battery Service is listed.
- CI runs on `macos-latest` only. Tests cover pure logic and CLI flags, so they need no Bluetooth or HID hardware.
