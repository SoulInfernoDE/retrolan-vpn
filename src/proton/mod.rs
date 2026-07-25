// =====================================================================
// RetroLAN VPN - Linux Proton Compatibility Tool Manager
// Scans Linux Steam directories for installed custom Proton builds,
// detects CPU AVX2 (x86-64-v3) & kernel NTSYNC support, and dynamically
// chooses between Proton-CachyOS v3 and GE-Proton.
// =====================================================================

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use serde::Deserialize;

/// Standard GitHub API endpoint for checking the latest GE-Proton releases.
pub const GE_PROTON_GITHUB_API: &str = "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases/latest";

/// Represents an installed compatibility tool found in the Linux Steam directory.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProtonTool {
    /// Directory folder name of the tool (e.g., "GE-Proton9-12" or "proton-cachyos-v3").
    pub name: String,
    /// Absolute filesystem path to the compatibility tool installation directory.
    pub path: PathBuf,
}

/// Helper struct for deserializing GitHub Release JSON responses.
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Manages scanning, verifying, and downloading Proton compatibility tools on Linux.
#[allow(dead_code)]
pub struct ProtonManager {
    /// Target Steam compatibilitytools.d directory path.
    pub compatibility_dir: PathBuf,
    /// List of currently detected compatibility tools on the local system.
    pub installed_tools: Vec<ProtonTool>,
    /// Flag indicating whether the local CPU supports AVX2 & FMA (x86-64-v3 architecture).
    pub cpu_supports_avx2: bool,
    /// Flag indicating whether the Linux kernel has the /dev/ntsync module loaded.
    pub kernel_supports_ntsync: bool,
}

#[allow(dead_code)]
impl ProtonManager {
    /// Initializes the Proton Manager, scans local hardware features, and locates Steam folders.
    pub fn new() -> Result<Self> {
        tracing::info!("Initializing RetroLAN Linux Proton Compatibility Manager...");
        
        let compatibility_dir = Self::locate_compatibility_dir()
            .context("Could not determine Steam compatibility directory on this Linux system")?;

        if !compatibility_dir.exists() {
            tracing::info!("Creating Steam compatibility directory at {:?}", compatibility_dir);
            fs::create_dir_all(&compatibility_dir)?;
        }

        // 1. Detect x86-64-v3 CPU capabilities (AVX2 + FMA)
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let cpu_supports_avx2 = std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("fma");
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        let cpu_supports_avx2 = false;

        // 2. Detect native Linux kernel NTSYNC support (/dev/ntsync)
        let kernel_supports_ntsync = Path::new("/dev/ntsync").exists();

        tracing::info!(
            "🧠 Hardware Diagnostics -> AVX2 (x86-64-v3): {} | Kernel NTSYNC: {}",
            if cpu_supports_avx2 { "✔ YES" } else { "❌ NO" },
            if kernel_supports_ntsync { "✔ YES (/dev/ntsync active)" } else { "❌ NO (Fallback to esync/fsync)" }
        );

        let mut manager = Self {
            compatibility_dir,
            installed_tools: Vec::new(),
            cpu_supports_avx2,
            kernel_supports_ntsync,
        };

        manager.scan_installed_tools()?;
        Ok(manager)
    }

