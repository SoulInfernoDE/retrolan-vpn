// =====================================================================
// RetroLAN VPN - Local mDNS Peer Discovery Engine
// Facilitates zero-config physical LAN party discovery without Steam
// or Internet access using Multicast DNS (mDNS) service beacons.
// =====================================================================

use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use anyhow::{Context, Result};

/// Standard mDNS service type for RetroLAN peer discovery.
#[allow(dead_code)]
pub const RETROLAN_MDNS_SERVICE_TYPE: &str = "_retrolan._udp.local.";

/// Default UDP port for local discovery beacons.
#[allow(dead_code)]
pub const RETROLAN_MDNS_PORT: u16 = 23757;

/// Represents a discovered peer on the physical LAN.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Instance name of the peer (e.g., "PC-Basement-1").
    pub instance_name: String,
    /// Physical LAN IP address where the peer can be reached.
    pub physical_ip: Ipv4Addr,
    /// Virtual WireGuard IP address assigned inside the RetroLAN subnet (10.133.7.x).
    pub virtual_ip: String,
    /// WireGuard public key used to establish the cryptographic user-space tunnel.
    pub wg_pub_key: String,
}

/// Manages broadcasting our local presence and discovering peers via mDNS.
#[allow(dead_code)]
pub struct MdnsDiscoveryEngine {
    /// Pure Rust mDNS daemon instance (no Bonjour/Avahi system daemon required).
    daemon: Arc<Mutex<Option<ServiceDaemon>>>,
    /// Our local instance name broadcasted to the physical LAN.
    instance_name: String,
    /// Our assigned virtual IPv4 address within the RetroLAN subnet.
    virtual_ip: String,
    /// Our local ephemeral WireGuard public key.
    wg_pub_key: String,
    /// List of actively discovered peers on the LAN.
    discovered_peers: Arc<Mutex<HashMap<String, DiscoveredPeer>>>,
}

#[allow(dead_code)]
impl MdnsDiscoveryEngine {
    /// Initializes a new mDNS discovery engine instance.
    pub fn new(instance_name: &str, virtual_ip: &str, wg_pub_key: &str) -> Self {
        tracing::info!(
            "Initializing RetroLAN mDNS Discovery Engine (Instance: '{}')...",
            instance_name
        );
        Self {
            daemon: Arc::new(Mutex::new(None)),
            instance_name: instance_name.to_string(),
            virtual_ip: virtual_ip.to_string(),
            wg_pub_key: wg_pub_key.to_string(),
            discovered_peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Starts broadcasting our local presence, WireGuard public key, and virtual IP via mDNS TXT records.
    pub async fn start_broadcasting(&self, physical_ip_str: &str) -> Result<()> {
        tracing::info!("Starting local mDNS presence broadcast on IP {}...", physical_ip_str);

        let daemon = ServiceDaemon::new()
            .context("Failed to create pure-Rust mDNS service daemon")?;

        let mut properties = HashMap::new();
        properties.insert("wg_pubkey".to_string(), self.wg_pub_key.clone());
        properties.insert("virtual_ip".to_string(), self.virtual_ip.clone());
        properties.insert("version".to_string(), "0.1.0".to_string());

        let host_name = format!("{}.local.", self.instance_name);

        // Create the mDNS ServiceInfo structure with our metadata TXT properties
        let service_info = ServiceInfo::new(
            RETROLAN_MDNS_SERVICE_TYPE,
            &self.instance_name,
            &host_name,
            physical_ip_str,
            RETROLAN_MDNS_PORT,
            Some(properties),
        ).context("Failed to construct mDNS ServiceInfo beacon")?;

        daemon.register(service_info)
            .context("Failed to register mDNS broadcast beacon on local network")?;

        *self.daemon.lock().await = Some(daemon);
        tracing::info!("✔ mDNS broadcast beacon active! Visible as '{}'", self.instance_name);

        Ok(())
    }

    /// Spawns an asynchronous background task to discover and monitor peers on the physical LAN.
    pub async fn start_discovery(&self) -> Result<()> {
        let mut daemon_guard = self.daemon.lock().await;
        let daemon = match daemon_guard.as_ref() {
            Some(d) => d.clone(),
            None => {
                let d = ServiceDaemon::new().context("Failed to initialize discovery daemon")?;
                *daemon_guard = Some(d.clone());
                d
            }
        };

        tracing::info!("Browsing physical LAN for '{}' service beacons...", RETROLAN_MDNS_SERVICE_TYPE);
        let receiver = daemon.browse(RETROLAN_MDNS_SERVICE_TYPE)
            .context("Failed to start mDNS browsing")?;

        let peers_ref = Arc::clone(&self.discovered_peers);
        let my_instance = self.instance_name.clone();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        // Ignore our own broadcasted beacon
                        if info.get_fullname().contains(&my_instance) {
                            continue;
                        }

                        let wg_pub_key = info.get_property_val_str("wg_pubkey").unwrap_or("").to_string();
                        let virtual_ip = info.get_property_val_str("virtual_ip").unwrap_or("").to_string();

                        if let Some(ip_addr) = info.get_addresses().iter().next() {
                            if let Ok(physical_ip) = Ipv4Addr::from_str(&ip_addr.to_string()) {
                                tracing::info!(
                                    "🌐 mDNS Discovered LAN Peer: '{}' (Physical: {}, Virtual: {})",
                                    info.get_fullname(), physical_ip, virtual_ip
                                );

                                let peer = DiscoveredPeer {
                                    instance_name: info.get_fullname().to_string(),
                                    physical_ip,
                                    virtual_ip,
                                    wg_pub_key,
                                };

                                peers_ref.lock().await.insert(peer.instance_name.clone(), peer);
                            }
                        }
                    }
                    ServiceEvent::ServiceRemoved(service_type, fullname) => {
                        tracing::info!("mDNS Peer left the LAN: '{}' ({})", fullname, service_type);
                        peers_ref.lock().await.remove(&fullname);
                    }
                    _ => {}
                }
            }
        });

        tracing::info!("✔ mDNS Peer Discovery listener actively monitoring LAN!");
        Ok(())
    }

    /// Retrieves a cloned list of all currently discovered physical LAN peers.
    pub async fn get_discovered_peers(&self) -> Vec<DiscoveredPeer> {
        let peers = self.discovered_peers.lock().await;
        peers.values().cloned().collect()
    }

    /// Unregisters local service beacons and shuts down the mDNS daemon.
    pub async fn shutdown(&self) -> Result<()> {
        let mut daemon_guard = self.daemon.lock().await;
        if let Some(daemon) = daemon_guard.take() {
            tracing::info!("Shutting down local mDNS discovery daemon...");
            let _ = daemon.shutdown();
        }
        tracing::info!("✔ mDNS Discovery Engine stopped.");
        Ok(())
    }
}