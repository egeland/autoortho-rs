// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Season selection for seasonal adjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Season {
    #[default]
    Disabled,
    Spring,
    Summer,
    Autumn,
    Winter,
}

/// Fallback level for missing tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FallbackLevel {
    #[default]
    Cache, // Check disk cache for lower-zoom tiles
    Downserve, // Scale from lower-resolution tile
    Network,   // Download on-demand
    Solid,     // Solid color fallback
}

/// Fallback configuration for missing tiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub level: FallbackLevel,
    pub max_zoom_gap: u32,
    pub solid_color: [u8; 3],
    pub cache_fallback: bool,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            level: FallbackLevel::Cache,
            max_zoom_gap: 4,
            solid_color: [20, 25, 15],
            cache_fallback: true,
        }
    }
}

/// A zoom rule: at or above this AGL altitude, use this zoom level.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZoomRule {
    /// Minimum AGL altitude in feet for this rule
    pub min_altitude_ft: f32,
    /// Zoom level to use at this altitude
    pub zoom_level: u32,
}

impl Default for ZoomRule {
    fn default() -> Self {
        Self {
            min_altitude_ft: 0.0,
            zoom_level: 19,
        }
    }
}

/// All persistent configuration for AutoOrtho.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutoOrthoConfig {
    #[serde(default = "default_xplane_path")]
    pub xplane_path: String,
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
    #[serde(default = "default_dds_cache_size_mb")]
    pub dds_cache_size_mb: u64,
    #[serde(default = "default_enable_dds_cache")]
    pub enable_dds_cache: bool,
    #[serde(default)]
    pub simbrief_user_id: String,
    #[serde(default = "default_route_consideration_radius_nm")]
    pub route_consideration_radius_nm: u32,
    #[serde(default = "default_route_deviation_threshold_nm")]
    pub route_deviation_threshold_nm: u32,
    #[serde(default = "default_route_prefetch_radius_nm")]
    pub route_prefetch_radius_nm: u32,
    #[serde(default = "default_prefetch_route_percent")]
    pub prefetch_route_percent: u32,
    #[serde(default = "default_prefetch_airports")]
    pub prefetch_airports: bool,
    #[serde(default = "default_airport_radius_nm")]
    pub airport_radius_nm: u32,
    #[serde(default = "default_near_airport_zoom")]
    pub near_airport_zoom: u32,
    #[serde(default = "default_enable_dynamic_zoom")]
    pub enable_dynamic_zoom: bool,
    #[serde(default = "default_zoom_rules")]
    pub zoom_rules: Vec<ZoomRule>,
    #[serde(default = "default_dds_memory_cache_mb")]
    pub dds_memory_cache_mb: u64,
    #[serde(default = "default_chunk_memory_cache_mb")]
    pub chunk_memory_cache_mb: u64,
    #[serde(default)]
    pub season: Season,
    #[serde(default = "default_spring_saturation")]
    pub spring_saturation: f32,
    #[serde(default = "default_summer_saturation")]
    pub summer_saturation: f32,
    #[serde(default = "default_autumn_saturation")]
    pub autumn_saturation: f32,
    #[serde(default = "default_winter_saturation")]
    pub winter_saturation: f32,
    #[serde(default)]
    pub fallback: FallbackConfig,
}

fn default_ui_scale() -> f64 {
    1.0
}

fn default_dds_cache_size_mb() -> u64 {
    4096
}

fn default_enable_dds_cache() -> bool {
    true
}

fn default_route_consideration_radius_nm() -> u32 {
    50
}

fn default_route_deviation_threshold_nm() -> u32 {
    40
}

fn default_route_prefetch_radius_nm() -> u32 {
    40
}

fn default_prefetch_route_percent() -> u32 {
    20
}

fn default_prefetch_airports() -> bool {
    true
}

fn default_airport_radius_nm() -> u32 {
    60
}

fn default_spring_saturation() -> f32 {
    0.70
}

fn default_summer_saturation() -> f32 {
    1.0
}

fn default_autumn_saturation() -> f32 {
    0.80
}

fn default_winter_saturation() -> f32 {
    0.55
}

fn default_near_airport_zoom() -> u32 {
    19
}

