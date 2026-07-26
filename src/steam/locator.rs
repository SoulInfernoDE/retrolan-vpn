// =====================================================================
// RetroLAN VPN - Steam Game Installation Locator
// Parses Steam's libraryfolders.vdf and appmanifest_<appid>.acf files
// to dynamically locate physical game directories across all drives.
// =====================================================================

use std::path::{Path, PathBuf};
use std::fs;

#[allow(dead_code)]
pub struct SteamGameLocator;

#[allow(dead_code)]
impl SteamGameLocator {
    /// Attempts to locate the physical installation directory of a Steam game by its AppID.
    /// Returns None if Steam is not installed or if the specific AppID is not found.
    pub fn find_game_dir(app_id: u32) -> Option<PathBuf> {
        tracing::debug!("🔍 [Steam-Locator] Suche Installationspfad für AppID {}...", app_id);

        let library_roots = Self::get_all_library_paths();
        if library_roots.is_empty() {
            tracing::warn!("⚠️ [Steam-Locator] Keine Steam-Bibliotheken im System gefunden.");
            return None;
        }

        let manifest_filename = format!("appmanifest_{}.acf", app_id);

        for root in &library_roots {
            let steamapps_dir = root.join("steamapps");
            let manifest_path = steamapps_dir.join(&manifest_filename);

            if manifest_path.exists() {
                tracing::debug!("✔ ACF Manifest gefunden: {:?}", manifest_path);
                if let Some(install_dir_name) = Self::parse_acf_installdir(&manifest_path) {
                    let full_game_path = steamapps_dir.join("common").join(&install_dir_name);
                    if full_game_path.exists() {
                        tracing::info!(
                            "🎯 [Steam-Locator] Echter Spielordner für AppID {} entdeckt: {:?}",
                            app_id, full_game_path
                        );
                        return Some(full_game_path);
                    }
                }
            }
        }

        tracing::warn!("⚠️ [Steam-Locator] AppID {} in {} Bibliotheken nicht gefunden.", app_id, library_roots.len());
        None
    }

    /// Discovers all mounted Steam library root directories across Linux and Windows.
    fn get_all_library_paths() -> Vec<PathBuf> {
        let mut roots = Vec::new();
        let mut base_paths = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                base_paths.push(PathBuf::from(&home).join(".steam/steam"));
                base_paths.push(PathBuf::from(&home).join(".steam/root"));
                base_paths.push(PathBuf::from(&home).join(".local/share/Steam"));
                base_paths.push(PathBuf::from(&home).join(".var/app/com.valvesoftware.Steam/data/Steam"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            base_paths.push(PathBuf::from("C:\\Program Files (x86)\\Steam"));
            base_paths.push(PathBuf::from("C:\\Program Files\\Steam"));
            base_paths.push(PathBuf::from("D:\\Steam"));
            base_paths.push(PathBuf::from("E:\\Steam"));
        }

        for base in base_paths {
            if base.exists() {
                if !roots.contains(&base) {
                    roots.push(base.clone());
                }

                let vdf_path = base.join("steamapps").join("libraryfolders.vdf");
                if vdf_path.exists() {
                    let extra_paths = Self::parse_libraryfolders_vdf(&vdf_path);
                    for p in extra_paths {
                        if !roots.contains(&p) && p.exists() {
                            roots.push(p);
                        }
                    }
                }
            }
        }

        roots
    }

    /// Zero-dependency custom tokenizer that extracts all "path" values from libraryfolders.vdf.
    fn parse_libraryfolders_vdf(vdf_path: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let content = match fs::read_to_string(vdf_path) {
            Ok(c) => c,
            Err(_) => return paths,
        };

        for line in content.lines() {
            if let Some((key, val)) = Self::extract_key_val(line) {
                if key.eq_ignore_ascii_case("path") {
                    // Clean escaped backslashes for Windows compatibility
                    let clean_val = val.replace("\\\\", "\\");
                    paths.push(PathBuf::from(clean_val));
                }
            }
        }

        paths
    }

    /// Extracts the "installdir" directory string from an appmanifest ACF file.
    fn parse_acf_installdir(acf_path: &Path) -> Option<String> {
        let content = fs::read_to_string(acf_path).ok()?;
        for line in content.lines() {
            if let Some((key, val)) = Self::extract_key_val(line) {
                if key.eq_ignore_ascii_case("installdir") {
                    return Some(val);
                }
            }
        }
        None
    }

    /// Helper that safely parses a VDF/ACF line formatted as: "key"  "value"
    fn extract_key_val(line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = line.split('"').collect();
        if parts.len() >= 5 {
            let key = parts[1].trim().to_string();
            let val = parts[3].trim().to_string();
            if !key.is_empty() && !val.is_empty() {
                return Some((key, val));
            }
        }
        None
    }
}