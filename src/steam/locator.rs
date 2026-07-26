// =====================================================================
// RetroLAN VPN - Steam & Retro Game Installation Locator
// Parses libraryfolders.vdf, appmanifests, and scans standard Linux/Win
// directories (GOG, Heroic, Wine, Lutris) to locate game folders.
// =====================================================================

use std::path::{Path, PathBuf};
use std::fs;

#[allow(dead_code)]
pub struct SteamGameLocator;

#[allow(dead_code)]
impl SteamGameLocator {
    /// Attempts to locate the physical installation directory of a game using a 3-stage smart fallback:
    /// 1. Steam AppID ACF manifest lookup across all discovered Steam library folders.
    /// 2. Direct directory name matching inside `steamapps/common/<game_name>`.
    /// 3. Standard Linux / Windows external launcher paths (GOG Galaxy, Heroic, Wine prefixes, ~/Games).
    pub fn find_game_dir(app_id: Option<u32>, game_name: &str) -> Option<PathBuf> {
        tracing::debug!("🔍 [Game-Locator] Starte 3-Stufen-Scan für '{}' (AppID: {:?})...", game_name, app_id);

        let library_roots = Self::get_all_library_paths();
        tracing::debug!("📁 [Game-Locator] Durchsuche {} Steam-Bibliotheksverzeichnisse...", library_roots.len());

        // --- STAGE 1: Steam ACF Manifest Lookup (by AppID) ---
        if let Some(id) = app_id {
            let manifest_filename = format!("appmanifest_{}.acf", id);
            for root in &library_roots {
                let steamapps_dir = root.join("steamapps");
                let manifest_path = steamapps_dir.join(&manifest_filename);

                if manifest_path.exists() {
                    if let Some(install_dir_name) = Self::parse_acf_installdir(&manifest_path) {
                        let full_game_path = steamapps_dir.join("common").join(&install_dir_name);
                        if full_game_path.exists() {
                            tracing::info!("🎯 [Stage 1 - ACF] Spielordner über AppID {} entdeckt: {:?}", id, full_game_path);
                            return Some(full_game_path);
                        }
                    }
                }
            }
        }

        // --- STAGE 2: Direct Steam Common Folder Scan (by Game Name) ---
        for root in &library_roots {
            let common_dir = root.join("steamapps").join("common");
            if common_dir.exists() {
                // Exact match check first
                let exact_path = common_dir.join(game_name);
                if exact_path.exists() {
                    tracing::info!("🎯 [Stage 2 - Common] Spielordner direkt in Steam Library entdeckt: {:?}", exact_path);
                    return Some(exact_path);
                }

                // Case-insensitive fuzzy scan inside common/
                if let Ok(entries) = fs::read_dir(&common_dir) {
                    for entry in entries.flatten() {
                        if let Ok(file_name) = entry.file_name().into_string() {
                            if file_name.eq_ignore_ascii_case(game_name) || file_name.to_lowercase().contains(&game_name.to_lowercase()) {
                                let path = entry.path();
                                if path.is_dir() {
                                    tracing::info!("🎯 [Stage 2 - Fuzzy] Ähnlichen Steam-Ordner entdeckt: {:?}", path);
                                    return Some(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        // --- STAGE 3: External Linux/Windows Retro Launchers (GOG, Heroic, Wine, ~/Games) ---
        let external_paths = Self::get_external_launcher_paths(game_name);
        for ext_path in external_paths {
            if ext_path.exists() && ext_path.is_dir() {
                tracing::info!("🎯 [Stage 3 - External] Spielordner in externem Launcher entdeckt: {:?}", ext_path);
                return Some(ext_path);
            }
        }

        tracing::warn!("⚠️ [Game-Locator] '{}' konnte in keiner Bibliothek gefunden werden. Weiche auf '.' aus.", game_name);
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

    /// Returns standard installation paths for GOG Galaxy, Heroic, Lutris, and standard Wine prefixes.
    fn get_external_launcher_paths(game_name: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let home_dir = PathBuf::from(&home);
                paths.push(home_dir.join("Games").join(game_name));
                paths.push(home_dir.join(".config/heroic/games").join(game_name));
                paths.push(home_dir.join(".local/share/lutris/runners/wine").join(game_name));
                paths.push(home_dir.join(".wine/drive_c/Program Files (x86)/GOG Galaxy/Games").join(game_name));
                paths.push(home_dir.join(".wine/drive_c/Program Files/GOG Galaxy/Games").join(game_name));
                paths.push(home_dir.join(".wine/drive_c/Games").join(game_name));
            }
        }

        #[cfg(target_os = "windows")]
        {
            paths.push(PathBuf::from(format!("C:\\Program Files (x86)\\GOG Galaxy\\Games\\{}", game_name)));
            paths.push(PathBuf::from(format!("C:\\Games\\{}", game_name)));
            paths.push(PathBuf::from(format!("D:\\Games\\{}", game_name)));
        }

        paths
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
