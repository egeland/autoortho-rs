//! Custom map configuration — per-cell provider overrides.
//!
//! Stores which satellite provider to use for each 1°×1° geographic cell.
//! Persisted as custom_map.json in the config directory.

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Custom map configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMapConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    /// Cell key ("lat,lon") → maptype ("BI", "ARC", etc.)
    pub cells: HashMap<String, String>,
}

fn default_version() -> u32 {
    1
}

impl Default for CustomMapConfig {
    fn default() -> Self {
        Self {
            version: 1,
            cells: HashMap::new(),
        }
    }
}

/// Thread-safe custom map store with disk persistence.
pub struct CustomMapStore {
    config: Mutex<CustomMapConfig>,
    path: PathBuf,
}

impl CustomMapStore {
    /// Load from disk or create empty.
    pub fn load(path: PathBuf) -> Arc<Self> {
        let config = if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
                Err(_) => CustomMapConfig::default(),
            }
        } else {
            CustomMapConfig::default()
        };

        info!(
            "Custom map: {} cells loaded from {}",
            config.cells.len(),
            path.display()
        );

        Arc::new(Self {
            config: Mutex::new(config),
            path,
        })
    }

    /// Get all cells.
    pub fn get_cells(&self) -> HashMap<String, String> {
        self.config.lock().expect("custommap lock").cells.clone()
    }

    /// Set/overwrite cells.
    pub fn set_cells(&self, cells: HashMap<String, String>) {
        {
            let mut cfg = self.config.lock().expect("custommap lock");
            for (k, v) in cells {
                cfg.cells.insert(k, v);
            }
        }
        self.save();
    }

    /// Remove cells by key.
    pub fn remove_cells(&self, keys: &[String]) {
        {
            let mut cfg = self.config.lock().expect("custommap lock");
            for k in keys {
                cfg.cells.remove(k);
            }
        }
        self.save();
    }

    /// Clear all cells.
    pub fn clear(&self) {
        {
            self.config.lock().expect("custommap lock").cells.clear();
        }
        self.save();
    }

    /// Export as JSON string.
    pub fn export_json(&self) -> String {
        let cfg = self.config.lock().expect("custommap lock");
        serde_json::to_string_pretty(&*cfg).unwrap_or_default()
    }

    /// Import from JSON, optionally merging with existing.
    pub fn import_json(&self, json: &str, merge: bool) -> Result<usize, String> {
        let imported: CustomMapConfig =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

        let count = imported.cells.len();
        {
            let mut cfg = self.config.lock().expect("custommap lock");
            if !merge {
                cfg.cells.clear();
            }
            for (k, v) in imported.cells {
                cfg.cells.insert(k, v);
            }
        }
        self.save();
        Ok(count)
    }

    fn save(&self) {
        let cfg = self.config.lock().expect("custommap lock");
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let json = serde_json::to_string_pretty(&*cfg).unwrap_or_default();
        // Atomic write
        let tmp = self.path.with_extension("json.tmp");
        if std::fs::write(&tmp, &json).is_ok() {
            std::fs::rename(&tmp, &self.path).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_set_and_get_cells() {
        let tmp = TempDir::new().unwrap();
        let store = CustomMapStore::load(tmp.path().join("custom_map.json"));

        let mut cells = HashMap::new();
        cells.insert("-34,151".to_string(), "BI".to_string());
        cells.insert("48,2".to_string(), "ARC".to_string());
        store.set_cells(cells);

        let got = store.get_cells();
        assert_eq!(got.len(), 2);
        assert_eq!(got["-34,151"], "BI");
    }

    #[test]
    fn test_remove_cells() {
        let tmp = TempDir::new().unwrap();
        let store = CustomMapStore::load(tmp.path().join("custom_map.json"));

        let mut cells = HashMap::new();
        cells.insert("a".to_string(), "BI".to_string());
        cells.insert("b".to_string(), "ARC".to_string());
        store.set_cells(cells);

        store.remove_cells(&["a".to_string()]);
        let got = store.get_cells();
        assert_eq!(got.len(), 1);
        assert!(!got.contains_key("a"));
    }

    #[test]
    fn test_clear() {
        let tmp = TempDir::new().unwrap();
        let store = CustomMapStore::load(tmp.path().join("custom_map.json"));

        let mut cells = HashMap::new();
        cells.insert("a".to_string(), "BI".to_string());
        store.set_cells(cells);
        store.clear();

        assert!(store.get_cells().is_empty());
    }

    #[test]
    fn test_persistence() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("custom_map.json");

        {
            let store = CustomMapStore::load(path.clone());
            let mut cells = HashMap::new();
            cells.insert("test".to_string(), "NAIP".to_string());
            store.set_cells(cells);
        }

        // Reload from disk
        let store2 = CustomMapStore::load(path);
        let got = store2.get_cells();
        assert_eq!(got["test"], "NAIP");
    }

    #[test]
    fn test_import_export() {
        let tmp = TempDir::new().unwrap();
        let store = CustomMapStore::load(tmp.path().join("custom_map.json"));

        let mut cells = HashMap::new();
        cells.insert("x".to_string(), "BI".to_string());
        store.set_cells(cells);

        let json = store.export_json();
        store.clear();
        assert!(store.get_cells().is_empty());

        let count = store.import_json(&json, false).unwrap();
        assert_eq!(count, 1);
        assert_eq!(store.get_cells()["x"], "BI");
    }
}
