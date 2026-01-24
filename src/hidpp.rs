//! HID++ protocol implementation for Logitech devices

use hidapi::HidDevice;
use log::{debug, warn};
use thiserror::Error;

/// HID++ Feature IDs
#[allow(dead_code)]
pub mod features {
    pub const ROOT: u16 = 0x0000;
    pub const BATTERY_STATUS: u16 = 0x1000;
    pub const UNIFIED_BATTERY: u16 = 0x1004;
}

/// HID++ Report types
pub mod report {
    pub const SHORT: u8 = 0x10;
    pub const LONG: u8 = 0x11;
    pub const SHORT_LEN: usize = 7;
    pub const LONG_LEN: usize = 20;
}

/// Software ID for our application
const SW_ID: u8 = 0x01;

/// Device indices to try
const DEVICE_INDICES: &[u8] = &[0x01, 0x02, 0xFF];

/// HID++ errors
#[derive(Error, Debug)]
pub enum HidppError {
    #[error("HID communication error: {0}")]
    HidError(#[from] hidapi::HidError),

    #[error("Feature not found: 0x{0:04X}")]
    FeatureNotFound(u16),

    #[allow(dead_code)]
    #[error("Invalid response length: expected {expected}, got {actual}")]
    InvalidResponseLength { expected: usize, actual: usize },

    #[error("Device error: code 0x{0:02X}")]
    DeviceError(u8),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Battery feature not supported")]
    BatteryNotSupported,
}

/// Battery status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    Discharging,
    Recharging,
    AlmostFull,
    Full,
    SlowRecharge,
    InvalidBattery,
    ThermalError,
    Unknown(u8),
}

impl From<u8> for BatteryStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => BatteryStatus::Discharging,
            1 => BatteryStatus::Recharging,
            2 => BatteryStatus::AlmostFull,
            3 => BatteryStatus::Full,
            4 => BatteryStatus::SlowRecharge,
            5 => BatteryStatus::InvalidBattery,
            6 => BatteryStatus::ThermalError,
            v => BatteryStatus::Unknown(v),
        }
    }
}

impl std::fmt::Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryStatus::Discharging => write!(f, "discharging"),
            BatteryStatus::Recharging => write!(f, "charging"),
            BatteryStatus::AlmostFull => write!(f, "almost full"),
            BatteryStatus::Full => write!(f, "full"),
            BatteryStatus::SlowRecharge => write!(f, "slow charging"),
            BatteryStatus::InvalidBattery => write!(f, "invalid battery"),
            BatteryStatus::ThermalError => write!(f, "thermal error"),
            BatteryStatus::Unknown(v) => write!(f, "unknown({})", v),
        }
    }
}

/// Battery information
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub level: u8,
    pub status: BatteryStatus,
    #[allow(dead_code)]
    pub next_level: Option<u8>,
}

/// HID++ device wrapper
pub struct HidppDevice<'a> {
    device: &'a HidDevice,
    timeout_ms: i32,
}

impl<'a> HidppDevice<'a> {
    /// Create new HID++ device wrapper
    pub fn new(device: &'a HidDevice) -> Self {
        Self {
            device,
            timeout_ms: 2000,
        }
    }

    /// Set timeout in milliseconds
    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout_ms: i32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Send HID++ short report and receive response
    fn send_short(
        &self,
        device_index: u8,
        feature_index: u8,
        function: u8,
        params: &[u8; 3],
    ) -> Result<[u8; report::SHORT_LEN], HidppError> {
        let mut request = [0u8; report::SHORT_LEN];
        request[0] = report::SHORT;
        request[1] = device_index;
        request[2] = feature_index;
        request[3] = (function << 4) | SW_ID;
        request[4..7].copy_from_slice(params);

        debug!("TX: {:02X?}", request);
        // Try output report first, then feature report
        match self.device.write(&request) {
            Ok(written) => debug!("Written {} bytes (output report)", written),
            Err(e) => {
                debug!("Output report failed: {}, trying feature report", e);
                match self.device.send_feature_report(&request) {
                    Ok(_) => debug!("Feature report sent"),
                    Err(e2) => {
                        debug!("Feature report also failed: {}", e2);
                        return Err(HidppError::HidError(e));
                    }
                }
            }
        }

        let mut response = [0u8; report::LONG_LEN];
        let mut retries = 5;

        loop {
            let len = self.device.read_timeout(&mut response, self.timeout_ms)?;
            if len == 0 {
                debug!("Read timeout (no data)");
                return Err(HidppError::Timeout);
            }

            debug!("RX ({} bytes): {:02X?}", len, &response[..len]);

            // Check for error response (Sub ID 0x8F)
            if (response[0] == report::SHORT || response[0] == report::LONG)
                && response[1] == device_index
                && response[2] == 0x8F
            {
                debug!("HID++ error response: code 0x{:02X}", response[5]);
                return Err(HidppError::DeviceError(response[5]));
            }

            // Check if this is our response
            if (response[0] == report::SHORT || response[0] == report::LONG)
                && response[1] == device_index
                && response[2] == feature_index
            {
                let mut result = [0u8; report::SHORT_LEN];
                result.copy_from_slice(&response[..report::SHORT_LEN]);
                return Ok(result);
            }

            retries -= 1;
            if retries == 0 {
                warn!("Max retries reached waiting for response");
                return Err(HidppError::Timeout);
            }
        }
    }

