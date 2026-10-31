//! Device and Windows System Identity utilities.
//!
//! Provides functions to extract the persistent, unique Windows `MachineGuid`
//! from `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid`, retrieve rich system
//! information via `sysinfo`, and generate disambiguated device names to prevent
//! device collision in multi-device households.

use serde::{Deserialize, Serialize};

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
};

/// Hardware and operating system metrics for the device.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    /// Disambiguated device name (e.g. `DESKTOP-FAMSHIELD (688F)`)
    pub computer_name: String,
    /// Host name reported by OS / sysinfo
    pub host_name: String,
    /// Operating system name (e.g. `Windows`, `macOS`, `Linux`)
    pub os_name: String,
    /// Full OS version string (e.g. `Windows 11 Pro 23H2 (Build 22631)`)
    pub os_version: String,
    /// Kernel version
    pub kernel_version: String,
    /// CPU architecture (e.g. `x86_64`, `aarch64`)
    pub cpu_arch: String,
    /// CPU brand / model name
    pub cpu_brand: String,
    /// Logical CPU core count
    pub cpu_cores: usize,
    /// Total physical RAM in bytes
    pub total_memory_bytes: u64,
    /// Used physical RAM in bytes
    pub used_memory_bytes: u64,
    /// Operating system uptime in seconds since boot
    pub uptime_seconds: u64,
    /// Windows Machine GUID if available
    pub machine_guid: Option<String>,
}

/// Retrieve the hardware-independent, unique Windows installation GUID.
///
/// Reads from `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid`.
/// This value is generated during Windows OS installation and remains constant
/// across reboots and network changes, serving as a reliable unique machine fingerprint.
pub fn get_machine_guid() -> Option<String> {
    #[cfg(windows)]
    {
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
    #[cfg(not(windows))]
    {
        None
    }
}

/// Retrieve the base hostname of the computer.
///
/// Uses the specialized `sysinfo` library to query the OS host name,
/// falling back to the `COMPUTERNAME` environment variable, and finally
/// a safe default `"DESKTOP-FAMSHIELD"`.
pub fn get_computer_name() -> String {
    sysinfo::System::host_name()
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty())
        .or_else(|| {
            std::env::var("COMPUTERNAME")
                .ok()
                .filter(|h| !h.trim().is_empty())
        })
        .unwrap_or_else(|| "DESKTOP-FAMSHIELD".to_string())
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

/// Collect comprehensive hardware and OS information using the specialized `sysinfo` library.
pub fn get_system_info() -> SystemInfo {
    let mut sys = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing()
            .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram())
            .with_cpu(sysinfo::CpuRefreshKind::nothing()),
    );
    sys.refresh_memory();

    let host_name = get_computer_name();

    let os_name = sysinfo::System::name().unwrap_or_else(|| {
        if cfg!(windows) {
            "Windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macOS".to_string()
        } else {
            "Unknown OS".to_string()
        }
    });

    let os_version = sysinfo::System::long_os_version()
        .or_else(sysinfo::System::os_version)
        .unwrap_or_else(|| "Unknown Version".to_string());

    let kernel_version =
        sysinfo::System::kernel_version().unwrap_or_else(|| "Unknown Kernel".to_string());

    let detected_arch = sysinfo::System::cpu_arch();
    let cpu_arch = if detected_arch.is_empty() {
        std::env::consts::ARCH.to_string()
    } else {
        detected_arch
    };

    let cpus = sys.cpus();
    let cpu_brand = cpus
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|b| !b.is_empty())
        .unwrap_or_else(|| "Unknown CPU".to_string());
    let cpu_cores = cpus.len();

    let total_memory_bytes = sys.total_memory();
    let used_memory_bytes = sys.used_memory();
    let uptime_seconds = sysinfo::System::uptime();
    let machine_guid = get_machine_guid();
    let computer_name = get_disambiguated_device_name(None);

    SystemInfo {
        computer_name,
        host_name,
        os_name,
        os_version,
        kernel_version,
        cpu_arch,
        cpu_brand,
        cpu_cores,
        total_memory_bytes,
        used_memory_bytes,
        uptime_seconds,
        machine_guid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_machine_guid_windows() {
        let guid = get_machine_guid();
        if cfg!(windows) {
            assert!(guid.is_some(), "MachineGuid should be present on Windows");
            let g = guid.unwrap();
            assert!(
                g.contains('-'),
                "MachineGuid should look like a UUID: {}",
                g
            );
        } else {
            assert!(guid.is_none());
        }
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

    #[test]
    fn test_get_system_info() {
        let info = get_system_info();
        assert!(!info.computer_name.is_empty());
        assert!(!info.host_name.is_empty());
        assert!(!info.os_name.is_empty());
        println!("System info collected: {:?}", info);
    }
}
