//! Device definitions for Logitech MX devices

/// Logitech vendor ID
pub const LOGITECH_VENDOR_ID: u16 = 0x046D;

/// Known Logitech device
#[derive(Debug, Clone)]
pub struct Device {
    pub product_id: u16,
    pub name: &'static str,
}

impl Device {
    pub const fn new(product_id: u16, name: &'static str) -> Self {
        Self { product_id, name }
    }
}

/// Known Logitech MX devices
pub static KNOWN_DEVICES: &[Device] = &[
    Device::new(0xB35B, "MX Keys"),
    Device::new(0xB023, "MX Master 3"),
    Device::new(0xB034, "MX Master 3S"),
    Device::new(0xB369, "MX Keys S"),
    Device::new(0xB35F, "MX Anywhere 3"),
    Device::new(0xB037, "MX Anywhere 3S"),
    Device::new(0xC548, "Bolt Receiver"),
];

/// Find device name by product ID
pub fn find_device_name(vendor_id: u16, product_id: u16) -> Option<&'static str> {
    if vendor_id != LOGITECH_VENDOR_ID {
        return None;
    }
    KNOWN_DEVICES
        .iter()
        .find(|d| d.product_id == product_id)
        .map(|d| d.name)
}

/// Check if vendor ID is Logitech
pub fn is_logitech(vendor_id: u16) -> bool {
    vendor_id == LOGITECH_VENDOR_ID
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_device_name() {
        assert_eq!(
            find_device_name(LOGITECH_VENDOR_ID, 0xB35B),
            Some("MX Keys")
        );
        assert_eq!(
            find_device_name(LOGITECH_VENDOR_ID, 0xB023),
            Some("MX Master 3")
        );
        assert_eq!(find_device_name(LOGITECH_VENDOR_ID, 0x0000), None);
        assert_eq!(find_device_name(0x0000, 0xB35B), None);
    }

    #[test]
    fn test_is_logitech() {
        assert!(is_logitech(LOGITECH_VENDOR_ID));
        assert!(!is_logitech(0x0000));
    }
}
