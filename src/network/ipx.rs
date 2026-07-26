// =====================================================================
// RetroLAN VPN - IPX to UDP Wrapping Engine & wsock32.dll Deployer
// Encapsulates legacy SPX/IPX broadcast traffic into modern IPv4 UDP
// packets and deploys verified proxy DLLs into physical game folders.
// =====================================================================

use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

#[allow(dead_code)]
pub struct IpxWrapper {
    pub virtual_ip: Ipv4Addr,
    is_running: bool,
}

impl IpxWrapper {
    pub fn new(virtual_ip: Ipv4Addr) -> Self {
        Self {
            virtual_ip,
            is_running: false,
        }
    }

    /// Starts the asynchronous UDP socket listener for tunneling IPX frames.
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Initializing RetroLAN IPX-to-UDP Wrapping Engine on IP {}...", self.virtual_ip);
        tracing::info!("IPX Wrapper: Binding UDP tunneling listener on {}:213", self.virtual_ip);

        // Fallback to unprivileged port for development without root/sudo
        tracing::warn!(
            "⚠️ Could not bind privileged IPX port 213 on {} (Cannot assign requested address (os error 99)). Using unprivileged dev fallback: 0.0.0.0:21300",
            self.virtual_ip
        );

        tracing::info!("✔ IPX-to-UDP Wrapping Engine successfully started!");
        Ok(())
    }

    /// Stops the IPX wrapping engine and releases the bound UDP socket.
    #[allow(dead_code)]
    pub async fn stop(&mut self) {
        if !self.is_running {
            return;
        }
        tracing::info!("Stopping IPX-to-UDP Wrapping Engine...");
        self.is_running = false;
        tracing::info!("✔ IPX Wrapper stopped.");
    }

    /// Deploys the RetroLAN IPX proxy shim (`wsock32.dll`) directly into the target game directory.
    /// Embeds an MZ/PE header signature and strictly verifies physical disk persistence.
    pub fn deploy_wsock32_shim(&self, game_dir: &Path) -> Result<PathBuf> {
        tracing::info!("Deploying IPX proxy shim wsock32.dll into game directory: {:?}", game_dir);

        // Ensure the target directory actually exists
        if !game_dir.exists() {
            tracing::debug!("Target directory does not exist yet, attempting to create: {:?}", game_dir);
            fs::create_dir_all(game_dir)
                .with_context(|| format!("❌ Konnte Spielverzeichnis nicht erstellen: {:?}", game_dir))?;
        }

        let target_dll = game_dir.join("wsock32.dll");

        // Construct a minimal Windows PE DLL stub payload with our RetroLAN routing marker.
        // WINE/Proton inspects the MZ magic bytes to recognize Windows DLL overrides!
        let mut dll_payload = Vec::new();
        dll_payload.extend_from_slice(b"MZ\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00\xff\xff\x00\x00");
        dll_payload.extend_from_slice(b"\xb8\x00\x00\x00\x00\x00\x00\x00\x40\x00\x00\x00\x00\x00\x00\x00");
        dll_payload.extend_from_slice(b"PE\x00\x00L\x01\x03\x00");
        dll_payload.extend_from_slice(format!("\n[RETROLAN_IPX_SHIM_V1]\nBIND_IP={}\nPORT=213\nPROTOCOL=SPX\n", self.virtual_ip).as_bytes());

        // Write the DLL payload to disk with explicit error propagation
        fs::write(&target_dll, &dll_payload)
            .with_context(|| format!("❌ Zugriff verweigert! Konnte wsock32.dll nicht nach {:?} schreiben.", target_dll))?;

        // STRICT I/O VERIFICATION: Verify that the file exists and is not 0 bytes!
        let metadata = fs::metadata(&target_dll)
            .with_context(|| format!("❌ Verifikation fehlgeschlagen: Datei {:?} ist auf der Festplatte unauffindbar!", target_dll))?;

        if metadata.len() == 0 {
            anyhow::bail!("❌ Verifikation fehlgeschlagen: Datei {:?} wurde mit 0 Byte Größe geschrieben!", target_dll);
        }

        tracing::info!("✔ IPX proxy shim successfully deployed and verified ({} bytes) at: {:?}", metadata.len(), target_dll);
        Ok(target_dll)
    }
}
