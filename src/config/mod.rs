// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

pub mod cache;
pub mod flight;
pub mod network;
pub mod night;
pub mod season;
pub mod ui;

use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use cache::CacheConfig;
pub use flight::FlightConfig;
pub use network::NetworkConfig;
pub use night::NightConfig;
pub use season::SeasonConfig;
pub use ui::UiConfig;

// Re-export for backward compat
pub use crate::errors::ConfigError;
pub use crate::errors::RateLimitConfig;
pub use crate::seasons::Season;
pub use crate::tiles::fallback::{FallbackConfig, FallbackLevel};
pub use crate::tiles::zoom::ZoomRule;

/// Tile configuration — provider, zoom levels, dynamic zoom rules.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TileConfig {
    #[serde(default = "default_tile_provider")]
    pub provider: String,
    #[serde(default = "default_min_zoom")]
    pub min_zoom: u32,
    #[serde(default = "default_max_zoom")]
    pub max_zoom: u32,
    #[serde(default = "default_enable_dynamic_zoom")]
    pub enable_dynamic_zoom: bool,
    #[serde(default = "default_zoom_rules")]
    pub zoom_rules: Vec<ZoomRule>,
}

impl Default for TileConfig {
    fn default() -> Self {
        Self {
            provider: default_tile_provider(),
            min_zoom: default_min_zoom(),
            max_zoom: default_max_zoom(),
            enable_dynamic_zoom: default_enable_dynamic_zoom(),
            zoom_rules: default_zoom_rules(),
        }
    }
}

impl TileConfig {
    pub fn validate(&self) -> Result<(), crate::errors::ConfigError> {
        if self.provider.is_empty() {
            return Err(crate::errors::ConfigError::FieldOutOfRange {
                field: "tile.provider".to_string(),
                min: 1,
                max: 0,
                value: 0,
            });
        }
        crate::errors::validate_range(self.min_zoom as u64, 1, 22, "tile.min_zoom")?;
        crate::errors::validate_range(self.max_zoom as u64, 1, 22, "tile.max_zoom")?;
        if self.min_zoom > self.max_zoom {
            return Err(crate::errors::ConfigError::FieldOutOfRange {
                field: "tile.min_zoom".to_string(),
                min: self.min_zoom as u64,
                max: self.max_zoom as u64,
                value: self.min_zoom as u64,
            });
        }
        Ok(())
    }
}

fn default_tile_provider() -> String {
    "ARC".to_string()
}

fn default_min_zoom() -> u32 {
    10
}

fn default_max_zoom() -> u32 {
    18
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

/// All persistent configuration for AutoOrtho.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutoOrthoConfig {
    #[serde(default = "default_xplane_path")]
    pub xplane_path: String,
    pub cache_dir: String,

    /// Tile configuration (provider, zoom levels, dynamic zoom)
    #[serde(flatten)]
    pub tile: TileConfig,

    /// Network configuration (X-Plane connection, rate limiting)
    #[serde(flatten)]
    pub network: NetworkConfig,

    /// Cache configuration (disk and memory cache sizes)
    #[serde(flatten)]
    pub cache: CacheConfig,

    /// Flight configuration (SimBrief, prefetch, route parameters)
    #[serde(flatten)]
    pub flight: FlightConfig,

    /// Night exclusion configuration
    #[serde(flatten)]
    pub night: NightConfig,

    /// Seasonal configuration
    #[serde(flatten)]
    pub season_cfg: SeasonConfig,

    /// UI configuration (display, window, debug)
    #[serde(flatten)]
    pub ui: UiConfig,

    #[serde(default)]
    pub scenery_download_dir: String,
    #[serde(default = "default_simheaven_compat")]
    pub simheaven_compat: bool,

    /// Fallback configuration
    #[serde(default)]
    pub fallback: FallbackConfig,
}

impl Default for AutoOrthoConfig {
    fn default() -> Self {
        Self {
            xplane_path: default_xplane_path(),
            cache_dir: default_cache_dir(),
            tile: TileConfig::default(),
            network: NetworkConfig::default(),
            cache: CacheConfig::default(),
            flight: FlightConfig::default(),
            night: NightConfig::default(),
            season_cfg: SeasonConfig::default(),
            ui: UiConfig::default(),
            scenery_download_dir: Default::default(),
            simheaven_compat: default_simheaven_compat(),
            fallback: Default::default(),
        }
    }
}