    /// Locates the standard Linux Steam `compatibilitytools.d` directory.
    /// Checks native packages, Flatpak installations, and environment overrides.
    fn locate_compatibility_dir() -> Option<PathBuf> {
        if let Ok(custom_path) = std::env::var("STEAM_COMPAT_DIR") {
            return Some(PathBuf::from(custom_path));
        }

        if let Some(home) = dirs::home_dir() {
            // 1. Check native Steam installation path
            let native_path = home.join(".steam/root/compatibilitytools.d");
            if native_path.exists() || home.join(".steam/root").exists() {
                return Some(native_path);
            }

            // 2. Check alternative .local/share/Steam path
            let local_share = home.join(".local/share/Steam/compatibilitytools.d");
            if local_share.exists() || home.join(".local/share/Steam").exists() {
                return Some(local_share);
            }

            // 3. Check Steam Flatpak sandbox path
            let flatpak_path = home.join(".var/app/com.valvesoftware.Steam/data/Steam/compatibilitytools.d");
            if flatpak_path.exists() {
                return Some(flatpak_path);
            }
        }

        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".steam/root/compatibilitytools.d"))
    }

    /// Scans the compatibility directory and populates the internal list of available tools.
    pub fn scan_installed_tools(&mut self) -> Result<()> {
        self.installed_tools.clear();
        
        tracing::debug!("Scanning for installed Proton builds in {:?}", self.compatibility_dir);

        let entries = fs::read_dir(&self.compatibility_dir)
            .with_context(|| format!("Failed to read directory {:?}", self.compatibility_dir))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    tracing::debug!("Detected compatibility tool: '{}'", name);
                    self.installed_tools.push(ProtonTool {
                        name: name.to_string(),
                        path,
                    });
                }
            }
        }

        tracing::info!("✔ Found {} installed Proton compatibility tools.", self.installed_tools.len());
        Ok(())
    }

    /// Determines the optimal Proton build based on local CPU & kernel features,
    /// checks if it is installed, and initiates automated downloads if necessary.
    pub async fn ensure_optimal_proton(&mut self, recommended_tool: &str) -> Result<PathBuf> {
        let mut target_tool = recommended_tool.to_string();

        // Dynamic optimization: If the profile requests CachyOS v3 or AVX, verify hardware support!
        if target_tool.to_lowercase().contains("cachyos") {
            if self.cpu_supports_avx2 {
                tracing::info!("🚀 AVX2 hardware detected! Prioritizing 'proton-cachyos-v3' for maximum performance.");
                target_tool = "proton-cachyos-v3".to_string();
            } else {
                tracing::warn!("⚠️ CPU does not support AVX2/x86-64-v3. Downgrading recommendation to secondary fallback: 'GE-Proton'");
                target_tool = "GE-Proton".to_string();
            }
        }

        // 1. Check if the determined optimal tool is already present (fuzzy matching)
        if let Some(tool) = self.installed_tools.iter().find(|t| t.name.to_lowercase().contains(&target_tool.to_lowercase())) {
            tracing::info!("✔ Optimal tool '{}' is already installed at {:?}", tool.name, tool.path);
            return Ok(tool.path.clone());
        }

        // 2. Try falling back to ANY installed GE-Proton or CachyOS if exact match is missing
        if let Some(fallback_tool) = self.installed_tools.iter().find(|t| {
            t.name.to_lowercase().contains("ge-proton") || t.name.to_lowercase().contains("cachyos")
        }) {
            tracing::info!("💡 Target tool '{}' missing, but found suitable alternative: '{}'", target_tool, fallback_tool.name);
            return Ok(fallback_tool.path.clone());
        }

        tracing::warn!("⚠️ No optimal Proton compatibility tool ('{}') found on system!", target_tool);

        // 3. Initiate automatic GitHub download for GE-Proton as universal fallback
        if target_tool.to_lowercase().contains("ge-proton") || target_tool.to_lowercase().contains("proton-ge") {
            tracing::info!("⬇️ Initiating automatic download for latest GE-Proton release...");
            return self.download_latest_ge_proton().await;
        }

        anyhow::bail!(
            "Please install '{}' (or any GE-Proton release) via ProtonUp-Qt or your package manager.",
            target_tool
        );
    }

    /// Fetches the latest GE-Proton release from GitHub and extracts it into `compatibilitytools.d`.
    async fn download_latest_ge_proton(&mut self) -> Result<PathBuf> {
        let client = reqwest::Client::builder()
            .user_agent("RetroLAN-VPN-ProtonManager/0.1.0")
            .build()?;

        tracing::info!("Querying GitHub API for latest GE-Proton release tag...");
        let response = client.get(GE_PROTON_GITHUB_API).send().await?
            .json::<GitHubRelease>().await
            .context("Failed to parse GitHub API release metadata")?;

        tracing::info!("Found latest GE-Proton release: {}", response.tag_name);

        let asset = response.assets.iter()
            .find(|a| a.name.ends_with(".tar.gz"))
            .context("No valid .tar.gz archive found in GitHub release assets")?;

        let temp_archive_path = std::env::temp_dir().join(&asset.name);
        tracing::info!("Downloading {} from {}...", asset.name, asset.browser_download_url);

        let archive_bytes = client.get(&asset.browser_download_url).send().await?
            .bytes().await
            .context("Failed to download archive bytes")?;

        fs::write(&temp_archive_path, &archive_bytes)
            .with_context(|| format!("Failed to write temporary archive to {:?}", temp_archive_path))?;

        tracing::info!("Unpacking archive into {:?}...", self.compatibility_dir);

        let status = Command::new("tar")
            .arg("-xzf")
            .arg(&temp_archive_path)
            .arg("-C")
            .arg(&self.compatibility_dir)
            .status()
            .context("Failed to execute native 'tar' command for archive extraction")?;

        if !status.success() {
            anyhow::bail!("Tar extraction failed with exit code: {:?}", status.code());
        }

        let _ = fs::remove_file(&temp_archive_path);

        tracing::info!("✔ Successfully installed {} into Steam!", response.tag_name);
        
        self.scan_installed_tools()?;
        
        let new_tool_path = self.compatibility_dir.join(&response.tag_name);
        Ok(new_tool_path)
    }
}
