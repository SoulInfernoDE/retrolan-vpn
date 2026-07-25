// =====================================================================
// RetroLAN VPN - Main Application Entry Point
// Combines User-Space WireGuard routing, Layer-2 broadcast reflection,
// IPX wrapping, TOML configuration, and Steamworks signaling.
// =====================================================================

mod config;
mod network;
mod steam;

use network::VpnEngine;
use steam::{SteamEngine, RETROLAN_DEV_APP_ID};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging to terminal
    tracing_subscriber::fmt::init();
    tracing::info!("🚀 Starting RetroLAN-VPN Core Engine...");

    // 1. Attempt to initialize Steamworks SDK signaling (graceful offline fallback)
    let steam_engine = match SteamEngine::init(RETROLAN_DEV_APP_ID)? {
        Some(engine) => {
            tracing::info!("🌐 Operating in Steam Online Mode (Relay & Lobby Signaling available).");
            Some(engine)
        }
        None => {
            tracing::warn!("🔌 Operating in Physical Offline LAN Mode (Local mDNS discovery only).");
            None
        }
    };

    // 2. Initialize our virtual gaming adapter on IP 10.133.7.1
    let mut vpn_engine = VpnEngine::new("retrolan0", "10.133.7.1").await?;

    // 3. Test: Load community game database and simulate applying a profile
    if let Ok(db) = config::GameDatabase::load_from_file(Path::new("games.toml")) {
        if let Some(flatout_profile) = db.find_by_process("FlatOut2.exe") {
            vpn_engine.apply_game_profile(flatout_profile, Path::new(".")).await?;
        }
    }

    // 4. Test: If Steam is active, simulate creating a signaling lobby
    if let Some(ref steam) = steam_engine {
        if let Ok(_lobby_id) = steam.create_signaling_lobby(8).await {
            steam.broadcast_wireguard_handshake("4x/example+wg+pubkey=", "10.133.7.1").await?;
        }
    }

    tracing::info!("✔ RetroLAN-VPN Engine successfully initialized and running!");
    
    // Simulate clean shutdown of all subsystems
    if let Some(ref steam) = steam_engine {
        steam.shutdown().await;
    }
    vpn_engine.shutdown().await?;

    Ok(())
}
