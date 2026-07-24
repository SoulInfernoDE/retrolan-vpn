// =====================================================================
// RetroLAN VPN - Main Application Entry Point
// =====================================================================

mod network;

use network::VpnEngine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging to terminal
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 Starting RetroLAN-VPN Core Engine...");

    // Initialize our virtual gaming adapter on IP 10.133.7.1
    let mut engine = VpnEngine::new("retrolan0", "10.133.7.1").await?;

    tracing::info!("✔ RetroLAN-VPN Engine successfully initialized!");
    
    // Simulate shutdown
    engine.shutdown().await?;

    Ok(())
}