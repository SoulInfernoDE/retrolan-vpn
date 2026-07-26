// =====================================================================
// RetroLAN VPN - Steamworks & Valve SDR Relay Integration
// Communicates with the Steam Client SDK (v0.13+) to host signaling
// lobbies and route VPN traffic over Valve's global SDR network when
// direct IPv6/IPv4 P2P connectivity fails due to CGNAT or DS-Lite.
// =====================================================================

pub mod locator;

use std::sync::Arc;
use steamworks::{Client, LobbyId, LobbyType};
use tokio::sync::Mutex;
use anyhow::{Context, Result};

/// Valve assigned AppID for RetroLAN development testing (Spacewar / SDK fallback).
#[allow(dead_code)]
pub const RETROLAN_DEV_APP_ID: u32 = 480;

/// Manages Steamworks SDK callbacks, P2P networking, and signaling lobbies.
#[allow(dead_code)]
pub struct SteamEngine {
    /// Native Steamworks client handle.
    client: Arc<Client>,
    /// Currently connected or hosted Steam signaling lobby ID.
    current_lobby: Arc<Mutex<Option<LobbyId>>>,
}

#[allow(dead_code)]
impl SteamEngine {
    /// Initializes the Steamworks SDK. Returns None if Steam is not running or offline.
    pub fn init(app_id: u32) -> Result<Option<Self>> {
        tracing::info!("Initializing Steamworks SDK integration (AppID: {})...", app_id);

        let client = match Client::init_app(app_id) {
            Ok(res) => res,
            Err(err) => {
                tracing::warn!(
                    "⚠️ Steam Client not detected or offline. RetroLAN will operate in physical Offline LAN Mode. ({})",
                    err
                );
                return Ok(None);
            }
        };

        let client = Arc::new(client);
        let cb_client = Arc::clone(&client);

        std::thread::spawn(move || {
            loop {
                cb_client.run_callbacks();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        });

        tracing::info!("✔ Steamworks SDK successfully initialized!");
        Ok(Some(Self {
            client,
            current_lobby: Arc::new(Mutex::new(None)),
        }))
    }

    /// Asynchronously creates a Steam signaling lobby for peer discovery.
    pub async fn create_signaling_lobby(&self, max_members: u32) -> Result<LobbyId> {
        tracing::info!("Creating Steam signaling lobby for {} players...", max_members);

        let (tx, rx) = tokio::sync::oneshot::channel();

        {
            let matchmaking = self.client.matchmaking();
            matchmaking.create_lobby(LobbyType::FriendsOnly, max_members, move |result| {
                let _ = tx.send(result);
            });
        }

        let lobby_id = rx.await
            .context("Steam lobby creation callback channel closed unexpectedly")?
            .context("Steamworks SDK refused lobby creation request")?;

        *self.current_lobby.lock().await = Some(lobby_id);
        tracing::info!("✔ Steam Signaling Lobby created! ID: {:?}", lobby_id);

        Ok(lobby_id)
    }

    /// Broadcasts our ephemeral WireGuard public key and virtual LAN IP to the active lobby.
    pub async fn broadcast_wireguard_handshake(&self, wg_pub_key: &str, virtual_ip: &str) -> Result<()> {
        let lobby_guard = self.current_lobby.lock().await;
        let lobby_id = match *lobby_guard {
            Some(id) => id,
            None => anyhow::bail!("Cannot broadcast WireGuard handshake without an active Steam lobby"),
        };

        tracing::info!(
            "Broadcasting WireGuard credentials to Steam Lobby {:?} (IP: {}, Key: {})...",
            lobby_id, virtual_ip, wg_pub_key
        );

        let matchmaking = self.client.matchmaking();
        matchmaking.set_lobby_data(lobby_id, "retrolan_wg_pubkey", wg_pub_key);
        matchmaking.set_lobby_data(lobby_id, "retrolan_virtual_ip", virtual_ip);
        matchmaking.set_lobby_data(lobby_id, "retrolan_version", "0.1.0");

        tracing::info!("✔ WireGuard handshake metadata successfully published to Steam Lobby!");
        Ok(())
    }

    /// Gracefully leaves any active signaling lobby and shuts down the Steam integration.
    pub async fn shutdown(&self) {
        let mut lobby_guard = self.current_lobby.lock().await;
        if let Some(lobby_id) = lobby_guard.take() {
            tracing::info!("Leaving Steam signaling lobby {:?}...", lobby_id);
            let matchmaking = self.client.matchmaking();
            matchmaking.leave_lobby(lobby_id);
        }
        tracing::info!("Steam Engine disconnected.");
    }
}
