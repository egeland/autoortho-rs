//! Scenery pack metadata, installation, and management.
//!
//! Re-exports download and extract functions for backward compatibility.

pub use super::download::{
    DownloadError, clean_downloads, download_hash_file, download_package, has_partial_downloads,
    verify_file_hash,
};
pub use super::extract::{ExtractError, extract_zip, extract_zip_with_pack_progress};

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("Download failed: {0}")]
    Download(String),
    #[error("Extract failed: {0}")]
    Extract(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cancelled")]
    Cancelled,
}

impl From<DownloadError> for InstallError {
    fn from(e: DownloadError) -> Self {
        match e {
            DownloadError::Download(s) => InstallError::Download(s),
            DownloadError::Io(e) => InstallError::Io(e),
            DownloadError::Cancelled => InstallError::Cancelled,
        }
    }
}

impl From<ExtractError> for InstallError {
    fn from(e: ExtractError) -> Self {
        match e {
            ExtractError::Extract(s) => InstallError::Extract(s),
            ExtractError::Io(e) => InstallError::Io(e),
        }
    }
}

/// Metadata for an installed scenery pack (stored as *_info.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackInfo {
    pub id: String,
    pub name: String,
    pub ver: String,
    pub ortho_prefix: String,
    pub overlay_prefix: String,
    pub ortho_dirs: Vec<String>,
    #[serde(default)]
    pub info_ver: String,
}

/// Save pack metadata to *_info.json.
pub fn save_pack_info(info: &PackInfo, data_dir: &Path) -> Result<(), InstallError> {
    let filename = format!("{}_info.json", info.id);
    let path = data_dir.join(filename);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json =
        serde_json::to_string_pretty(info).map_err(|e| InstallError::Extract(e.to_string()))?;
    std::fs::write(&path, json)?;

    info!("Saved pack info: {}", path.display());
    Ok(())
}

/// Load pack metadata from *_info.json.
pub fn load_pack_info(region_id: &str, data_dir: &Path) -> Result<PackInfo, InstallError> {
    let filename = format!("{}_info.json", region_id);
    let path = data_dir.join(filename);

    let json = std::fs::read_to_string(&path)?;
    let info: PackInfo =
        serde_json::from_str(&json).map_err(|e| InstallError::Extract(e.to_string()))?;

    Ok(info)
}

/// Uninstall a scenery region: remove installed files and metadata.
pub fn uninstall_region(region_id: &str, data_dir: &Path) -> Result<(), InstallError> {
    let scenery_path = data_dir.join("scenery").join(format!("z_ao_{}", region_id));
    if scenery_path.exists() {
        info!("Removing scenery directory: {}", scenery_path.display());
        std::fs::remove_dir_all(&scenery_path)?;
    }

    let info_path = data_dir.join(format!("{}_info.json", region_id));
    if info_path.exists() {
        info!("Removing metadata: {}", info_path.display());
        std::fs::remove_file(&info_path)?;
    }

    info!("Uninstalled region '{}'", region_id);
    Ok(())
}

/// List installed scenery packs by scanning for *_info.json files.
pub fn list_installed_packs(data_dir: &Path) -> Vec<PackInfo> {
    let mut packs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("_info.json")
                && let Ok(json) = std::fs::read_to_string(entry.path())
                && let Ok(info) = serde_json::from_str::<PackInfo>(&json)
            {
                packs.push(info);
            }
        }
    }

    packs.sort_by(|a, b| a.id.cmp(&b.id));
    packs
}

/// Migrate scenery files from old location (`{xplane}/Custom Scenery/z_autoortho/`)
/// to the new data directory (`{cache_dir}/scenery/z_autoortho/`).
///
/// Skips `textures/` (was the old mount point — now we mount at the parent).
/// Returns the number of items migrated.
pub fn migrate_scenery(old_dir: &Path, new_dir: &Path) -> Result<u32, InstallError> {
    let mut count = 0;

    if !old_dir.exists() {
        return Ok(0);
    }

    std::fs::create_dir_all(new_dir)?;

    if let Ok(entries) = std::fs::read_dir(old_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip textures dir (was the old mount point)
            if name == "textures" {
                continue;
            }
            let source = entry.path();
            let dest = new_dir.join(&name);
            // Don't overwrite existing files in new location
            if dest.exists() {
                info!(
                    "Skipping migration of {} — already exists at new location",
                    name
                );
                continue;
            }
            info!("Migrating {} to {}", source.display(), dest.display());
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                // Try rename first (fast if same filesystem), fall back to copy
                if std::fs::rename(&source, &dest).is_err() {
                    info!("rename failed for {}, trying copy", source.display());
                    copy_dir(&source, &dest)?;
                    std::fs::remove_dir_all(&source)?;
                }
            } else {
                std::fs::rename(&source, &dest).or_else(|_| {
                    std::fs::copy(&source, &dest)?;
                    std::fs::remove_file(&source)
                })?;
            }
            count += 1;
        }
    }

    Ok(count)
}

