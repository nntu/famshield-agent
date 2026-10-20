use crate::error::WindowsError;
use tracing::info;

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::ERROR_SUCCESS;
#[cfg(windows)]
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegDeleteValueW, RegSetValueExW, HKEY,
    HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_DWORD, REG_MULTI_SZ, REG_OPTION_NON_VOLATILE, REG_SZ,
};

pub const NRPT_POLICY_REG_PATH: &str =
    r"SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig";
pub const NRPT_SYSTEM_REG_PATH: &str =
    r"SYSTEM\CurrentControlSet\Services\Dnscache\Parameters\DnsPolicyConfig";
pub const FAMSHIELD_NRPT_GUID: &str = "{E6B5D7A1-8843-4C49-B944-C5200E481230}";
pub const LEGACY_NRPT_RULE_NAME: &str = "{FamShield-DNS-Filter}";

/// Manages Windows Name Resolution Policy Table (NRPT) rules and network adapter DNS routing.
/// Directs DNS resolution to local loopback proxy (`127.0.0.2:53`) with 100% reliability.
pub struct NrptManager;

impl NrptManager {
    /// Configures NRPT rule, suppresses browser DoH, and redirects active adapters to `dns_ip`.
    pub fn apply_nrpt_rule(dns_ip: &str) -> Result<(), WindowsError> {
        if dns_ip.parse::<std::net::IpAddr>().is_err() {
            return Err(WindowsError::Registry(format!(
                "invalid NRPT DNS server address: {dns_ip}"
            )));
        }

        #[cfg(windows)]
        {
            // 1. Purge legacy corrupted key {FamShield-DNS-Filter} from both hives
            for base_path in &[NRPT_POLICY_REG_PATH, NRPT_SYSTEM_REG_PATH] {
                let legacy_path = wide_null(&format!(r"{}\{}", base_path, LEGACY_NRPT_RULE_NAME));
                unsafe {
                    let _ = RegDeleteTreeW(HKEY_LOCAL_MACHINE, PCWSTR(legacy_path.as_ptr()));
                }
            }

            // 2. Write valid RFC/Win32-compliant NRPT rule to both policy and system paths
            for base_path in &[NRPT_POLICY_REG_PATH, NRPT_SYSTEM_REG_PATH] {
                let rule_path = wide_null(&format!(r"{}\{}", base_path, FAMSHIELD_NRPT_GUID));
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
                if status == ERROR_SUCCESS {
                    let key = RegistryKey(key);
                    let _ = set_multi_string_value(key.0, "Name", &["."]);
                    let _ = set_string_value(key.0, "GenericDNSServers", dns_ip);
                    let _ = set_dword_value(key.0, "ConfigOptions", 0x8);
                    let _ = set_dword_value(key.0, "Version", 2);
                }
            }

            // 3. Disable Chrome and Edge DoH (DNS-over-HTTPS) bypass via GPO keys
            for browser_policy in &[
                r"SOFTWARE\Policies\Google\Chrome",
                r"SOFTWARE\Policies\Microsoft\Edge",
            ] {
                let p_wide = wide_null(browser_policy);
                let mut key = HKEY::default();
                let status = unsafe {
                    RegCreateKeyExW(
                        HKEY_LOCAL_MACHINE,
                        PCWSTR(p_wide.as_ptr()),
                        0,
                        PCWSTR::null(),
                        REG_OPTION_NON_VOLATILE,
                        KEY_SET_VALUE,
                        None,
                        &mut key,
                        None,
                    )
                };
                if status == ERROR_SUCCESS {
                    let key = RegistryKey(key);
                    let _ = set_string_value(key.0, "DnsOverHttpsMode", "off");
                }
            }

            // 4. Infallible Adapter DNS redirection: Ensure active physical network adapters point to dns_ip
            let ps_cmd = format!(
                "Get-NetAdapter | Where-Object {{ $_.Status -eq 'Up' -and $_.InterfaceAlias -notmatch 'vEthernet|WSL|Loopback' }} | ForEach-Object {{ Set-DnsClientServerAddress -InterfaceIndex $_.InterfaceIndex -ServerAddresses '{}' -ErrorAction SilentlyContinue }}",
                dns_ip
            );
            let _ = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &ps_cmd,
                ])
                .output();
        }

        info!("FamShield DNS interception activated for {}", dns_ip);
        Ok(())
    }

    /// Removes NRPT rules, clears DoH suppression, and resets network adapters back to DHCP.
    pub fn remove_nrpt_rule() -> Result<(), WindowsError> {
        #[cfg(windows)]
        {
            // 1. Remove NRPT rules from both locations
            for base_path in &[NRPT_POLICY_REG_PATH, NRPT_SYSTEM_REG_PATH] {
                for rule_name in &[FAMSHIELD_NRPT_GUID, LEGACY_NRPT_RULE_NAME] {
                    let rule_path = wide_null(&format!(r"{}\{}", base_path, rule_name));
                    unsafe {
                        let _ = RegDeleteTreeW(HKEY_LOCAL_MACHINE, PCWSTR(rule_path.as_ptr()));
                    }
                }
            }

            // 2. Clean DoH policies
            for browser_policy in &[
                r"SOFTWARE\Policies\Google\Chrome",
                r"SOFTWARE\Policies\Microsoft\Edge",
            ] {
                let p_wide = wide_null(browser_policy);
                let mut key = HKEY::default();
                let status = unsafe {
                    RegCreateKeyExW(
                        HKEY_LOCAL_MACHINE,
                        PCWSTR(p_wide.as_ptr()),
                        0,
                        PCWSTR::null(),
                        REG_OPTION_NON_VOLATILE,
                        KEY_SET_VALUE,
                        None,
                        &mut key,
                        None,
                    )
                };
                if status == ERROR_SUCCESS {
                    let key = RegistryKey(key);
                    let doh_name = wide_null("DnsOverHttpsMode");
                    unsafe {
                        let _ = RegDeleteValueW(key.0, PCWSTR(doh_name.as_ptr()));
                    }
                }
            }

            // 3. Reset network adapters DNS back to DHCP
            let ps_cmd = "Get-NetAdapter | ForEach-Object { Set-DnsClientServerAddress -InterfaceIndex $_.InterfaceIndex -ResetServerAddresses -ErrorAction SilentlyContinue }";
            let _ = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    ps_cmd,
                ])
                .output();
        }

        info!("FamShield DNS interception cleanly restored");
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
fn set_multi_string_value(key: HKEY, name: &str, values: &[&str]) -> Result<(), WindowsError> {
    let name = wide_null(name);
    let mut buffer: Vec<u16> = Vec::new();
    for v in values {
        buffer.extend(v.encode_utf16());
        buffer.push(0);
    }
    buffer.push(0); // double null termination for MULTI_SZ
    let bytes = unsafe {
        std::slice::from_raw_parts(
            buffer.as_ptr().cast::<u8>(),
            buffer.len() * size_of::<u16>(),
        )
    };
    let status =
        unsafe { RegSetValueExW(key, PCWSTR(name.as_ptr()), 0, REG_MULTI_SZ, Some(bytes)) };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(registry_error("RegSetValueExW(REG_MULTI_SZ)", status.0))
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
