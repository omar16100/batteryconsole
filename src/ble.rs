//! BLE battery service implementation using CoreBluetooth on macOS

use log::{debug, warn};
use std::time::Duration;
use thiserror::Error;


/// BLE errors
#[derive(Error, Debug)]
pub enum BleError {
    #[error("BLE error: {0}")]
    BleError(String),

    #[error("Timeout")]
    Timeout,
}

/// Battery info from BLE device
#[derive(Debug, Clone)]
pub struct BleBatteryInfo {
    pub name: String,
    pub level: u8,
}

/// Get battery levels using Swift helper (most reliable on macOS)
#[cfg(target_os = "macos")]
pub async fn get_ble_batteries() -> Result<Vec<BleBatteryInfo>, BleError> {
    use tokio::process::Command;

    debug!("Using Swift CoreBluetooth to read battery levels");

    // Create Swift code to read battery levels
    let swift_code = r#"
import CoreBluetooth
import Foundation

class BatteryReader: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate {
    var centralManager: CBCentralManager!
    var results: [String: Int] = [:]
    var peripherals: [CBPeripheral] = []
    var pendingCount = 0
    let semaphore = DispatchSemaphore(value: 0)
    let batteryServiceUUID = CBUUID(string: "180F")
    let batteryLevelUUID = CBUUID(string: "2A19")

    override init() {
        super.init()
        centralManager = CBCentralManager(delegate: self, queue: DispatchQueue.main, options: [CBCentralManagerOptionShowPowerAlertKey: false])
    }

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        if central.state == .poweredOn {
            let connected = central.retrieveConnectedPeripherals(withServices: [batteryServiceUUID])
            if connected.isEmpty {
                semaphore.signal()
                return
            }
            pendingCount = connected.count
            for peripheral in connected {
                peripheral.delegate = self
                peripherals.append(peripheral)
                central.connect(peripheral, options: nil)
            }
            DispatchQueue.main.asyncAfter(deadline: .now() + 5) { self.semaphore.signal() }
        } else {
            semaphore.signal()
        }
    }

    func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        peripheral.discoverServices([batteryServiceUUID])
    }

    func centralManager(_ central: CBCentralManager, didFailToConnect peripheral: CBPeripheral, error: Error?) {
        pendingCount -= 1
        if pendingCount == 0 { semaphore.signal() }
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        guard error == nil else { pendingCount -= 1; if pendingCount == 0 { semaphore.signal() }; return }
        for service in peripheral.services ?? [] {
            if service.uuid == batteryServiceUUID {
                peripheral.discoverCharacteristics([batteryLevelUUID], for: service)
            }
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverCharacteristicsFor service: CBService, error: Error?) {
        guard error == nil else { pendingCount -= 1; if pendingCount == 0 { semaphore.signal() }; return }
        for char in service.characteristics ?? [] {
            if char.uuid == batteryLevelUUID { peripheral.readValue(for: char) }
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didUpdateValueFor characteristic: CBCharacteristic, error: Error?) {
        if error == nil, characteristic.uuid == batteryLevelUUID, let data = characteristic.value, !data.isEmpty {
            let level = Int(data[0])
            let name = peripheral.name ?? "Unknown"
            results[name] = level
        }
        pendingCount -= 1
        if pendingCount == 0 { semaphore.signal() }
    }

    func run() {
        RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.5))
        _ = semaphore.wait(timeout: .now() + 6)
    }
}

let reader = BatteryReader()
reader.run()

// Output as simple key=value pairs
for (name, level) in reader.results {
    print("\(name)=\(level)")
}
"#;

    // Run Swift interpreter
    let output = Command::new("swift")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let mut child = match output {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to spawn Swift: {}", e);
            return Err(BleError::BleError(format!("Failed to run Swift: {}", e)));
        }
    };

    // Write Swift code to stdin
    if let Some(stdin) = child.stdin.as_mut() {
        use tokio::io::AsyncWriteExt;
        if let Err(e) = stdin.write_all(swift_code.as_bytes()).await {
            warn!("Failed to write Swift code: {}", e);
            return Err(BleError::BleError(format!("Failed to write Swift code: {}", e)));
        }
    }

    // Wait for output with timeout
    let output = tokio::time::timeout(Duration::from_secs(10), child.wait_with_output()).await;

    let output = match output {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            warn!("Swift process error: {}", e);
            return Err(BleError::BleError(format!("Swift error: {}", e)));
        }
        Err(_) => {
            warn!("Swift timeout");
            return Err(BleError::Timeout);
        }
    };

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        if let Some((name, level_str)) = line.split_once('=') {
            if let Ok(level) = level_str.parse::<u8>() {
                debug!("{}: {}%", name, level);
                results.push(BleBatteryInfo {
                    name: name.to_string(),
                    level,
                });
            }
        }
    }

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!("Swift stderr: {}", stderr);
    }

    Ok(results)
}

#[cfg(not(target_os = "macos"))]
pub async fn get_ble_batteries() -> Result<Vec<BleBatteryInfo>, BleError> {
    Err(BleError::BleError("BLE battery reading only supported on macOS".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ble_battery_info() {
        let info = BleBatteryInfo {
            name: "MX Keys".to_string(),
            level: 75,
        };
        assert_eq!(info.name, "MX Keys");
        assert_eq!(info.level, 75);
    }
}
