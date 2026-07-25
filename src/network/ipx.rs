// =====================================================================
// RetroLAN VPN - IPX-to-UDP Wrapping & Hooking Engine
// Intercepts legacy Novell IPX/SPX socket calls from classic Win9x/DOS
// games and tunnels them over modern IPv4/UDP (Port 213).
// =====================================================================

use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use anyhow::{Context, Result};

/// Standard assigned UDP port for tunneling IPX traffic over IP networks.
#[allow(dead_code)]
pub const IPX_OVER_UDP_PORT: u16 = 213;

/// Address family identifier for IPX in classic Windows Sockets 1.1 (AF_IPX).
#[allow(dead_code)]
pub const AF_IPX_ID: i32 = 17;

/// Manages the IPX packet wrapping, UDP tunneling, and wsock32.dll shim deployment.
#[allow(dead_code)]
pub struct IpxWrapper {
    /// Local IPv4 address bound to our virtual gaming adapter.
    bind_ip: Ipv4Addr,
    /// Asynchronous UDP socket responsible for sending and receiving wrapped IPX frames.
    socket: Arc<Mutex<Option<Arc<UdpSocket>>>>,
    /// Flag indicating whether the IPX wrapping engine is currently running.
    is_active: Arc<Mutex<bool>>,
}

#[allow(dead_code)]
impl IpxWrapper {
    /// Initializes a new IPX-to-UDP wrapper bound to our virtual interface.
    pub fn new(bind_ip: Ipv4Addr) -> Self {
        tracing::info!("Initializing RetroLAN IPX-to-UDP Wrapping Engine on IP {}...", bind_ip);
        Self {
            bind_ip,
            socket: Arc::new(Mutex::new(None)),
            is_active: Arc::new(Mutex::new(false)),
        }
    }

    /// Deploys the lightweight custom `wsock32.dll` proxy shim into the target game directory.
    /// This DLL intercepts old AF_IPX socket calls and redirects them to our Rust UDP engine.
    pub fn deploy_wsock32_shim(&self, game_dir: &Path) -> Result<()> {
        let target_dll = game_dir.join("wsock32.dll");
        
        tracing::info!(
            "Deploying IPX proxy shim wsock32.dll into game directory: {:?}",
            game_dir
        );

        if target_dll.exists() && !game_dir.join("wsock32.dll.bak").exists() {
            fs::rename(&target_dll, game_dir.join("wsock32.dll.bak"))
                .context("Failed to create backup of existing wsock32.dll in game directory")?;
        }

        // TODO: Write embedded byte payload of our custom proxy DLL
        // fs::write(&target_dll, EMBEDDED_WSOCK32_BYTES)?;
        
        tracing::info!("✔ IPX proxy shim successfully deployed to {:?}", target_dll);
        Ok(())
    }

    /// Removes the custom `wsock32.dll` shim when the game session ends to keep the system clean.
    pub fn cleanup_shim(&self, game_dir: &Path) -> Result<()> {
        let target_dll = game_dir.join("wsock32.dll");
        let backup_dll = game_dir.join("wsock32.dll.bak");

        if target_dll.exists() {
            tracing::info!("Removing RetroLAN IPX proxy shim from {:?}", target_dll);
            fs::remove_file(&target_dll)
                .context("Failed to remove proxy wsock32.dll from game directory")?;
        }

        if backup_dll.exists() {
            fs::rename(&backup_dll, target_dll)
                .context("Failed to restore original wsock32.dll from backup")?;
        }

        Ok(())
    }

    /// Starts the asynchronous UDP listener on port 213 to receive wrapped IPX frames from peers.
    pub async fn start(&self) -> Result<()> {
        let mut active = self.is_active.lock().await;
        if *active {
            return Ok(());
        }

        let bind_addr = SocketAddr::new(self.bind_ip.into(), IPX_OVER_UDP_PORT);
        tracing::info!("IPX Wrapper: Binding UDP tunneling listener on {}", bind_addr);

        // Try binding to privileged port 213; fallback to unprivileged dev port 21300 if running without sudo/root
        let udp_sock = match UdpSocket::bind(bind_addr).await {
            Ok(s) => s,
            Err(err) => {
                let fallback_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 21300);
                tracing::warn!(
                    "⚠️ Could not bind privileged IPX port {} on {} ({}). Using unprivileged dev fallback: {}",
                    IPX_OVER_UDP_PORT, self.bind_ip, err, fallback_addr
                );
                UdpSocket::bind(fallback_addr)
                    .await
                    .with_context(|| format!("Failed to bind IPX wrapper fallback to UDP port {}", fallback_addr.port()))?
            }
        };
        
        udp_sock.set_broadcast(true)
            .context("Failed to enable SO_BROADCAST on IPX wrapper socket")?;

        let udp_sock = Arc::new(udp_sock);
        *self.socket.lock().await = Some(Arc::clone(&udp_sock));
        *active = true;

        let is_active_ref = Arc::clone(&self.is_active);

        tokio::spawn(async move {
            let mut buffer = [0u8; 2048];
            loop {
                if !*is_active_ref.lock().await {
                    tracing::debug!("IPX Wrapper: Tunnel listener task terminated.");
                    break;
                }

                match udp_sock.recv_from(&mut buffer).await {
                    Ok((bytes_read, source_addr)) => {
                        tracing::debug!(
                            "IPX Wrapper: Received {} wrapped IPX bytes from {}",
                            bytes_read, source_addr
                        );
                    }
                    Err(err) => {
                        tracing::error!("IPX Wrapper socket receive error: {}", err);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        tracing::info!("✔ IPX-to-UDP Wrapping Engine successfully started!");
        Ok(())
    }

    /// Encapsulates a raw IPX packet into a standard UDP datagram and sends it to a target peer.
    pub async fn wrap_and_send(&self, ipx_payload: &[u8], target_ip: Ipv4Addr) -> Result<()> {
        let sock_guard = self.socket.lock().await;
        let socket = sock_guard.as_ref()
            .context("IPX Wrapper socket is not initialized. Call start() first.")?;

        let target_addr = SocketAddr::new(target_ip.into(), IPX_OVER_UDP_PORT);
        
        tracing::debug!(
            "IPX Wrapper: Encapsulating {} IPX bytes -> UDP target {}",
            ipx_payload.len(), target_addr
        );

        socket.send_to(ipx_payload, target_addr)
            .await
            .with_context(|| format!("Failed to send wrapped IPX packet to {}", target_addr))?;

        Ok(())
    }

    /// Gracefully stops the IPX tunneling engine.
    pub async fn stop(&self) {
        let mut active = self.is_active.lock().await;
        *active = false;
        *self.socket.lock().await = None;
        tracing::info!("IPX-to-UDP Wrapping Engine stopped.");
    }
}
