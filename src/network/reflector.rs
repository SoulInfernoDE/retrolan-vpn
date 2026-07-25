// =====================================================================
// RetroLAN VPN - Layer-2 UDP Broadcast Reflector
// Intercepts LAN discovery broadcasts and forwards them as unicast
// packets to all active WireGuard peers in the virtual subnet.
// =====================================================================

use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use anyhow::{Context, Result};

/// Manages UDP broadcast interception and replication across WireGuard peers.
#[allow(dead_code)]
pub struct BroadcastReflector {
    /// Active virtual IPv4 addresses of peers in the current gaming lobby.
    peers: Arc<Mutex<HashSet<Ipv4Addr>>>,
    /// List of UDP ports currently being monitored for LAN game broadcasts.
    monitored_ports: Arc<Mutex<HashSet<u16>>>,
    /// Flag indicating if the background reflector tasks are running.
    is_active: Arc<Mutex<bool>>,
}

#[allow(dead_code)]
impl BroadcastReflector {
    /// Creates a new, inactive UDP broadcast reflector.
    pub fn new() -> Self {
        tracing::info!("Initializing Layer-2 UDP Broadcast Reflector module...");
        Self {
            peers: Arc::new(Mutex::new(HashSet::new())),
            monitored_ports: Arc::new(Mutex::new(HashSet::new())),
            is_active: Arc::new(Mutex::new(false)),
        }
    }

    /// Registers a new peer IP address to receive broadcast reflections.
    pub async fn add_peer(&self, peer_ip: Ipv4Addr) {
        let mut peers = self.peers.lock().await;
        if peers.insert(peer_ip) {
            tracing::info!("Reflector: Added peer {} to broadcast replication list", peer_ip);
        }
    }

    /// Removes a peer IP address when they leave the lobby.
    pub async fn remove_peer(&self, peer_ip: &Ipv4Addr) {
        let mut peers = self.peers.lock().await;
        if peers.remove(peer_ip) {
            tracing::info!("Reflector: Removed peer {} from replication list", peer_ip);
        }
    }

    /// Starts monitoring a specific UDP port for LAN game discovery packets.
    pub async fn start_monitoring_port(&self, port: u16, bind_ip: Ipv4Addr) -> Result<()> {
        let mut ports = self.monitored_ports.lock().await;
        if !ports.insert(port) {
            tracing::debug!("Port {} is already being monitored by reflector", port);
            return Ok(());
        }

        let bind_addr = SocketAddr::new(bind_ip.into(), port);
        tracing::info!("Reflector: Spawning asynchronous broadcast listener on {}", bind_addr);

        // Bind UDP socket with broadcast capabilities enabled
        let socket = UdpSocket::bind(bind_addr)
            .await
            .with_context(|| format!("Failed to bind broadcast reflector to port {}", port))?;
        
        socket.set_broadcast(true)
            .context("Failed to enable SO_BROADCAST on reflector socket")?;

        let socket = Arc::new(socket);
        let peers_ref = Arc::clone(&self.peers);
        let is_active_ref = Arc::clone(&self.is_active);

        // Spawn a dedicated Tokio background task for this game port
        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];
            
            loop {
                if !*is_active_ref.lock().await {
                    tracing::debug!("Reflector loop terminated for port {}", port);
                    break;
                }

                match socket.recv_from(&mut buffer).await {
                    Ok((bytes_read, source_addr)) => {
                        // Prevent infinite loops by ignoring packets sent by ourselves
                        if source_addr.ip() == bind_ip {
                            continue;
                        }

                        tracing::debug!(
                            "Reflector: Intercepted {} broadcast bytes on port {} from {}",
                            bytes_read, port, source_addr
                        );

                        let peers = peers_ref.lock().await;
                        for peer_ip in peers.iter() {
                            let target_addr = SocketAddr::new((*peer_ip).into(), port);
                            if let Err(err) = socket.send_to(&buffer[..bytes_read], target_addr).await {
                                tracing::warn!(
                                    "Reflector: Failed to forward broadcast to peer {}: {}",
                                    target_addr, err
                                );
                            }
                        }
                    }
                    Err(err) => {
                        tracing::error!("Reflector socket error on port {}: {}", port, err);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Activates the reflector engine.
    pub async fn start(&self) {
        let mut active = self.is_active.lock().await;
        *active = true;
        tracing::info!("✔ Layer-2 UDP Broadcast Reflector engine activated!");
    }

    /// Gracefully stops all monitoring tasks.
    pub async fn stop(&self) {
        let mut active = self.is_active.lock().await;
        *active = false;
        tracing::info!("Layer-2 UDP Broadcast Reflector engine stopped.");
    }
}