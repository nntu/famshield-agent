#![allow(clippy::field_reassign_with_default)]

use crate::error::WindowsError;
use tracing::{info, warn};

#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FwpmEngineClose0, FwpmEngineOpen0, FwpmFilterAdd0, FwpmFilterDeleteById0,
    FWPM_CONDITION_IP_REMOTE_ADDRESS, FWPM_CONDITION_IP_REMOTE_PORT, FWPM_FILTER0,
    FWPM_FILTER_CONDITION0, FWPM_LAYER_ALE_AUTH_CONNECT_V4, FWPM_LAYER_ALE_AUTH_CONNECT_V6,
    FWPM_SESSION0, FWPM_SESSION_FLAG_DYNAMIC, FWPM_SUBLAYER_UNIVERSAL, FWP_ACTION_BLOCK,
    FWP_ACTION_PERMIT, FWP_CONDITION_VALUE0, FWP_MATCH_EQUAL, FWP_UINT16, FWP_UINT32, FWP_UINT8,
    FWP_V4_ADDR_AND_MASK, FWP_V4_ADDR_MASK,
};

/// RAII Guard for WFP User-Mode Session (`fwpmu.dll`).
/// Uses `FWPM_SESSION_FLAG_DYNAMIC` to guarantee that Windows automatically
/// purges all registered filters if the process crashes or exits unexpectedly,
/// eliminating the risk of network lockup.
pub struct WfpSessionGuard {
    #[cfg(windows)]
    engine_handle: HANDLE,
    active: bool,
    filter_ids: Vec<u64>,
}

impl WfpSessionGuard {
    #[cfg(windows)]
    pub fn open_dynamic(session_name: &str) -> Result<Self, WindowsError> {
        let mut handle = HANDLE::default();
        let session = FWPM_SESSION0 {
            flags: FWPM_SESSION_FLAG_DYNAMIC,
            ..Default::default()
        };

        unsafe {
            let res = FwpmEngineOpen0(
                None,
                1, // RPC_C_AUTHN_WINNT
                None,
                Some(&session),
                &mut handle,
            );
            if res != 0 {
                return Err(WindowsError::Wfp(format!(
                    "FwpmEngineOpen0 failed with status code: {:#x}",
                    res
                )));
            }
        }

        info!(
            "Opened dynamic WFP user-mode session for '{}'",
            session_name
        );
        Ok(Self {
            engine_handle: handle,
            active: true,
            filter_ids: Vec::new(),
        })
    }

    #[cfg(not(windows))]
    pub fn open_dynamic(session_name: &str) -> Result<Self, WindowsError> {
        info!(
            "Mock dynamic WFP session for '{}' on non-windows platform",
            session_name
        );
        Ok(Self {
            active: true,
            filter_ids: Vec::new(),
        })
    }

    /// Block all external outbound Port 53 (DNS) traffic, forcing local resolution through
    /// FamShield's loopback DNS filter while permitting explicitly configured upstream IPs.
    #[cfg(windows)]
    pub fn block_external_dns(
        &mut self,
        permitted_upstream_ips: &[std::net::Ipv4Addr],
    ) -> Result<Vec<u64>, WindowsError> {
        let mut added = Vec::new();

        // 1. Permit Loopback DNS (127.0.0.0/8 : 53) with higher priority (Weight 15)
        let permit_loopback_id = self.add_permit_loopback_dns_v4()?;
        added.push(permit_loopback_id);

        // 2. Permit configured upstream DNS servers with higher priority (Weight 15)
        for &upstream_ip in permitted_upstream_ips {
            let permit_upstream_id = self.add_permit_ip_dns_v4(upstream_ip)?;
            added.push(permit_upstream_id);
        }

        // 3. Block all remaining outbound IPv4 Port 53 (Weight 10)
        let block_v4_id = self.add_block_dns_v4()?;
        added.push(block_v4_id);

        // 4. Block all outbound IPv6 Port 53 (Weight 10)
        let block_v6_id = self.add_block_dns_v6()?;
        added.push(block_v6_id);

        self.filter_ids.extend(&added);
        info!(
            "Successfully installed {} WFP DNS enforcement filters",
            added.len()
        );
        Ok(added)
    }

    #[cfg(not(windows))]
    pub fn block_external_dns(
        &mut self,
        _permitted_upstream_ips: &[std::net::Ipv4Addr],
    ) -> Result<Vec<u64>, WindowsError> {
        let mock_ids = vec![101, 102, 103, 104];
        self.filter_ids.extend(&mock_ids);
        info!("Mock registered 4 WFP Port 53 enforcement filters");
        Ok(mock_ids)
    }