/// Recursively copy a directory.
fn copy_dir(src: &Path, dst: &Path) -> Result<(), InstallError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_info() -> PackInfo {
        PackInfo {
            id: "sa".to_string(),
            name: "South America".to_string(),
            ver: "0.0.53".to_string(),
            ortho_prefix: "z_sa_".to_string(),
            overlay_prefix: "y_sa_overlays".to_string(),
            ortho_dirs: vec![
                "/Custom Scenery/z_sa_00".to_string(),
                "/Custom Scenery/z_sa_01".to_string(),
            ],
            info_ver: "v1".to_string(),
        }
    }

    #[test]
    fn test_save_and_load_pack_info() {
        let tmp = TempDir::new().unwrap();
        let info = sample_info();

        save_pack_info(&info, tmp.path()).unwrap();
        let loaded = load_pack_info("sa", tmp.path()).unwrap();

        assert_eq!(loaded.id, "sa");
        assert_eq!(loaded.ver, "0.0.53");
        assert_eq!(loaded.ortho_dirs.len(), 2);
    }

    #[test]
    fn test_list_installed_empty() {
        let tmp = TempDir::new().unwrap();
        let packs = list_installed_packs(tmp.path());
        assert!(packs.is_empty());
    }

    #[test]
    fn test_list_installed_with_packs() {
        let tmp = TempDir::new().unwrap();

        save_pack_info(&sample_info(), tmp.path()).unwrap();

        let mut info2 = sample_info();
        info2.id = "na".to_string();
        info2.name = "North America".to_string();
        save_pack_info(&info2, tmp.path()).unwrap();

        let packs = list_installed_packs(tmp.path());
        assert_eq!(packs.len(), 2);
        assert_eq!(packs[0].id, "na");
        assert_eq!(packs[1].id, "sa");
    }

    #[test]
    fn test_uninstall_region() {
        let tmp = TempDir::new().unwrap();
        save_pack_info(&sample_info(), tmp.path()).unwrap();

        // Create fake scenery dir
        let scenery_dir = tmp.path().join("scenery").join("z_ao_sa");
        std::fs::create_dir_all(&scenery_dir).unwrap();
        std::fs::write(scenery_dir.join("test.dsf"), b"data").unwrap();

        uninstall_region("sa", tmp.path()).unwrap();

        assert!(!scenery_dir.exists());
        assert!(!tmp.path().join("sa_info.json").exists());
    }

    #[test]
    fn test_migrate_scenery_empty_old() {
        let tmp = TempDir::new().unwrap();
        let old = tmp.path().join("old");
        let new = tmp.path().join("new");
        let count = migrate_scenery(&old, &new).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_migrate_scenery_moves_files() {
        let tmp = TempDir::new().unwrap();
        let old = tmp.path().join("old");
        let new = tmp.path().join("new");

        std::fs::create_dir_all(old.join("scenery").join("z_ao_na")).unwrap();
        std::fs::write(
            old.join("scenery").join("z_ao_na").join("+40-070.dsf"),
            b"dsf",
        )
        .unwrap();
        std::fs::write(old.join("na_info.json"), b"info").unwrap();
        std::fs::create_dir_all(old.join("textures")).unwrap();

        let count = migrate_scenery(&old, &new).unwrap();
        assert_eq!(count, 2);

        assert!(
            new.join("scenery")
                .join("z_ao_na")
                .join("+40-070.dsf")
                .exists()
        );
        assert!(new.join("na_info.json").exists());
        assert!(!new.join("textures").exists());
    }

    #[test]
    fn test_install_error_from_download_error() {
        let err = InstallError::from(DownloadError::Download("test".into()));
        assert!(matches!(err, InstallError::Download(_)));

        let err = InstallError::from(DownloadError::Cancelled);
        assert!(matches!(err, InstallError::Cancelled));
    }

    #[test]
    fn test_install_error_from_extract_error() {
        let err = InstallError::from(ExtractError::Extract("test".into()));
        assert!(matches!(err, InstallError::Extract(_)));
    }

    #[test]
    fn test_save_pack_info_invalid_path() {
        let info = sample_info();
        let tmp = TempDir::new().unwrap();
        // Create a file, then try to write under it — fails on all platforms
        let file_as_dir = tmp.path().join("blocking_file");
        std::fs::write(&file_as_dir, b"x").unwrap();
        let result = save_pack_info(&info, &file_as_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_pack_info_missing_file() {
        let tmp = TempDir::new().unwrap();
        let result = load_pack_info("nonexistent", tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_uninstall_region_no_scenery_dir() {
        let tmp = TempDir::new().unwrap();
        // No scenery dir exists, should still succeed
        uninstall_region("nonexistent", tmp.path()).unwrap();
    }

    #[test]
    fn test_migrate_scenery_skip_existing_dest() {
        let tmp = TempDir::new().unwrap();
        let old = tmp.path().join("old");
        let new = tmp.path().join("new");

        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(old.join("test.txt"), b"old").unwrap();
        // Pre-existing file in new location
        std::fs::write(new.join("test.txt"), b"new").unwrap();

        let count = migrate_scenery(&old, &new).unwrap();
        assert_eq!(count, 0); // Skipped because dest exists
        // Original content preserved
        assert_eq!(
            std::fs::read_to_string(new.join("test.txt")).unwrap(),
            "new"
        );
    }

    #[test]
    fn test_migrate_scenery_copies_dir() {
        let tmp = TempDir::new().unwrap();
        let old = tmp.path().join("old");
        let new = tmp.path().join("new");

        // Create nested dir structure
        std::fs::create_dir_all(old.join("subdir").join("nested")).unwrap();
        std::fs::write(old.join("file.txt").to_path_buf(), b"data").unwrap();
        std::fs::write(old.join("subdir").join("nested").join("deep.txt"), b"deep").unwrap();

        let count = migrate_scenery(&old, &new).unwrap();
        assert_eq!(count, 2);
        assert!(new.join("file.txt").exists());
        assert!(new.join("subdir").join("nested").join("deep.txt").exists());
    }
}
