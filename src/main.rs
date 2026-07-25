mod config;
mod network;

use network::VpnEngine;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("🚀 Starting RetroLAN-VPN Core Engine...");

    let mut engine = VpnEngine::new("retrolan0", "10.133.7.1").await?;

    // Test: Load database and simulate applying a profile
    if let Ok(db) = config::GameDatabase::load_from_file(Path::new("games.toml")) {
        if let Some(flatout_profile) = db.find_by_process("FlatOut2.exe") {
            engine.apply_game_profile(flatout_profile, Path::new(".")).await?;
        }
    }

    tracing::info!("✔ RetroLAN-VPN Engine successfully initialized!");
    engine.shutdown().await?;
    Ok(())
}