fn default_dds_memory_cache_mb() -> u64 {
    256
}

fn default_chunk_memory_cache_mb() -> u64 {
    512
}

impl AutoOrthoConfig {
    /// Estimated memory per DDS tile in MB (4096x4096 BC3 compressed).
    const DDS_TILE_SIZE_MB: u64 = 22;

    /// Estimated memory per chunk in KB (256x256 JPEG).
    const CHUNK_SIZE_KB: u64 = 30;

    /// Calculate the number of DDS tiles that fit in the configured memory.
    pub fn dds_memory_cache_entries(&self) -> usize {
        ((self.dds_memory_cache_mb / Self::DDS_TILE_SIZE_MB).max(1)) as usize
    }

    /// Calculate the number of chunks that fit in the configured memory.
    pub fn chunk_memory_cache_entries(&self) -> usize {
        ((self.chunk_memory_cache_mb * 1024 / Self::CHUNK_SIZE_KB).max(1)) as usize
    }
}

fn default_enable_dynamic_zoom() -> bool {
    true
}

fn default_zoom_rules() -> Vec<ZoomRule> {
    vec![
        ZoomRule {
            min_altitude_ft: 0.0,
            zoom_level: 19,
        },
        ZoomRule {
            min_altitude_ft: 10000.0,
            zoom_level: 16,
        },
    ]
}

fn default_xplane_path() -> String {
    dirs::home_dir()
        .map(|p| p.join("X-Plane 12").to_string_lossy().into_owned())
        .unwrap_or_else(|| "X-Plane 12".to_string())
}

impl Default for AutoOrthoConfig {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .map(|p| p.join("autoortho").to_string_lossy().into_owned())
            .unwrap_or_else(|| "autoortho_cache".to_string());

        let scenery_download_dir = dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
            .map(|p| p.join("autoortho-scenery").to_string_lossy().into_owned())
            .unwrap_or_else(|| "downloads".to_string());

        Self {
            xplane_path: default_xplane_path(),
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
            ui_scale: 1.0,
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
            dds_cache_size_mb: 4096,
            enable_dds_cache: true,
            simbrief_user_id: String::new(),
            route_consideration_radius_nm: 50,
            route_deviation_threshold_nm: 40,
            route_prefetch_radius_nm: 40,
            prefetch_route_percent: 20,
            prefetch_airports: true,
            airport_radius_nm: 60,
            near_airport_zoom: 19,
            enable_dynamic_zoom: true,
            zoom_rules: default_zoom_rules(),
            dds_memory_cache_mb: default_dds_memory_cache_mb(),
            chunk_memory_cache_mb: default_chunk_memory_cache_mb(),
            season: Season::Disabled,
            spring_saturation: 0.70,
            summer_saturation: 1.0,
            autumn_saturation: 0.80,
            winter_saturation: 0.55,
            fallback: FallbackConfig::default(),
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

        debug!("Saved config to {}", path.display());
        Ok(())
    }

    /// X-Plane's Custom Scenery directory, derived from `xplane_path`.
    pub fn custom_scenery_path(&self) -> PathBuf {
        PathBuf::from(&self.xplane_path).join("Custom Scenery")
    }

    /// FUSE mount point, derived from `xplane_path`.
    pub fn mount_dir(&self) -> PathBuf {
        self.custom_scenery_path()
            .join("z_autoortho")
            .join("textures")
    }

    /// Scenery install directory (Custom Scenery), derived from `xplane_path`.
    pub fn scenery_install_dir(&self) -> PathBuf {
        self.custom_scenery_path()
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

    #[test]
    fn test_derived_paths() {
        let mut config = AutoOrthoConfig::default();
        config.xplane_path = "/home/user/X-Plane 12".to_string();

        assert_eq!(
            config.custom_scenery_path(),
            PathBuf::from("/home/user/X-Plane 12/Custom Scenery")
        );
        assert_eq!(
            config.mount_dir(),
            PathBuf::from("/home/user/X-Plane 12/Custom Scenery/z_autoortho/textures")
        );
        assert_eq!(
            config.scenery_install_dir(),
            PathBuf::from("/home/user/X-Plane 12/Custom Scenery")
        );
    }
}
