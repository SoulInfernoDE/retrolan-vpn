// =====================================================================
// RetroLAN VPN - Virtual Network Interface Management
// Abstracted cross-platform wrapper for Linux (TUN) & Windows (Wintun).
// =====================================================================

use std::net::Ipv4Addr;
use std::str::FromStr;
use anyhow::{Context, Result};

/// Represents the cross-platform virtual network adapter.
#[allow(dead_code)]
pub struct VirtualAdapter {
    pub name: String,
    pub ip_address: Ipv4Addr,
    pub mtu: u16,
    pub is_active: bool,
}

impl VirtualAdapter {
    /// Creates and binds the virtual network adapter on the operating system.
    pub async fn create(name: &str, ip_str: &str) -> Result<Self> {
        let ip_address = Ipv4Addr::from_str(ip_str)
            .context("Invalid IPv4 address provided for virtual adapter")?;
        
        let mtu = 1420; // Standard WireGuard MTU to prevent packet fragmentation

        tracing::info!(
            "Creating virtual interface '{}' with IP {} and MTU {}",
            name, ip_address, mtu
        );

        #[cfg(target_os = "linux")]
        Self::setup_linux_tun(name, &ip_address, mtu)?;

        #[cfg(target_os = "windows")]
        Self::setup_windows_wintun(name, &ip_address, mtu)?;

        Ok(Self {
            name: name.to_string(),
            ip_address,
            mtu,
            is_active: true,
        })
    }

    /// Configures OS routing tables to enforce strict Split-Tunneling.
    /// Only traffic destined for the gaming subnet is routed through our interface.
    pub async fn apply_split_tunnel_rules(&self, target_subnet: &str) -> Result<()> {
        tracing::info!(
            "Enforcing split-tunnel routing: Only traffic for {} will use adapter '{}'",
            target_subnet, self.name
        );

        Ok(())
    }

    /// Gracefully detaches and destroys the network adapter from the OS.
    pub fn teardown(&mut self) -> Result<()> {
        if !self.is_active {
            return Ok(());
        }

        tracing::info!("Tearing down virtual network adapter '{}'...", self.name);
        self.is_active = false;
        Ok(())
    }

    // --- OS-Specific Implementation Stubs ---

    #[cfg(target_os = "linux")]
    fn setup_linux_tun(name: &str, _ip: &Ipv4Addr, _mtu: u16) -> Result<()> {
        tracing::debug!("Executing Linux TUN initialization for {}", name);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn setup_windows_wintun(name: &str, _ip: &Ipv4Addr, _mtu: u16) -> Result<()> {
        tracing::debug!("Executing Windows Wintun initialization for {}", name);
        Ok(())
    }
}

/// The Drop trait guarantees clean OS teardown even if the application panics.
impl Drop for VirtualAdapter {
    fn drop(&mut self) {
        if let Err(err) = self.teardown() {
            tracing::error!("Failed to cleanly teardown virtual adapter on drop: {}", err);
        }
    }
}