    #[cfg(windows)]
    fn add_permit_loopback_dns_v4(&self) -> Result<u64, WindowsError> {
        let mut port_val = FWP_CONDITION_VALUE0::default();
        port_val.r#type = FWP_UINT16;
        port_val.Anonymous.uint16 = 53;

        let mut mask = FWP_V4_ADDR_AND_MASK::default();
        mask.addr = 0x7f000000;
        mask.mask = 0xff000000;
        let mut addr_val = FWP_CONDITION_VALUE0::default();
        addr_val.r#type = FWP_V4_ADDR_MASK;
        addr_val.Anonymous.v4AddrMask = &mut mask;

        let mut conditions = [
            FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_REMOTE_PORT,
                matchType: FWP_MATCH_EQUAL,
                conditionValue: port_val,
            },
            FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_REMOTE_ADDRESS,
                matchType: FWP_MATCH_EQUAL,
                conditionValue: addr_val,
            },
        ];

        let mut filter = FWPM_FILTER0::default();
        filter.displayData.name = windows::core::PWSTR(
            windows::core::w!("FamShield-Permit-Loopback-DNS").as_ptr() as *mut u16,
        );
        filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V4;
        filter.subLayerKey = FWPM_SUBLAYER_UNIVERSAL;
        filter.weight.r#type = FWP_UINT8;
        filter.weight.Anonymous.uint8 = 15;
        filter.action.r#type = FWP_ACTION_PERMIT;
        filter.numFilterConditions = conditions.len() as u32;
        filter.filterCondition = conditions.as_mut_ptr();

        let mut filter_id = 0u64;
        let res =
            unsafe { FwpmFilterAdd0(self.engine_handle, &filter, None, Some(&mut filter_id)) };
        if res != 0 {
            return Err(WindowsError::Wfp(format!(
                "FwpmFilterAdd0 (permit loopback) failed with code: {:#x}",
                res
            )));
        }
        Ok(filter_id)
    }

    #[cfg(windows)]
    fn add_permit_ip_dns_v4(&self, ip: std::net::Ipv4Addr) -> Result<u64, WindowsError> {
        let mut port_val = FWP_CONDITION_VALUE0::default();
        port_val.r#type = FWP_UINT16;
        port_val.Anonymous.uint16 = 53;

        let mut addr_val = FWP_CONDITION_VALUE0::default();
        addr_val.r#type = FWP_UINT32;
        addr_val.Anonymous.uint32 = u32::from_be_bytes(ip.octets());

        let mut conditions = [
            FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_REMOTE_PORT,
                matchType: FWP_MATCH_EQUAL,
                conditionValue: port_val,
            },
            FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_REMOTE_ADDRESS,
                matchType: FWP_MATCH_EQUAL,
                conditionValue: addr_val,
            },
        ];

        let mut filter = FWPM_FILTER0::default();
        filter.displayData.name = windows::core::PWSTR(
            windows::core::w!("FamShield-Permit-Upstream-DNS").as_ptr() as *mut u16,
        );
        filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V4;
        filter.subLayerKey = FWPM_SUBLAYER_UNIVERSAL;
        filter.weight.r#type = FWP_UINT8;
        filter.weight.Anonymous.uint8 = 15;
        filter.action.r#type = FWP_ACTION_PERMIT;
        filter.numFilterConditions = conditions.len() as u32;
        filter.filterCondition = conditions.as_mut_ptr();

        let mut filter_id = 0u64;
        let res =
            unsafe { FwpmFilterAdd0(self.engine_handle, &filter, None, Some(&mut filter_id)) };
        if res != 0 {
            return Err(WindowsError::Wfp(format!(
                "FwpmFilterAdd0 (permit upstream {}) failed with code: {:#x}",
                ip, res
            )));
        }
        Ok(filter_id)
    }

    #[cfg(windows)]
    fn add_block_dns_v4(&self) -> Result<u64, WindowsError> {
        let mut port_val = FWP_CONDITION_VALUE0::default();
        port_val.r#type = FWP_UINT16;
        port_val.Anonymous.uint16 = 53;

        let mut conditions = [FWPM_FILTER_CONDITION0 {
            fieldKey: FWPM_CONDITION_IP_REMOTE_PORT,
            matchType: FWP_MATCH_EQUAL,
            conditionValue: port_val,
        }];

        let mut filter = FWPM_FILTER0::default();
        filter.displayData.name = windows::core::PWSTR(
            windows::core::w!("FamShield-Block-External-DNS-IPv4").as_ptr() as *mut u16,
        );
        filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V4;
        filter.subLayerKey = FWPM_SUBLAYER_UNIVERSAL;
        filter.weight.r#type = FWP_UINT8;
        filter.weight.Anonymous.uint8 = 10;
        filter.action.r#type = FWP_ACTION_BLOCK;
        filter.numFilterConditions = conditions.len() as u32;
        filter.filterCondition = conditions.as_mut_ptr();

        let mut filter_id = 0u64;
        let res =
            unsafe { FwpmFilterAdd0(self.engine_handle, &filter, None, Some(&mut filter_id)) };
        if res != 0 {
            return Err(WindowsError::Wfp(format!(
                "FwpmFilterAdd0 (block v4 dns) failed with code: {:#x}",
                res
            )));
        }
        Ok(filter_id)
    }

    #[cfg(windows)]
    fn add_block_dns_v6(&self) -> Result<u64, WindowsError> {
        let mut port_val = FWP_CONDITION_VALUE0::default();
        port_val.r#type = FWP_UINT16;
        port_val.Anonymous.uint16 = 53;

        let mut conditions = [FWPM_FILTER_CONDITION0 {
            fieldKey: FWPM_CONDITION_IP_REMOTE_PORT,
            matchType: FWP_MATCH_EQUAL,
            conditionValue: port_val,
        }];

        let mut filter = FWPM_FILTER0::default();
        filter.displayData.name = windows::core::PWSTR(
            windows::core::w!("FamShield-Block-External-DNS-IPv6").as_ptr() as *mut u16,
        );
        filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V6;
        filter.subLayerKey = FWPM_SUBLAYER_UNIVERSAL;
        filter.weight.r#type = FWP_UINT8;
        filter.weight.Anonymous.uint8 = 10;
        filter.action.r#type = FWP_ACTION_BLOCK;
        filter.numFilterConditions = conditions.len() as u32;
        filter.filterCondition = conditions.as_mut_ptr();

        let mut filter_id = 0u64;
        let res =
            unsafe { FwpmFilterAdd0(self.engine_handle, &filter, None, Some(&mut filter_id)) };
        if res != 0 {
            return Err(WindowsError::Wfp(format!(
                "FwpmFilterAdd0 (block v6 dns) failed with code: {:#x}",
                res
            )));
        }
        Ok(filter_id)
    }

    /// Remove a specific filter by its ID.
    #[cfg(windows)]
    pub fn remove_filter(&mut self, filter_id: u64) -> Result<(), WindowsError> {
        if self.active && !self.engine_handle.is_invalid() {
            let res = unsafe { FwpmFilterDeleteById0(self.engine_handle, filter_id) };
            if res != 0 {
                return Err(WindowsError::Wfp(format!(
                    "FwpmFilterDeleteById0 failed for filter {}: {:#x}",
                    filter_id, res
                )));
            }
            self.filter_ids.retain(|&id| id != filter_id);
            info!("Removed WFP filter {}", filter_id);
        }
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn remove_filter(&mut self, filter_id: u64) -> Result<(), WindowsError> {
        self.filter_ids.retain(|&id| id != filter_id);
        info!("Mock removed WFP filter {}", filter_id);
        Ok(())
    }

    /// Remove all registered filters from this session.
    pub fn remove_all_filters(&mut self) -> Result<(), WindowsError> {
        let ids = self.filter_ids.clone();
        for id in ids {
            if let Err(e) = self.remove_filter(id) {
                warn!("Failed to remove WFP filter {}: {}", id, e);
            }
        }
        self.filter_ids.clear();
        Ok(())
    }

    pub fn filter_count(&self) -> usize {
        self.filter_ids.len()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Drop for WfpSessionGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        if self.active && !self.engine_handle.is_invalid() {
            let _ = self.remove_all_filters();
            unsafe {
                let _ = FwpmEngineClose0(self.engine_handle);
            }
            info!("Released WFP user-mode session handle cleanly");
            self.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wfp_session_lifecycle() {
        // Dynamic session creation should either succeed or fail gracefully without panicking
        match WfpSessionGuard::open_dynamic("FamShieldTestSession") {
            Ok(mut guard) => {
                assert!(guard.is_active());
                assert_eq!(guard.filter_count(), 0);

                // Attempting filter addition - requires admin rights on Windows
                let _ = guard.block_external_dns(&[]);
                let _ = guard.remove_all_filters();
            }
            Err(e) => {
                // If unprivileged on Windows, FwpmEngineOpen0 or permissions return an error cleanly
                tracing::warn!("WFP session test skipped due to environment: {}", e);
            }
        }
    }
}
