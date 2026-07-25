// =====================================================================
// RetroLAN VPN - Configuration & Game Database Parser
// Reads games.toml and maps process names or Steam App IDs to specific
// networking requirements (IPX wrapping, UDP broadcast ports, Proton).
// =====================================================================

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

/// Represents a single game profile entry defined in games.toml.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProfile {
    /// Official display name of the game.
    pub name: String,
    /// Steam Application ID (if applicable, 0 or None for non-Steam/GOG games).
    pub steam_appid: Option<u32>,
    /// List of executable process names associated with this game (e.g., ["FlatOut2.exe"]).
    pub process_names: Vec<String>,
    /// Networking protocol required: "udp_broadcast", "ipx", or "directplay".
    pub protocol: String,
    /// UDP port used for LAN lobby discovery broadcasts (required if protocol == "udp_broadcast").
    pub broadcast_port: Option<u16>,
    /// Whether this game requires deploying our custom wsock32.dll IPX proxy shim.
    pub require_wsock32_hook: Option<bool>,
    /// Whether Wine/Proton should be forced to bind exclusively to our virtual TUN adapter.
    pub force_bind_ip: Option<bool>,
    /// Recommended Proton compatibility tool version (e.g., "Proton-CachyOS" or "Proton-GE").
    pub recommended_proton: Option<String>,
    /// Additional developer or community notes for this profile.
    pub notes: Option<String>,
}

/// Root structure representing the entire games.toml database.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDatabase {
    pub games: Vec<GameProfile>,
}

#[allow(dead_code)]
impl GameDatabase {
    /// Loads and parses the games.toml configuration file from the specified filesystem path.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        tracing::info!("Loading RetroLAN community game database from {:?}", path);
        
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read game database file at {:?}", path))?;
        
        let db: GameDatabase = toml::from_str(&content)
            .context("Failed to parse games.toml syntax")?;
        
        tracing::info!("✔ Successfully loaded {} game profiles from database.", db.games.len());
        Ok(db)
    }

    /// Searches the database for a matching game profile by its executable process name.
    /// Case-insensitive comparison ensures matching across different OS environments.
    pub fn find_by_process(&self, process_name: &str) -> Option<&GameProfile> {
        let target = process_name.to_lowercase();
        self.games.iter().find(|profile| {
            profile.process_names.iter().any(|p| p.to_lowercase() == target)
        })
    }

    /// Searches the database for a matching game profile by its official Steam App ID.
    pub fn find_by_appid(&self, appid: u32) -> Option<&GameProfile> {
        if appid == 0 {
            return None;
        }
        self.games.iter().find(|profile| profile.steam_appid == Some(appid))
    }
}