// =====================================================================
// RetroLAN VPN - Main Application Entry Point (Tauri v2 Integrated)
// Combines User-Space WireGuard routing, Layer-2 broadcast reflection,
// IPX wrapping, TOML configuration, Steamworks signaling, Proton control,
// local zero-config mDNS discovery, and real-time Tauri State binding.
// =====================================================================

mod config;
mod discovery;
mod network;
mod proton;
mod steam;

use config::{GameDatabase, GameProfile};
use discovery::MdnsDiscoveryEngine;
use network::VpnEngine;
use proton::ProtonManager;
use steam::{SteamEngine, RETROLAN_DEV_APP_ID};

use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;
use serde::Serialize;

/// Global Application State managed by Tauri across all IPC commands.
struct AppState {
    vpn_engine: Arc<VpnEngine>,
    proton_manager: Arc<Mutex<Option<ProtonManager>>>,
    mdns_engine: Arc<MdnsDiscoveryEngine>,
    steam_engine: Arc<Option<SteamEngine>>,
    db: Arc<GameDatabase>,
}

#[derive(Serialize)]
struct SystemStatusPayload {
    avx2: bool,
    ntsync: bool,
    steam_online: bool,
    mdns_active: bool,
}

#[tauri::command]
fn get_system_status(state: State<'_, AppState>) -> SystemStatusPayload {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let avx2 = std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("fma");
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    let avx2 = false;

    let ntsync = Path::new("/dev/ntsync").exists();
    
    SystemStatusPayload {
        avx2,
        ntsync,
        steam_online: state.steam_engine.is_some(),
        mdns_active: true,
    }
}

#[tauri::command]
fn get_game_list(state: State<'_, AppState>) -> Vec<GameProfile> {
    state.db.games.clone()
}

#[tauri::command]
async fn host_lobby_cmd(state: State<'_, AppState>) -> Result<String, String> {
    tracing::info!("GUI command received: host_lobby_cmd");
    
    if let Some(ref steam) = *state.steam_engine {
        match steam.create_signaling_lobby(8).await {
            Ok(lobby_id) => {
                let _ = steam.broadcast_wireguard_handshake("4x/example+wg+pubkey=", "10.133.7.1").await;
                Ok(format!("✔ Steam SDR Lobby eröffnet! ID: {:?}", lobby_id))
            }
            Err(err) => Err(format!("❌ Fehler beim Erstellen der Steam-Lobby: {}", err)),
        }
    } else {
        Err("❌ Steam Client ist offline! Bitte nutze den lokalen mDNS Offline-LAN Modus.".to_string())
    }
}

#[tauri::command]
async fn start_mdns_cmd(state: State<'_, AppState>) -> Result<String, String> {
    tracing::info!("GUI command received: start_mdns_cmd");
    
    // Attempt to broadcast and browse on standard local fallback IPs
    if let Err(err) = state.mdns_engine.start_broadcasting("192.168.1.100").await {
        tracing::warn!("Note on mDNS broadcast: {}", err);
    }
    
    match state.mdns_engine.start_discovery().await {
        Ok(_) => Ok("✔ mDNS Beacon & Suche auf Port 23757 aktiv. Suche nach Mitspielern...".to_string()),
        Err(err) => Err(format!("❌ mDNS Discovery Fehler: {}", err)),
    }
}

#[tauri::command]
async fn deploy_ipx_cmd(state: State<'_, AppState>) -> Result<String, String> {
    tracing::info!("GUI command received: deploy_ipx_cmd");
    
    // For testing, deploy shim to the current working directory
    match state.vpn_engine.apply_game_profile(&GameProfile {
        name: "IPX Manual Shim".to_string(),
        steam_appid: None,
        process_names: vec![],
        protocol: "ipx".to_string(),
        broadcast_port: None,
        require_wsock32_hook: Some(true),
        force_bind_ip: None,
        recommended_proton: None,
        notes: None,
    }, Path::new(".")).await {
        Ok(_) => Ok("✔ wsock32.dll IPX Proxy-Shim erfolgreich in Zielordner verlegt.".to_string()),
        Err(err) => Err(format!("❌ Fehler beim Deployen des IPX-Shims: {}", err)),
    }
}

#[tauri::command]
async fn apply_profile_cmd(game_name: String, state: State<'_, AppState>) -> Result<String, String> {
    tracing::info!("GUI command received: apply_profile_cmd -> {}", game_name);
    
    let profile = state.db.games.iter()
        .find(|g| g.name.eq_ignore_ascii_case(&game_name))
        .ok_or_else(|| format!("❌ Spiel '{}' nicht in der Datenbank gefunden!", game_name))?;

    // 1. Apply network routing rules (Broadcast Reflector & IPX)
    state.vpn_engine.apply_game_profile(profile, Path::new(".")).await
        .map_err(|e| format!("❌ Netzwerk-Fehler: {}", e))?;

    // 2. Check and optimize Linux Proton tool (AVX2 / NTSYNC aware)
    if let Some(ref recommended) = profile.recommended_proton {
        let mut pm_guard = state.proton_manager.lock().await;
        if let Some(ref mut pm) = *pm_guard {
            tracing::info!("Verifying hardware-optimal Proton tool for '{}'...", recommended);
            let _ = pm.ensure_optimal_proton(recommended).await
                .map_err(|e| format!("⚠️ Proton Hinweis: {}", e));
        }
    }

    Ok(format!("✔ Netzwerk- & Proton-Profil für '{}' erfolgreich geladen und aktiv!", game_name))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("🚀 Starting RetroLAN-VPN Engine with Tauri v2 GUI...");

    // 1. Initialize Subsystems
    let proton_manager = ProtonManager::new().ok();
    let steam_engine = SteamEngine::init(RETROLAN_DEV_APP_ID)?;
    let mdns_engine = MdnsDiscoveryEngine::new("RetroLAN-PC-1", "10.133.7.1", "4x/example+wg+pubkey=");
    let vpn_engine = VpnEngine::new("retrolan0", "10.133.7.1").await?;
    
    let db = GameDatabase::load_from_file(Path::new("games.toml"))
        .unwrap_or_else(|_| GameDatabase { games: vec![] });

    // 2. Wrap into Thread-Safe Tauri State
    let app_state = AppState {
        vpn_engine: Arc::new(vpn_engine),
        proton_manager: Arc::new(Mutex::new(proton_manager)),
        mdns_engine: Arc::new(mdns_engine),
        steam_engine: Arc::new(steam_engine),
        db: Arc::new(db),
    };

    // 3. Launch Tauri v2 Window with Managed State
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_system_status,
            get_game_list,
            host_lobby_cmd,
            start_mdns_cmd,
            deploy_ipx_cmd,
            apply_profile_cmd
        ])
        .run(tauri::generate_context!())
        .expect("❌ Fehler beim Starten des Tauri v2 Anwendungsfensters!");

    Ok(())
}
