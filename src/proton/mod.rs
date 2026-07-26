// =====================================================================
// RetroLAN VPN - Proton Compatibility & GitHub Release Downloader
// Detects AVX2/NTSYNC hardware capabilities, enforces strict CPU arch
// filtering, resolves games.toml keywords, and auto-installs missing tools.
// =====================================================================

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};

pub struct ProtonManager {
    compatibility_dirs: Vec<PathBuf>,
    pub installed_tools: Vec<String>,
}

impl ProtonManager {
    /// Initializes the Proton Manager, scans hardware diagnostics, and discovers installed tools.
    pub fn new() -> Result<Self> {
        tracing::info!("Initializing RetroLAN Linux Proton Compatibility Manager...");

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let avx2 = std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("fma");
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        let avx2 = false;

        let ntsync = Path::new("/dev/ntsync").exists();
        tracing::info!(
            "🧠 Hardware Diagnostics -> AVX2 (x86-64-v3): {} | Kernel NTSYNC: {} (/dev/ntsync active)",
            if avx2 { "✔ YES" } else { "❌ NO" },
            if ntsync { "✔ YES" } else { "❌ NO" }
        );

        let mut dirs = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let home_path = PathBuf::from(&home);
                let p1 = home_path.join(".steam/root/compatibilitytools.d");
                let p2 = home_path.join(".local/share/Steam/compatibilitytools.d");
                let p3 = home_path.join(".var/app/com.valvesoftware.Steam/data/Steam/compatibilitytools.d");

                for p in [p1, p2, p3] {
                    if !dirs.contains(&p) {
                        if !p.exists() {
                            let _ = fs::create_dir_all(&p);
                        }
                        if p.exists() {
                            dirs.push(p);
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let p1 = PathBuf::from("C:\\Program Files (x86)\\Steam\\compatibilitytools.d");
            if !p1.exists() {
                let _ = fs::create_dir_all(&p1);
            }
            if p1.exists() {
                dirs.push(p1);
            }
        }

        let mut mgr = Self {
            compatibility_dirs: dirs,
            installed_tools: Vec::new(),
        };

        mgr.rescan_tools();
        tracing::info!("✔ Found {} installed Proton compatibility tools.", mgr.installed_tools.len());

        Ok(mgr)
    }

    /// Scans all registered compatibility directories for available Proton/WINE runners.
    pub fn rescan_tools(&mut self) {
        self.installed_tools.clear();
        for dir in &self.compatibility_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if entry.path().is_dir() && !self.installed_tools.contains(&name) {
                            self.installed_tools.push(name);
                        }
                    }
                }
            }
        }
    }

    /// Automatically ensures optimal Proton is present. Downloads automatically if missing.
    pub async fn ensure_optimal_proton(&mut self, target_tool: &str) -> Result<String> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let avx2_capable = std::is_x86_feature_detected!("avx2");
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        let avx2_capable = false;

        let resolved_query = match target_tool.to_lowercase().as_str() {
            "proton-cachyos-v3-latest" => {
                return self.fetch_and_install_github_release("CachyOS/proton-cachyos", true).await;
            }
            "proton-cachyos-latest" => {
                return self.fetch_and_install_github_release("CachyOS/proton-cachyos", false).await;
            }
            "proton-ge-latest" => {
                return self.fetch_and_install_github_release("GloriousEggroll/proton-ge-custom", false).await;
            }
            _ => target_tool
        };

        if (resolved_query.eq_ignore_ascii_case("proton-cachyos") || resolved_query.to_lowercase().contains("cachyos")) && avx2_capable {
            if let Some(existing_v3) = self.installed_tools.iter().find(|t| t.to_lowercase().contains("cachyos") && (t.contains("v3") || t.contains("x86_64_v3"))) {
                return Ok(format!("✔ AVX2-optimales Proton '{}' ist bereits vorhanden.", existing_v3));
            }
            if let Ok(res) = self.fetch_and_install_github_release("CachyOS/proton-cachyos", true).await {
                return Ok(res);
            }
        }

        if self.installed_tools.iter().any(|t| t.eq_ignore_ascii_case(resolved_query)) {
            return Ok(format!("✔ Vorgeschriebenes Proton '{}' ist vorhanden.", resolved_query));
        }

        if let Some(fuzzy) = self.installed_tools.iter().find(|t| t.to_lowercase().contains(&resolved_query.to_lowercase()) || resolved_query.to_lowercase().contains(&t.to_lowercase())) {
            return Ok(format!("✔ Lokale Alternative '{}' wird verwendet.", fuzzy));
        }

        tracing::info!("⚠️ Proton '{}' fehlt. Starte automatische Installation...", resolved_query);
        if resolved_query.to_lowercase().contains("cachyos") {
            self.fetch_and_install_github_release("CachyOS/proton-cachyos", avx2_capable).await
        } else {
            self.fetch_and_install_github_release("GloriousEggroll/proton-ge-custom", false).await
        }
    }

    /// Asynchronously fetches a release from GitHub, ensuring strict CPU architecture verification and idempotency.
    pub async fn fetch_and_install_github_release(&mut self, repo: &str, require_v3: bool) -> Result<String> {
        let target_dir = self.compatibility_dirs.first()
            .cloned()
            .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".steam/root/compatibilitytools.d"));

        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        let repo_str = repo.to_string();
        let target_dir_clone = target_dir.clone();
        let installed_clone = self.installed_tools.clone();

        let (tool_name, bytes_written) = tokio::task::spawn_blocking(move || -> Result<(String, u64)> {
            let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo_str);
            let output = Command::new("curl")
                .args(["-s", &api_url])
                .output()
                .context("❌ Konnte curl nicht ausführen.")?;

            let json_str = String::from_utf8_lossy(&output.stdout);
            let (url, name, ext) = Self::parse_compatible_release(&json_str, require_v3)
                .context("❌ Kein kompatibles Release gefunden.")?;

            if installed_clone.contains(&name) || target_dir_clone.join(&name).exists() {
                return Ok((name, 0));
            }

            tracing::info!("📥 Lade Proton Tool '{}' herunter...", name);
            let tmp_archive = std::env::temp_dir().join(format!("retrolan_proton_dl.{}", ext));

            let dl_status = Command::new("curl")
                .args(["-L", "-s", "-o", tmp_archive.to_str().unwrap_or("/tmp/dl.tar"), &url])
                .status()
                .context("❌ Download fehlgeschlagen.")?;

            if !dl_status.success() {
                anyhow::bail!("❌ Download abgebrochen.");
            }

            let meta = fs::metadata(&tmp_archive)?;
            let tar_status = Command::new("tar")
                .args(["-xf", tmp_archive.to_str().unwrap_or("/tmp/dl.tar"), "-C", target_dir_clone.to_str().unwrap_or(".")])
                .status()
                .context("❌ Entpacken fehlgeschlagen.")?;

            let _ = fs::remove_file(&tmp_archive);

            if !tar_status.success() {
                anyhow::bail!("❌ Entpacken abgebrochen.");
            }

            Ok((name, meta.len()))
        }).await.context("❌ Task fehlgeschlagen")??;

        self.rescan_tools();
        if bytes_written > 0 {
            Ok(format!("✔ '{}' installiert und registriert!", tool_name))
        } else {
            Ok(format!("✔ '{}' ist bereits aktiv.", tool_name))
        }
    }

    fn parse_compatible_release(json: &str, require_v3: bool) -> Option<(String, String, String)> {
        for line in json.lines() {
            if line.contains("browser_download_url") && (line.contains(".tar.gz") || line.contains(".tar.zst") || line.contains(".tar.xz")) {
                let parts: Vec<&str> = line.split('"').collect();
                for part in parts {
                    if part.starts_with("https://") && (part.ends_with(".tar.gz") || part.ends_with(".tar.zst") || part.ends_with(".tar.xz")) {
                        let filename = part.split('/').last().unwrap_or("Proton.tar.gz");
                        let lower = filename.to_lowercase();

                        #[cfg(target_arch = "x86_64")]
                        if lower.contains("aarch64") || lower.contains("arm64") || lower.contains("armv7") {
                            continue;
                        }

                        if require_v3 && (!lower.contains("v3") && !lower.contains("x86_64_v3")) {
                            continue;
                        }

                        let ext = if lower.ends_with(".tar.zst") { "tar.zst" } else if lower.ends_with(".tar.xz") { "tar.xz" } else { "tar.gz" };
                        let tool_name = filename.replace(".tar.gz", "").replace(".tar.zst", "").replace(".tar.xz", "");
                        return Some((part.to_string(), tool_name, ext.to_string()));
                    }
                }
            }
        }
        None
    }
}
