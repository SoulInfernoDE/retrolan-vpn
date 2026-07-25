// =====================================================================
// RetroLAN VPN - Linux Proton Compatibility Tool Manager
// Scans Linux Steam directories for installed custom Proton builds
// (GE-Proton, Proton-CachyOS) and fetches missing releases on demand.
// =====================================================================

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use serde::Deserialize;

/// Standard GitHub API endpoint for checking the latest GE-Proton releases.
pub const GE_PROTON_GITHUB_API: &str = "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases/latest";

/// Represents an installed compatibility tool found in the Linux Steam directory.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProtonTool {
    /// Directory folder name of the tool (e.g., "GE-Proton9-12").
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
}

#[allow(dead_code)]
impl ProtonManager {
    /// Initializes the Proton Manager and locates the system's Steam compatibility directory.
    pub fn new() -> Result<Self> {
        tracing::info!("Initializing RetroLAN Linux Proton Compatibility Manager...");
        
        let compatibility_dir = Self::locate_compatibility_dir()
            .context("Could not determine Steam compatibility directory on this Linux system")?;

        if !compatibility_dir.exists() {
            tracing::info!("Creating Steam compatibility directory at {:?}", compatibility_dir);
            fs::create_dir_all(&compatibility_dir)?;
        }

        let mut manager = Self {
            compatibility_dir,
            installed_tools: Vec::new(),
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

        // Default fallback to standard Unix home directory structure
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

    /// Checks if a required Proton version is installed. If missing, automatically downloads it.
    pub async fn ensure_proton_installed(&mut self, required_tool: &str) -> Result<PathBuf> {
        if let Some(tool) = self.installed_tools.iter().find(|t| t.name.to_lowercase().contains(&required_tool.to_lowercase())) {
            tracing::info!("✔ Recommended tool '{}' is already installed at {:?}", tool.name, tool.path);
            return Ok(tool.path.clone());
        }

        tracing::warn!("⚠️ Recommended Proton build '{}' is not installed!", required_tool);

        if required_tool.to_lowercase().contains("ge-proton") || required_tool.to_lowercase().contains("proton-ge") {
            tracing::info!("⬇️ Initiating automatic download for latest GE-Proton release...");
            return self.download_latest_ge_proton().await;
        }

        anyhow::bail!("Automatic downloading for '{}' is not yet supported. Please install it manually via ProtonUp-Qt.", required_tool);
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
