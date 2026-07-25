// =====================================================================
// RetroLAN VPN - Main Application Entry Point
// Combines User-Space WireGuard routing, Layer-2 broadcast reflection,
// IPX wrapping, TOML configuration, Steamworks signaling, Proton control,
// and local zero-config mDNS physical LAN discovery.
// =====================================================================

mod config;
mod discovery;
mod network;
mod proton;
mod steam;

use discovery::MdnsDiscoveryEngine;
use network::VpnEngine;
use proton::ProtonManager;
use steam::{SteamEngine, RETROLAN_DEV_APP_ID};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging to terminal
    tracing_subscriber::fmt::init();
    tracing::info!("🚀 Starting RetroLAN-VPN Core Engine...");

    // 1. Scan Linux system for Steam Proton compatibility tools
    let mut proton_manager = match ProtonManager::new() {
        Ok(manager) => {
            for tool in &manager.installed_tools {
                tracing::info!("  -> Found Proton Flavor: {}", tool.name);
            }
            Some(manager)
        }
        Err(err) => {
            tracing::debug!("Proton manager disabled or not running on standard Linux path: {}", err);
            None
        }
    };

    // 2. Attempt to initialize Steamworks SDK signaling (graceful offline fallback)
    let steam_engine = match SteamEngine::init(RETROLAN_DEV_APP_ID)? {
        Some(engine) => {
            tracing::info!("🌐 Operating in Steam Online Mode (Relay & Lobby Signaling available).");
            Some(engine)
        }
        None => {
            tracing::warn!("🔌 Operating in Physical Offline LAN Mode (Local mDNS discovery will be prioritized).");
            None
        }
    };

    // 3. Initialize local mDNS peer discovery engine for physical offline LAN parties
    let mdns_engine = MdnsDiscoveryEngine::new("RetroLAN-PC-1", "10.133.7.1", "4x/example+wg+pubkey=");
    if let Err(err) = mdns_engine.start_broadcasting("192.168.1.100").await {
        tracing::debug!("Note on mDNS broadcast: {}", err);
    }
    if let Err(err) = mdns_engine.start_discovery().await {
        tracing::debug!("Note on mDNS discovery: {}", err);
    }

    // 4. Initialize our virtual gaming adapter on IP 10.133.7.1
    let mut vpn_engine = VpnEngine::new("retrolan0", "10.133.7.1").await?;

    // 5. Test: Load community game database and check game requirements
    if let Ok(db) = config::GameDatabase::load_from_file(Path::new("games.toml")) {
        if let Some(flatout_profile) = db.find_by_process("FlatOut2.exe") {
            vpn_engine.apply_game_profile(flatout_profile, Path::new(".")).await?;
            
            if let Some(ref recommended) = flatout_profile.recommended_proton {
                if let Some(ref mut pm) = proton_manager {
                    tracing::info!("Checking if required tool '{}' is present on system...", recommended);
                    let _ = pm.ensure_proton_installed(recommended).await;
                }
            }
        }
    }

    // 6. Test: If Steam is active, simulate creating a signaling lobby
    if let Some(ref steam) = steam_engine {
        if let Ok(_lobby_id) = steam.create_signaling_lobby(8).await {
            steam.broadcast_wireguard_handshake("4x/example+wg+pubkey=", "10.133.7.1").await?;
        }
    }

    tracing::info!("✔ RetroLAN-VPN Engine successfully initialized and running!");
    
    // Simulate clean shutdown of all subsystems
    let _ = mdns_engine.shutdown().await;
    if let Some(ref steam) = steam_engine {
        steam.shutdown().await;
    }
    vpn_engine.shutdown().await?;

    Ok(())
}
