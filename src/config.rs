// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// All persistent configuration for AutoOrtho.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutoOrthoConfig {
    pub mount_dir: String,
    pub cache_dir: String,
    pub xplane_host: String,
    pub xplane_port: u16,
    pub tile_provider: String,
    pub min_zoom: u32,
    pub max_zoom: u32,
    pub enable_night_exclusion: bool,
    pub night_threshold: f32,
    pub day_threshold: f32,
    #[serde(default)]
    pub scenery_download_dir: String,
    #[serde(default)]
    pub scenery_install_dir: String,
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f64,
    #[serde(default)]
    pub window_x: Option<f32>,
    #[serde(default)]
    pub window_y: Option<f32>,
    #[serde(default)]
    pub window_width: Option<f32>,
    #[serde(default)]
    pub window_height: Option<f32>,
}

fn default_ui_scale() -> f64 {
    1.0
}

impl Default for AutoOrthoConfig {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .map(|p| p.join("autoortho").to_string_lossy().into_owned())
            .unwrap_or_else(|| "autoortho_cache".to_string());

        let mount_dir = if cfg!(target_os = "windows") {
            dirs::home_dir()
                .map(|p| p.join("autoortho_mount").to_string_lossy().into_owned())
                .unwrap_or_else(|| "C:\\autoortho".to_string())
        } else {
            "/tmp/autoortho".to_string()
        };

        let scenery_download_dir = dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
            .map(|p| p.join("autoortho-scenery").to_string_lossy().into_owned())
            .unwrap_or_else(|| "downloads".to_string());

        let scenery_install_dir = dirs::home_dir()
            .map(|p| {
                p.join("X-Plane 12")
                    .join("Custom Scenery")
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap_or_else(|| "Custom Scenery".to_string());

        Self {
            mount_dir,
            cache_dir,
            xplane_host: "127.0.0.1".to_string(),
            xplane_port: 49000,
            tile_provider: "ARC".to_string(),
            min_zoom: 10,
            max_zoom: 18,
            enable_night_exclusion: true,
            night_threshold: -12.0,
            day_threshold: -10.0,
            scenery_download_dir,
            scenery_install_dir,
            ui_scale: 1.0,
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
        }
    }
}

impl AutoOrthoConfig {
    /// Platform-appropriate config file path.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("autoortho")
            .join("config.toml")
    }

    /// Load config from the default path, falling back to defaults.
    pub fn load() -> Self {
        let path = Self::config_path();
        match Self::from_file(&path) {
            Ok(config) => {
                info!("Loaded config from {}", path.display());
                config
            }
            Err(e) => {
                if path.exists() {
                    warn!(
                        "Failed to load config from {}: {}. Using defaults.",
                        path.display(),
                        e
                    );
                }
                Self::default()
            }
        }
    }

    /// Save config to the default path.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        self.save_to(&path)
    }

    /// Save config to a specific path.
    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create config directory: {}", e))?;
        }

        let toml = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        // Atomic write
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, &toml).map_err(|e| format!("Failed to write config: {}", e))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("Failed to rename config: {}", e))?;

        info!("Saved config to {}", path.display());
        Ok(())
    }

    /// Clear saved window position/size (for --reset-window).
    pub fn reset_window_position(&mut self) {
        self.window_x = None;
        self.window_y = None;
        self.window_width = None;
        self.window_height = None;
    }

    /// Load from a specific file.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        toml::from_str(&content).map_err(|e| format!("Cannot parse {}: {}", path.display(), e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = AutoOrthoConfig::default();
        assert_eq!(config.xplane_port, 49000);
        assert_eq!(config.min_zoom, 10);
        assert_eq!(config.max_zoom, 18);
        assert!(config.enable_night_exclusion);
    }

    #[test]
    fn test_config_clone() {
        let config1 = AutoOrthoConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.xplane_host, config2.xplane_host);
    }

    #[test]
    fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");

        let mut config = AutoOrthoConfig::default();
        config.tile_provider = "BI".to_string();
        config.xplane_port = 12345;
        config.scenery_download_dir = "/my/downloads".to_string();

        config.save_to(&path).unwrap();

        let loaded = AutoOrthoConfig::from_file(&path).unwrap();
        assert_eq!(loaded.tile_provider, "BI");
        assert_eq!(loaded.xplane_port, 12345);
        assert_eq!(loaded.scenery_download_dir, "/my/downloads");
    }

    #[test]
    fn test_load_missing_file_returns_error() {
        let result = AutoOrthoConfig::from_file(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_with_fallback() {
        // load() returns defaults when file doesn't exist
        let config = AutoOrthoConfig::load();
        assert_eq!(config.xplane_port, 49000);
    }

    #[test]
    fn test_config_path_not_empty() {
        let path = AutoOrthoConfig::config_path();
        assert!(path.to_string_lossy().contains("autoortho"));
    }
}
