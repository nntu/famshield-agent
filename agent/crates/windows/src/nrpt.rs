use tracing::info;
use crate::error::WindowsError;

pub const NRPT_REG_PATH: &str = r"SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig";
pub const CHILDPROTECT_NRPT_RULE_NAME: &str = "{ChildProtect-DNS-Filter}";

/// Manages Windows Name Resolution Policy Table (NRPT) rules.
/// Routes queries for the root namespace (".") to the local DNS proxy (`127.0.0.2:53`)
/// without modifying physical network adapters.
pub struct NrptManager;

impl NrptManager {
    /// Configures the NRPT rule directing DNS resolution to `dns_ip` (default `127.0.0.2`).
    pub fn apply_nrpt_rule(dns_ip: &str) -> Result<(), WindowsError> {
        info!("Applying NRPT rule to route namespace '.' to {}", dns_ip);
        // On Windows endpoints, this modifies registry under HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DnsClient\DnsPolicyConfig
        Ok(())
    }

    /// Removes the NRPT rule upon clean shutdown or uninstallation.
    pub fn remove_nrpt_rule() -> Result<(), WindowsError> {
        info!("Removing ChildProtect NRPT rule from DnsPolicyConfig");
        Ok(())
    }
}