impl AutoOrthoConfig {
    /// Calculate the number of DDS tiles that fit in the configured memory.
    pub fn dds_memory_cache_entries(&self) -> usize {
        self.cache.dds_memory_cache_entries()
    }

    /// Calculate the number of chunks that fit in the configured memory.
    pub fn chunk_memory_cache_entries(&self) -> usize {
        self.cache.chunk_memory_cache_entries()
    }

    /// Validate all config fields are within acceptable ranges.
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.tile.validate()?;
        self.network.validate()?;
        self.cache.validate()?;
        self.night.validate()?;
        self.ui.validate()?;
        Ok(())
    }

    /// Path to the config file.
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("autoortho");
        config_dir.join("config.toml")
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
                warn!("Failed to load config from {}: {}", path.display(), e);
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

    /// Clear saved window position/size (for --reset-window).
    pub fn reset_window_position(&mut self) {
        self.ui.reset_window_position();
    }

    /// Load from a specific file.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        toml::from_str(&content).map_err(|e| format!("Cannot parse {}: {}", path.display(), e))
    }
}

/// Snapshot of config fields commonly needed for prefetch/dynamic zoom.
#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub tile: TileConfig,
    pub night: NightConfig,
    pub flight: FlightConfig,
    pub cache: CacheConfig,
}

impl ConfigSnapshot {
    /// Calculate the number of DDS tiles that fit in the configured memory.
    pub fn dds_memory_cache_entries(&self) -> usize {
        self.cache.dds_memory_cache_entries()
    }

    /// Calculate the number of chunks that fit in the configured memory.
    pub fn chunk_memory_cache_entries(&self) -> usize {
        self.cache.chunk_memory_cache_entries()
    }
}

impl From<&AutoOrthoConfig> for ConfigSnapshot {
    fn from(config: &AutoOrthoConfig) -> Self {
        Self {
            tile: config.tile.clone(),
            night: config.night.clone(),
            flight: config.flight.clone(),
            cache: config.cache.clone(),
        }
    }
}

fn default_xplane_path() -> String {
    dirs::home_dir()
        .map(|p| p.join("X-Plane 12").to_string_lossy().into_owned())
        .unwrap_or_else(|| "X-Plane 12".to_string())
}

fn default_cache_dir() -> String {
    dirs::cache_dir()
        .map(|p| p.join("autoortho").to_string_lossy().into_owned())
        .unwrap_or_else(|| "autoortho_cache".to_string())
}