    /// Discover feature index by feature ID
    fn discover_feature(&self, device_index: u8, feature_id: u16) -> Result<u8, HidppError> {
        debug!(
            "Discovering feature 0x{:04X} on device index 0x{:02X}",
            feature_id, device_index
        );

        let params = [
            (feature_id >> 8) as u8,
            (feature_id & 0xFF) as u8,
            0x00,
        ];

        let response = self.send_short(device_index, 0x00, 0x00, &params)?;
        let feature_index = response[4];

        if feature_index == 0 {
            return Err(HidppError::FeatureNotFound(feature_id));
        }

        debug!("Feature 0x{:04X} -> index {}", feature_id, feature_index);
        Ok(feature_index)
    }

    /// Get battery info using UNIFIED_BATTERY feature (0x1004)
    fn get_unified_battery(
        &self,
        device_index: u8,
        feature_index: u8,
    ) -> Result<BatteryInfo, HidppError> {
        debug!("Querying UNIFIED_BATTERY at index {}", feature_index);

        let response = self.send_short(device_index, feature_index, 0x00, &[0, 0, 0])?;

        let level = response[4];
        let next_level = if response[5] > 0 {
            Some(response[5])
        } else {
            None
        };
        let status = BatteryStatus::from(response[6]);

        debug!(
            "Battery: level={}, next_level={:?}, status={:?}",
            level, next_level, status
        );

        Ok(BatteryInfo {
            level,
            status,
            next_level,
        })
    }

    /// Get battery info using BATTERY_STATUS feature (0x1000)
    fn get_battery_status(
        &self,
        device_index: u8,
        feature_index: u8,
    ) -> Result<BatteryInfo, HidppError> {
        debug!("Querying BATTERY_STATUS at index {}", feature_index);

        let response = self.send_short(device_index, feature_index, 0x00, &[0, 0, 0])?;

        let level = response[4];
        let next_level = if response[5] > 0 {
            Some(response[5])
        } else {
            None
        };
        let status = BatteryStatus::from(response[6]);

        debug!(
            "Battery: level={}, next_level={:?}, status={:?}",
            level, next_level, status
        );

        Ok(BatteryInfo {
            level,
            status,
            next_level,
        })
    }

    /// Try to get battery with a specific device index
    fn try_get_battery(&self, device_index: u8) -> Result<BatteryInfo, HidppError> {
        // Try UNIFIED_BATTERY first (newer feature)
        if let Ok(index) = self.discover_feature(device_index, features::UNIFIED_BATTERY) {
            return self.get_unified_battery(device_index, index);
        }

        // Fall back to BATTERY_STATUS
        if let Ok(index) = self.discover_feature(device_index, features::BATTERY_STATUS) {
            return self.get_battery_status(device_index, index);
        }

        Err(HidppError::BatteryNotSupported)
    }

    /// Get battery info (tries multiple device indices)
    pub fn get_battery(&self) -> Result<BatteryInfo, HidppError> {
        for &device_index in DEVICE_INDICES {
            debug!("Trying device index 0x{:02X}", device_index);
            match self.try_get_battery(device_index) {
                Ok(info) => return Ok(info),
                Err(e) => {
                    debug!("Device index 0x{:02X} failed: {}", device_index, e);
                }
            }
        }

        Err(HidppError::BatteryNotSupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_status_from() {
        assert_eq!(BatteryStatus::from(0), BatteryStatus::Discharging);
        assert_eq!(BatteryStatus::from(1), BatteryStatus::Recharging);
        assert_eq!(BatteryStatus::from(2), BatteryStatus::AlmostFull);
        assert_eq!(BatteryStatus::from(3), BatteryStatus::Full);
        assert_eq!(BatteryStatus::from(99), BatteryStatus::Unknown(99));
    }

    #[test]
    fn test_battery_status_display() {
        assert_eq!(format!("{}", BatteryStatus::Discharging), "discharging");
        assert_eq!(format!("{}", BatteryStatus::Recharging), "charging");
        assert_eq!(format!("{}", BatteryStatus::Full), "full");
    }
}
