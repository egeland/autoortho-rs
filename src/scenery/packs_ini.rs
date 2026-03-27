//! X-Plane scenery_packs.ini management.
//!
//! X-Plane uses `Custom Scenery/scenery_packs.ini` to discover scenery.
//! AutoOrtho needs to add its entries (z_ao_* and yAutoOrtho_Overlays)
//! to this file for X-Plane to see the scenery packs.

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IniError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("scenery_packs.ini not found at {0}")]
    NotFound(PathBuf),
}

/// A parsed entry from scenery_packs.ini.
#[derive(Debug, Clone, PartialEq)]
pub struct PackEntry {
    /// Full path as it appears in the file (e.g., "Custom Scenery/z_ao_sa/")
    pub path: String,
    /// Whether this entry is enabled (SCENERY_PACK) or disabled (SCENERY_PACK_DISABLED)
    pub enabled: bool,
}

/// Read and parse scenery_packs.ini.
pub fn read_packs_ini(xplane_dir: &Path) -> Result<Vec<PackEntry>, IniError> {
    let ini_path = xplane_dir.join("Custom Scenery").join("scenery_packs.ini");

    if !ini_path.exists() {
        return Err(IniError::NotFound(ini_path));
    }

    let content = std::fs::read_to_string(&ini_path)?;
    let mut entries = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("SCENERY_PACK_DISABLED ") {
            let path = line
                .trim_start_matches("SCENERY_PACK_DISABLED ")
                .trim()
                .to_string();
            entries.push(PackEntry {
                path,
                enabled: false,
            });
        } else if line.starts_with("SCENERY_PACK ") {
            let path = line.trim_start_matches("SCENERY_PACK ").trim().to_string();
            entries.push(PackEntry {
                path,
                enabled: true,
            });
        }
    }

    Ok(entries)
}

/// Write scenery_packs.ini, preserving order.
pub fn write_packs_ini(xplane_dir: &Path, entries: &[PackEntry]) -> Result<(), IniError> {
    let ini_path = xplane_dir.join("Custom Scenery").join("scenery_packs.ini");

    let mut content = String::from("I\n1000 Version\nSCENERY\n\n");

    for entry in entries {
        if entry.enabled {
            content.push_str(&format!("SCENERY_PACK {}\n", entry.path));
        } else {
            content.push_str(&format!("SCENERY_PACK_DISABLED {}\n", entry.path));
        }
    }

    // Atomic write
    let tmp_path = ini_path.with_extension("ini.tmp");
    std::fs::write(&tmp_path, &content)?;
    std::fs::rename(&tmp_path, &ini_path)?;

    Ok(())
}

/// Ensure an AutoOrtho scenery pack entry exists in scenery_packs.ini.
/// Adds it at the end if not present.
pub fn ensure_pack_entry(
    xplane_dir: &Path,
    pack_path: &str,
    enabled: bool,
) -> Result<(), IniError> {
    let mut entries = read_packs_ini(xplane_dir)?;

    // Check if already present
    let existing = entries.iter_mut().find(|e| e.path == pack_path);

    match existing {
        Some(entry) => {
            entry.enabled = enabled;
        }
        None => {
            entries.push(PackEntry {
                path: pack_path.to_string(),
                enabled,
            });
        }
    }

    write_packs_ini(xplane_dir, &entries)
}

/// Remove an entry from scenery_packs.ini.
pub fn remove_pack_entry(xplane_dir: &Path, pack_path: &str) -> Result<(), IniError> {
    let mut entries = read_packs_ini(xplane_dir)?;
    entries.retain(|e| e.path != pack_path);
    write_packs_ini(xplane_dir, &entries)
}

/// Check if a pack is registered and enabled.
pub fn is_pack_enabled(xplane_dir: &Path, pack_path: &str) -> Result<bool, IniError> {
    let entries = read_packs_ini(xplane_dir)?;
    Ok(entries.iter().any(|e| e.path == pack_path && e.enabled))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_ini(tmp: &TempDir, content: &str) {
        let custom_scenery = tmp.path().join("Custom Scenery");
        std::fs::create_dir_all(&custom_scenery).unwrap();
        std::fs::write(custom_scenery.join("scenery_packs.ini"), content).unwrap();
    }

    #[test]
    fn test_read_packs_ini() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
            SCENERY_PACK Custom Scenery/z_ao_sa/\n\
            SCENERY_PACK_DISABLED Custom Scenery/z_ao_na/\n\
            SCENERY_PACK Custom Scenery/yAutoOrtho_Overlays/\n",
        );

        let entries = read_packs_ini(tmp.path()).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].path, "Custom Scenery/z_ao_sa/");
        assert!(entries[0].enabled);
        assert_eq!(entries[1].path, "Custom Scenery/z_ao_na/");
        assert!(!entries[1].enabled);
    }

    #[test]
    fn test_write_packs_ini() {
        let tmp = TempDir::new().unwrap();
        setup_ini(&tmp, ""); // Create directory structure

        let entries = vec![
            PackEntry {
                path: "Custom Scenery/z_ao_sa/".into(),
                enabled: true,
            },
            PackEntry {
                path: "Custom Scenery/z_ao_na/".into(),
                enabled: false,
            },
        ];

        write_packs_ini(tmp.path(), &entries).unwrap();

        let read_back = read_packs_ini(tmp.path()).unwrap();
        assert_eq!(read_back.len(), 2);
        assert!(read_back[0].enabled);
        assert!(!read_back[1].enabled);
    }

    #[test]
    fn test_ensure_pack_entry_adds_new() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
            SCENERY_PACK Custom Scenery/existing_pack/\n",
        );

        ensure_pack_entry(tmp.path(), "Custom Scenery/z_ao_sa/", true).unwrap();

        let entries = read_packs_ini(tmp.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .any(|e| e.path == "Custom Scenery/z_ao_sa/" && e.enabled)
        );
    }

    #[test]
    fn test_ensure_pack_entry_updates_existing() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
            SCENERY_PACK_DISABLED Custom Scenery/z_ao_sa/\n",
        );

        ensure_pack_entry(tmp.path(), "Custom Scenery/z_ao_sa/", true).unwrap();

        let entries = read_packs_ini(tmp.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].enabled);
    }

    #[test]
    fn test_remove_pack_entry() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
            SCENERY_PACK Custom Scenery/z_ao_sa/\n\
            SCENERY_PACK Custom Scenery/z_ao_na/\n",
        );

        remove_pack_entry(tmp.path(), "Custom Scenery/z_ao_sa/").unwrap();

        let entries = read_packs_ini(tmp.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "Custom Scenery/z_ao_na/");
    }

    #[test]
    fn test_is_pack_enabled() {
        let tmp = TempDir::new().unwrap();
        setup_ini(
            &tmp,
            "I\n1000 Version\nSCENERY\n\n\
            SCENERY_PACK Custom Scenery/z_ao_sa/\n\
            SCENERY_PACK_DISABLED Custom Scenery/z_ao_na/\n",
        );

        assert!(is_pack_enabled(tmp.path(), "Custom Scenery/z_ao_sa/").unwrap());
        assert!(!is_pack_enabled(tmp.path(), "Custom Scenery/z_ao_na/").unwrap());
        assert!(!is_pack_enabled(tmp.path(), "Custom Scenery/nonexistent/").unwrap());
    }

    #[test]
    fn test_not_found() {
        let tmp = TempDir::new().unwrap();
        let result = read_packs_ini(tmp.path());
        assert!(result.is_err());
    }
}
