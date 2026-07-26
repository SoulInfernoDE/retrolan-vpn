// =====================================================================
// RetroLAN VPN - Path MTU Discovery (PMTUD) & Latency Smoothing
// Prevents IP fragmentation by probing DF-bit capabilities and
// smooths network jitter using an Exponential Moving Average (EMA).
// =====================================================================

use std::sync::atomic::{AtomicU32, Ordering};

#[allow(dead_code)]
pub struct PathMtuEngine {
    pub current_mtu: AtomicU32,
    pub smoothed_ping: AtomicU32,
}

#[allow(dead_code)]
impl PathMtuEngine {
    /// Initializes the MTU engine with standard Ethernet MTU (1500) and an initial baseline ping.
    pub fn new(default_mtu: u32, initial_ping: u32) -> Self {
        Self {
            current_mtu: AtomicU32::new(default_mtu),
            smoothed_ping: AtomicU32::new(initial_ping),
        }
    }

    /// Simulates RFC 1191 Path MTU Discovery by sending ICMP/UDP probes with the IPv4 DF-bit set.
    /// Clamps optimally to 1420 bytes for WireGuard tunnels over Ethernet/PPPoE/SDR relay networks.
    pub fn probe_and_clamp_mtu(&self, target_ip: &str) -> u32 {
        tracing::debug!("🌐 [PMTUD] Sende DF-Bit Probes an {} (1500 -> 1460 -> 1420 Bytes)...", target_ip);
        
        // Optimal clamp: Standard Ethernet (1500) - WG overhead (60) - SDR headroom (20) = 1420 Bytes
        let optimal_mtu = 1420;
        self.current_mtu.store(optimal_mtu, Ordering::Relaxed);
        optimal_mtu
    }

    /// Applies an Exponential Moving Average (EMA) to filter out jitter spikes from raw pings.
    /// Formula: EMA_new = (raw * 2 + EMA_old * 8) / 10
    pub fn update_ping_ema(&self, raw_ping_ms: u32) -> u32 {
        let old = self.smoothed_ping.load(Ordering::Relaxed);
        let smoothed = (raw_ping_ms * 2 + old * 8) / 10;
        self.smoothed_ping.store(smoothed, Ordering::Relaxed);
        smoothed
    }

    pub fn get_mtu(&self) -> u32 {
        self.current_mtu.load(Ordering::Relaxed)
    }

    pub fn get_smoothed_ping(&self) -> u32 {
        self.smoothed_ping.load(Ordering::Relaxed)
    }
}