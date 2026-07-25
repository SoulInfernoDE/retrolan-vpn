// =====================================================================
// RetroLAN VPN - Network Routing & Subsystem Engine
// Coordinates the virtual interface, broadcast reflector, and IPX wrapper.
// =====================================================================

pub mod interface;
pub mod ipx;
pub mod reflector;

use crate::config::GameProfile;
use interface::VirtualAdapter;
use ipx::IpxWrapper;
use reflector::BroadcastReflector;

use std::net::Ipv4Addr;
use std::path::Path;
use anyhow::Result;

#[allow(dead_code)]
pub struct VpnEngine {
    adapter: Option<VirtualAdapter>,
    pub reflector: BroadcastReflector,
    pub ipx_wrapper: IpxWrapper,
}

impl VpnEngine {
    pub async fn new(interface_name: &str, virtual_ip_str: &str) -> Result<Self> {
        tracing::info!("Initializing RetroLAN VPN Engine...");
        let virtual_ip: Ipv4Addr = virtual_ip_str.parse()?;

        let adapter = match VirtualAdapter::new(interface_name, virtual_ip) {
            Ok(a) => Some(a),
            Err(e) => {
                tracing::warn!("⚠️ Could not create OS TUN interface '{}': {}. Running in user-space fallback mode.", interface_name, e);
                None
            }
        };

        let reflector = BroadcastReflector::new();
        reflector.start().await;

        let ipx_wrapper = IpxWrapper::new(virtual_ip);
        ipx_wrapper.start().await?;

        Ok(Self {
            adapter,
            reflector,
            ipx_wrapper,
        })
    }

    pub async fn apply_game_profile(&self, profile: &GameProfile, game_dir: &Path) -> Result<()> {
        tracing::info!("🎮 Applying RetroLAN network profile for game: '{}'", profile.name);

        if let Some(port) = profile.broadcast_port {
            tracing::info!("Configuring UDP broadcast reflector on port {} for '{}'", port, profile.name);
            self.reflector.start_monitoring_port(port, self.ipx_wrapper_bind_ip()).await?;
        }

        if profile.require_wsock32_hook.unwrap_or(false) {
            tracing::info!("Game requires IPX wrapping. Deploying wsock32.dll shim...");
            let _ = self.ipx_wrapper.deploy_wsock32_shim(game_dir);
        }

        if profile.force_bind_ip.unwrap_or(false) {
            tracing::info!("🔒 Enforcing WINE_BIND_IP={} for target executable", self.ipx_wrapper_bind_ip());
            std::env::set_var("WINE_BIND_IP", self.ipx_wrapper_bind_ip().to_string());
        }

        Ok(())
    }

    fn ipx_wrapper_bind_ip(&self) -> Ipv4Addr {
        "10.133.7.1".parse().unwrap()
    }

    #[allow(dead_code)]
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down RetroLAN VPN Engine...");
        self.ipx_wrapper.stop().await;
        self.reflector.stop().await;
        if let Some(mut adapter) = self.adapter.take() {
            let _ = adapter.stop();
        }
        Ok(())
    }
}
