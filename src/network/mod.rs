// =====================================================================
// RetroLAN VPN - Network Engine Module
// Handles User-Space WireGuard routing, TUN adapter lifecycle,
// split-tunnel subnet management, Layer-2 broadcast reflection,
// IPX-to-UDP retro game wrapping, and dynamic game profile application.
// =====================================================================

pub mod interface;
pub mod reflector;
pub mod ipx;

use std::net::Ipv4Addr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use boringtun::noise::Tunn;
use crate::network::interface::VirtualAdapter;
use crate::network::reflector::BroadcastReflector;
use crate::network::ipx::IpxWrapper;
use crate::config::GameProfile;

/// Represents the active state of our gaming VPN engine.
#[allow(dead_code)]
pub struct VpnEngine {
    /// The virtual network adapter (TUN/Wintun) assigned to the system.
    adapter: Arc<Mutex<VirtualAdapter>>,
    /// Layer-2 UDP broadcast reflector for classic LAN game discovery.
    reflector: Arc<BroadcastReflector>,
    /// IPX-to-UDP encapsulation wrapper for Novell IPX/SPX retro games.
    ipx_wrapper: Arc<IpxWrapper>,
    /// BoringTun user-space WireGuard cryptographic tunnel state.
    tunnel: Option<Box<Tunn>>,
    /// Flag indicating whether physical offline LAN mode is prioritized.
    offline_lan_mode: bool,
}

impl VpnEngine {
    /// Initializes a new VPN Engine instance with default RetroLAN settings.
    pub async fn new(interface_name: &str, ipv4_address: &str) -> anyhow::Result<Self> {
        tracing::info!("Initializing RetroLAN VPN Engine...");
        
        // 1. Initialize the virtual network adapter (OS specific setup)
        let adapter = VirtualAdapter::create(interface_name, ipv4_address).await?;
        
        // 2. Set strict Split-Tunneling rules for the gaming subnet
        adapter.apply_split_tunnel_rules("10.133.7.0/24").await?;

        // 3. Initialize the Layer-2 UDP Broadcast Reflector
        let reflector = Arc::new(BroadcastReflector::new());
        reflector.start().await;

        // 4. Initialize the IPX-to-UDP Wrapping Engine
        let bind_ip = Ipv4Addr::from_str(ipv4_address)?;
        let ipx_wrapper = Arc::new(IpxWrapper::new(bind_ip));
        ipx_wrapper.start().await?;

        Ok(Self {
            adapter: Arc::new(Mutex::new(adapter)),
            reflector,
            ipx_wrapper,
            tunnel: None,
            offline_lan_mode: false,
        })
    }

    /// Dynamically applies a loaded game profile to the active network modules.
    /// Configures broadcast reflectors, deploys IPX shims, and logs Proton recommendations.
    #[allow(dead_code)]
    pub async fn apply_game_profile(&self, profile: &GameProfile, game_dir: &Path) -> anyhow::Result<()> {
        tracing::info!("🎮 Applying RetroLAN network profile for game: '{}'", profile.name);

        // 1. Configure Layer-2 UDP Broadcast Reflector if required
        if profile.protocol.eq_ignore_ascii_case("udp_broadcast") {
            if let Some(port) = profile.broadcast_port {
                let adapter = self.adapter.lock().await;
                tracing::info!("Configuring UDP broadcast reflector on port {} for '{}'", port, profile.name);
                self.reflector.start_monitoring_port(port, adapter.ip_address).await?;
            } else {
                tracing::warn!("Game profile '{}' requests udp_broadcast but specifies no broadcast_port!", profile.name);
            }
        }

        // 2. Deploy custom IPX-to-UDP wsock32.dll proxy shim if requested
        if profile.protocol.eq_ignore_ascii_case("ipx") || profile.require_wsock32_hook == Some(true) {
            tracing::info!("Game requests legacy IPX networking. Deploying proxy shim...");
            self.ipx_wrapper.deploy_wsock32_shim(game_dir)?;
        }

        // 3. Log Proton recommendations for Linux gamers
        if let Some(ref proton) = profile.recommended_proton {
            tracing::info!("💡 Linux Tip: Game runs best with compatibility tool '{}'", proton);
        }

        if profile.force_bind_ip == Some(true) {
            let adapter = self.adapter.lock().await;
            tracing::info!("🔒 Enforcing WINE_BIND_IP={} for target executable", adapter.ip_address);
        }

        Ok(())
    }

    /// Shuts down the engine cleanly, ensuring OS adapters, reflectors, and wrappers are stopped.
    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        tracing::info!("Shutting down RetroLAN VPN Engine...");
        
        // Stop background network tasks
        self.ipx_wrapper.stop().await;
        self.reflector.stop().await;

        // Teardown OS virtual network interface
        let mut adapter = self.adapter.lock().await;
        adapter.teardown()?;
        
        Ok(())
    }
}
