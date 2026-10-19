use crate::error::WindowsError;
use tracing::info;

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
#[cfg(windows)]
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY, HKEY_LOCAL_MACHINE,
    KEY_SET_VALUE, REG_DWORD, REG_OPTION_NON_VOLATILE, REG_SZ,
};

pub const NRPT_REG_PATH: &str = r"SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig";
pub const FAMSHIELD_NRPT_RULE_NAME: &str = "{FamShield-DNS-Filter}";

/// Manages Windows Name Resolution Policy Table (NRPT) rules.
/// Routes queries for the root namespace (".") to the local DNS proxy (`127.0.0.2:53`)
/// without modifying physical network adapters.
pub struct NrptManager;

impl NrptManager {
    /// Configures the NRPT rule directing DNS resolution to `dns_ip` (default `127.0.0.2`).
    pub fn apply_nrpt_rule(dns_ip: &str) -> Result<(), WindowsError> {
        if dns_ip.parse::<std::net::IpAddr>().is_err() {
            return Err(WindowsError::Registry(format!(
                "invalid NRPT DNS server address: {dns_ip}"
            )));
        }

        #[cfg(windows)]
        {
            let rule_path = wide_null(&format!(r"{}\{}", NRPT_REG_PATH, FAMSHIELD_NRPT_RULE_NAME));
            let mut key = HKEY::default();
            let status = unsafe {
                RegCreateKeyExW(
                    HKEY_LOCAL_MACHINE,
                    PCWSTR(rule_path.as_ptr()),
                    0,
                    PCWSTR::null(),
                    REG_OPTION_NON_VOLATILE,
                    KEY_SET_VALUE,
                    None,
                    &mut key,
                    None,
                )
            };
            if status != ERROR_SUCCESS {
                return Err(registry_error("RegCreateKeyExW", status.0));
            }
            let key = RegistryKey(key);

            set_string_value(key.0, "Name", ".")?;
            set_string_value(key.0, "GenericDNSServers", dns_ip)?;
            set_dword_value(key.0, "ConfigOptions", 0x8)?;
            set_dword_value(key.0, "Version", 1)?;
        }

        info!("Applying NRPT rule to route namespace '.' to {}", dns_ip);
        Ok(())
    }

    /// Removes the NRPT rule upon clean shutdown or uninstallation.
    pub fn remove_nrpt_rule() -> Result<(), WindowsError> {
        #[cfg(windows)]
        {
            let rule_path = wide_null(&format!(r"{}\{}", NRPT_REG_PATH, FAMSHIELD_NRPT_RULE_NAME));
            let status = unsafe { RegDeleteTreeW(HKEY_LOCAL_MACHINE, PCWSTR(rule_path.as_ptr())) };
            if status != ERROR_SUCCESS && status != ERROR_FILE_NOT_FOUND {
                return Err(registry_error("RegDeleteTreeW", status.0));
            }
        }

        info!("Removing FamShield NRPT rule from DnsPolicyConfig");
        Ok(())
    }
}

#[cfg(windows)]
struct RegistryKey(HKEY);

#[cfg(windows)]
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

#[cfg(windows)]
fn registry_error(operation: &str, status: u32) -> WindowsError {
    WindowsError::Registry(format!("{operation} failed with status code {status:#x}"))
}

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn set_string_value(key: HKEY, name: &str, value: &str) -> Result<(), WindowsError> {
    let name = wide_null(name);
    let value = wide_null(value);
    let bytes = unsafe {
        std::slice::from_raw_parts(value.as_ptr().cast::<u8>(), value.len() * size_of::<u16>())
    };
    let status = unsafe { RegSetValueExW(key, PCWSTR(name.as_ptr()), 0, REG_SZ, Some(bytes)) };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(registry_error("RegSetValueExW(REG_SZ)", status.0))
    }
}

#[cfg(windows)]
fn set_dword_value(key: HKEY, name: &str, value: u32) -> Result<(), WindowsError> {
    let name = wide_null(name);
    let bytes = value.to_le_bytes();
    let status = unsafe { RegSetValueExW(key, PCWSTR(name.as_ptr()), 0, REG_DWORD, Some(&bytes)) };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(registry_error("RegSetValueExW(REG_DWORD)", status.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_dns_server_address() {
        assert!(NrptManager::apply_nrpt_rule("not-an-ip-address").is_err());
    }
}
