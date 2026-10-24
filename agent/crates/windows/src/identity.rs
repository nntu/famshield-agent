//! Device and Windows System Identity utilities.
//!
//! Provides functions to extract the persistent, unique Windows `MachineGuid`
//! from `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid` and generate
//! disambiguated device names to prevent device collision in multi-device households.

use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
};

/// Retrieve the hardware-independent, unique Windows installation GUID.
///
/// Reads from `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid`.
/// This value is generated during Windows OS installation and remains constant
/// across reboots and network changes, serving as a reliable unique machine fingerprint.
pub fn get_machine_guid() -> Option<String> {
    let subkey: Vec<u16> = "SOFTWARE\\Microsoft\\Cryptography\0"
        .encode_utf16()
        .collect();
    let value_name: Vec<u16> = "MachineGuid\0".encode_utf16().collect();

    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return None;
        }

        let mut buf_len = 0u32;
        let mut val_type = REG_SZ;

        // First query to determine buffer size
        if RegQueryValueExW(
            hkey,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut val_type),
            None,
            Some(&mut buf_len),
        )
        .is_err()
            || buf_len == 0
        {
            let _ = RegCloseKey(hkey);
            return None;
        }

        let mut buffer = vec![0u8; buf_len as usize];
        let res = RegQueryValueExW(
            hkey,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut val_type),
            Some(buffer.as_mut_ptr()),
            Some(&mut buf_len),
        );

        let _ = RegCloseKey(hkey);

        if res.is_ok() && val_type == REG_SZ {
            let u16_slice: &[u16] =
                std::slice::from_raw_parts(buffer.as_ptr() as *const u16, buf_len as usize / 2);
            let s = String::from_utf16_lossy(u16_slice);
            let trimmed = s.trim_matches('\0').trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    None
}

/// Retrieve the base hostname of the computer.
pub fn get_computer_name() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "DESKTOP-FAMSHIELD".to_string())
}

/// Produce a disambiguated device name that is distinct even if multiple computers
/// share the same hostname (e.g. `DESKTOP-FAMSHIELD (688F)`).
pub fn get_disambiguated_device_name(custom_name: Option<&str>) -> String {
    if let Some(name) = custom_name {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let base = get_computer_name();
    if let Some(guid) = get_machine_guid() {
        let clean_guid = guid.replace('-', "");
        if clean_guid.len() >= 4 {
            let suffix = &clean_guid[clean_guid.len() - 4..];
            return format!("{} ({})", base, suffix.to_uppercase());
        }
    }

    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_machine_guid_windows() {
        let guid = get_machine_guid();
        assert!(guid.is_some(), "MachineGuid should be present on Windows");
        let g = guid.unwrap();
        assert!(
            g.contains('-'),
            "MachineGuid should look like a UUID: {}",
            g
        );
    }

    #[test]
    fn test_disambiguated_device_name() {
        // Custom name is preserved
        assert_eq!(get_disambiguated_device_name(Some("Máy Con")), "Máy Con");

        // Default name includes disambiguation suffix
        let default_name = get_disambiguated_device_name(None);
        assert!(!default_name.is_empty());
        println!("Disambiguated name: {}", default_name);
    }
}
