//! BatteryConsole - Check Logitech MX device battery levels

mod ble;
mod devices;
mod hidpp;

use clap::Parser;
use log::{debug, error, info};

/// Check battery levels for Logitech MX devices
#[derive(Parser, Debug)]
#[command(name = "batteryconsole")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Use HID++ protocol instead of BLE (requires sudo)
    #[arg(long)]
    hidpp: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize logger
    let log_level = if args.verbose { "debug" } else { "warn" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .init();

    info!("Starting batteryconsole");

    if args.hidpp {
        if let Err(e) = run_hidpp() {
            error!("HID++ error: {}", e);
            std::process::exit(1);
        }
    } else {
        if let Err(e) = run_ble().await {
            error!("BLE error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_ble() -> Result<(), Box<dyn std::error::Error>> {
    debug!("Using BLE Battery Service");

    let results = ble::get_ble_batteries().await?;

    if results.is_empty() {
        println!("No Logitech devices with battery found");
        println!("Hint: Make sure devices are connected via Bluetooth");
    } else {
        for result in &results {
            println!("{}: {}%", result.name, result.level);
        }
    }

    Ok(())
}

fn run_hidpp() -> Result<(), Box<dyn std::error::Error>> {
    use hidapi::HidApi;
    use std::collections::HashSet;

    use crate::devices::{find_device_name, is_logitech};
    use crate::hidpp::HidppDevice;

    debug!("Using HID++ protocol");
    let api = HidApi::new()?;

    let mut seen_devices: HashSet<(u16, u16, String)> = HashSet::new();
    let mut found_any = false;

    const HIDPP_USAGE_PAGES: &[u16] = &[0xFF00, 0xFF43];

    for device_info in api.device_list() {
        let vendor_id = device_info.vendor_id();
        let product_id = device_info.product_id();
        let usage_page = device_info.usage_page();

        if !is_logitech(vendor_id) {
            continue;
        }

        if !HIDPP_USAGE_PAGES.contains(&usage_page) {
            continue;
        }

        let serial = device_info
            .serial_number()
            .unwrap_or_default()
            .to_string();

        let key = (vendor_id, product_id, serial.clone());
        if seen_devices.contains(&key) {
            continue;
        }
        seen_devices.insert(key);

        let device_name = find_device_name(vendor_id, product_id)
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                device_info
                    .product_string()
                    .unwrap_or("Unknown Device")
                    .to_string()
            });

        debug!("Found: {} (0x{:04X})", device_name, product_id);

        if let Ok(device) = device_info.open_device(&api) {
            let hidpp = HidppDevice::new(&device);
            if let Ok(battery) = hidpp.get_battery() {
                println!("{}: {}% ({})", device_name, battery.level, battery.status);
                found_any = true;
            }
        }
    }

    if !found_any {
        println!("No Logitech devices with battery found");
        println!("Hint: Try running with sudo for HID access");
    }

    Ok(())
}
