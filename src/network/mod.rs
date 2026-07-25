// =====================================================================
// RetroLAN VPN - Network Engine Module
// Handles User-Space WireGuard routing, TUN adapter lifecycle,
// split-tunnel subnet management, and Layer-2 broadcast reflection.
// =====================================================================

pub mod interface;
pub mod reflector;

use std::sync::Arc;
use tokio::sync::Mutex;
use boringtun::noise::Tunn;
use crate::network::interface::VirtualAdapter;
use crate::network::reflector::BroadcastReflector;

/// Represents the active state of our gaming VPN engine.
#[allow(dead_code)]
pub struct VpnEngine {
    /// The virtual network adapter (TUN/Wintun) assigned to the system.
    adapter: Arc<Mutex<VirtualAdapter>>,
    /// Layer-2 UDP broadcast reflector for classic LAN game discovery.
    reflector: Arc<BroadcastReflector>,
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

        Ok(Self {
            adapter: Arc::new(Mutex::new(adapter)),
            reflector,
            tunnel: None,
            offline_lan_mode: false,
        })
    }

    /// Shuts down the engine cleanly, ensuring the OS adapter and reflectors are removed.
    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        tracing::info!("Shutting down RetroLAN VPN Engine...");
        
        // Stop broadcast replication
        self.reflector.stop().await;

        // Teardown OS virtual network interface
        let mut adapter = self.adapter.lock().await;
        adapter.teardown()?;
        
        Ok(())
    }
}