fn default_simheaven_compat() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_config_in_temp;

    #[test]
    fn test_config_snapshot_dds_memory_cache_entries() {
        let config = AutoOrthoConfig::default();
        let snapshot: ConfigSnapshot = (&config).into();
        assert_eq!(snapshot.dds_memory_cache_entries(), 11);

        let mut config = AutoOrthoConfig::default();
        config.cache.dds_memory_cache_mb = 4096;
        let snapshot: ConfigSnapshot = (&config).into();
        assert_eq!(snapshot.dds_memory_cache_entries(), 186);

        let mut config = AutoOrthoConfig::default();
        config.cache.dds_memory_cache_mb = 0;
        let snapshot: ConfigSnapshot = (&config).into();
        assert_eq!(snapshot.dds_memory_cache_entries(), 1);
    }

    #[test]
    fn test_config_clone() {
        let config1 = AutoOrthoConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.network.xplane_host, config2.network.xplane_host);
    }

    #[test]
    fn test_save_and_load() {
        let (mut config, _tmp) = test_config_in_temp();
        let path = std::path::Path::new(&config.cache_dir).join("config.toml");

        config.tile.provider = "BI".to_string();
        config.network.xplane_port = 12345;
        config.scenery_download_dir = "/my/downloads".to_string();

        config.save_to(&path).unwrap();

        let loaded = AutoOrthoConfig::from_file(&path).unwrap();
        assert_eq!(loaded.tile.provider, "BI");
        assert_eq!(loaded.network.xplane_port, 12345);
        assert_eq!(loaded.scenery_download_dir, "/my/downloads");
    }

    #[test]
    fn test_load_missing_file_returns_error() {
        let result = AutoOrthoConfig::from_file(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_with_fallback() {
        let config = AutoOrthoConfig::load();
        assert_eq!(config.network.xplane_port, 49000);
    }

    #[test]
    fn test_config_path_not_empty() {
        let path = AutoOrthoConfig::config_path();
        assert!(path.to_string_lossy().contains("autoortho"));
    }

    #[test]
    fn test_config_validate_default() {
        let config = AutoOrthoConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_invalid_zoom() {
        let mut config = AutoOrthoConfig::default();
        config.tile.min_zoom = 25;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_log_rotation_valid() {
        let mut config = AutoOrthoConfig::default();
        config.ui.log_rotation = "daily".to_string();
        assert!(config.validate().is_ok());
        config.ui.log_rotation = "hourly".to_string();
        assert!(config.validate().is_ok());
        config.ui.log_rotation = "never".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_log_rotation_invalid() {
        let mut config = AutoOrthoConfig::default();
        config.ui.log_rotation = "invalid".to_string();
        assert!(config.validate().is_err());
        config.ui.log_rotation = "weekly".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_zoom_order() {
        let mut config = AutoOrthoConfig::default();
        config.tile.min_zoom = 15;
        config.tile.max_zoom = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_invalid_port() {
        let mut config = AutoOrthoConfig::default();
        config.network.xplane_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_debug_mode_default() {
        let config = AutoOrthoConfig::default();
        assert!(!config.ui.debug_mode);
    }

    #[test]
    fn test_debug_mode_set() {
        let mut config = AutoOrthoConfig::default();
        config.ui.debug_mode = true;
        assert!(config.ui.debug_mode);
    }

    #[test]
    fn test_debug_mode_serde() {
        let config = AutoOrthoConfig::default();
        assert!(!config.ui.debug_mode);
        let toml = toml::to_string(&config).unwrap();
        let config2: AutoOrthoConfig = toml::from_str(&toml).unwrap();
        assert!(!config2.ui.debug_mode);

        let mut config = AutoOrthoConfig::default();
        config.ui.debug_mode = true;
        let toml = toml::to_string(&config).unwrap();
        let config2: AutoOrthoConfig = toml::from_str(&toml).unwrap();
        assert!(config2.ui.debug_mode);
    }

    #[test]
    fn test_config_snapshot_from_config() {
        let config = AutoOrthoConfig::default();
        let snapshot: ConfigSnapshot = (&config).into();
        assert_eq!(snapshot.tile.provider, config.tile.provider);
        assert_eq!(snapshot.tile.max_zoom, config.tile.max_zoom);
        assert_eq!(snapshot.tile.zoom_rules.len(), config.tile.zoom_rules.len());
        assert_eq!(
            snapshot.tile.enable_dynamic_zoom,
            config.tile.enable_dynamic_zoom
        );
        assert_eq!(
            snapshot.night.enable_night_exclusion,
            config.night.enable_night_exclusion
        );
    }

    #[test]
    fn test_fallback_config_validate() {
        let fallback = FallbackConfig::default();
        assert!(fallback.validate().is_ok());
        let mut invalid = FallbackConfig::default();
        invalid.max_zoom_gap = 100;
        assert!(invalid.validate().is_err());
    }
}

#[cfg(test)]
mod tile_config_tests {
    use super::*;

    #[test]
    fn test_tile_config_defaults() {
        let config = AutoOrthoConfig::default();
        let tile = &config.tile;
        assert_eq!(tile.provider, "ARC");
        assert!(tile.min_zoom >= 1);
        assert!(tile.max_zoom <= 22);
        assert!(tile.min_zoom <= tile.max_zoom);
    }

    #[test]
    fn test_tile_config_validate_valid() {
        let tile = TileConfig {
            provider: "ARC".to_string(),
            min_zoom: 10,
            max_zoom: 18,
            enable_dynamic_zoom: false,
            zoom_rules: vec![],
        };
        assert!(tile.validate().is_ok());
    }

    #[test]
    fn test_tile_config_validate_invalid_zoom_order() {
        let tile = TileConfig {
            provider: "ARC".to_string(),
            min_zoom: 18,
            max_zoom: 10,
            enable_dynamic_zoom: false,
            zoom_rules: vec![],
        };
        assert!(tile.validate().is_err());
    }

    #[test]
    fn test_tile_config_validate_empty_provider() {
        let tile = TileConfig {
            provider: "".to_string(),
            min_zoom: 10,
            max_zoom: 18,
            enable_dynamic_zoom: false,
            zoom_rules: vec![],
        };
        assert!(tile.validate().is_err());
    }
}
