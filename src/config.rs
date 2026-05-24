// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use crate::errors::{validate_f32_range, validate_log_rotation, validate_range};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Season — owned by `seasons` module. Re-exported here for compat.
pub use crate::seasons::Season;

/// FallbackLevel and FallbackConfig — owned by `tiles::fallback` module.
/// Re-exported here for compat.
pub use crate::tiles::fallback::{FallbackConfig, FallbackLevel};

/// ZoomRule — owned by `tiles::zoom` module. Re-exported here for compat.
pub use crate::tiles::zoom::ZoomRule;

/// RateLimitConfig — owned by `errors` module. Re-exported here for compat.
pub use crate::errors::RateLimitConfig;

/// ConfigError — owned by `errors` module. Re-exported here for compat.
pub use crate::errors::ConfigError;

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
    #[serde(default = "default_use_simbrief_altitude")]
    pub use_simbrief_altitude: bool,
    #[serde(default = "default_simheaven_compat")]
    pub simheaven_compat: bool,
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
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub debug_mode: bool,
    #[serde(default = "default_log_rotation")]
    pub log_rotation: String, // "daily", "hourly", "never"
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

fn default_debug_mode() -> bool {
    false
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

fn default_log_rotation() -> String {
    "daily".to_string()
}

fn default_xplane_path() -> String {
    dirs::home_dir()
        .map(|p| p.join("X-Plane 12").to_string_lossy().into_owned())
        .unwrap_or_else(|| "X-Plane 12".to_string())
}

fn default_enable_dynamic_zoom() -> bool {
    true
}

fn default_use_simbrief_altitude() -> bool {
    true
}

fn default_simheaven_compat() -> bool {
    false
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

    /// Validate all config fields are within acceptable ranges.
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_range(self.xplane_port as u64, 1, 65535, "xplane_port")?;

        let min_zoom = self.min_zoom as u64;
        let max_zoom = self.max_zoom as u64;
        validate_range(min_zoom, 0, 21, "min_zoom")?;
        validate_range(max_zoom, 0, 21, "max_zoom")?;
        if self.min_zoom > self.max_zoom {
            return Err(ConfigError::FieldInvalid {
                field: "min_zoom".to_string(),
                message: format!(
                    "min_zoom ({}) > max_zoom ({})",
                    self.min_zoom, self.max_zoom
                ),
            });
        }

        // Night thresholds
        validate_f32_range(self.night_threshold, -90.0, 0.0, "night_threshold")?;
        validate_f32_range(self.day_threshold, -90.0, 90.0, "day_threshold")?;
        if self.night_threshold > self.day_threshold {
            return Err(ConfigError::FieldInvalid {
                field: "night_threshold".to_string(),
                message: "night_threshold must be <= day_threshold".to_string(),
            });
        }

        // UI scale
        if self.ui_scale < 0.5 || self.ui_scale > 1.5 {
            return Err(ConfigError::FieldOutOfRange {
                field: "ui_scale".to_string(),
                min: 50,
                max: 150,
                value: (self.ui_scale * 100.0) as u64,
            });
        }

        // Cache sizes
        validate_range(self.dds_cache_size_mb, 0, 102400, "dds_cache_size_mb")?;
        validate_range(self.dds_memory_cache_mb, 0, 4096, "dds_memory_cache_mb")?;
        validate_range(self.chunk_memory_cache_mb, 0, 4096, "chunk_memory_cache_mb")?;

        // Route settings
        validate_range(
            self.route_consideration_radius_nm as u64,
            0,
            500,
            "route_consideration_radius_nm",
        )?;
        validate_range(
            self.route_deviation_threshold_nm as u64,
            0,
            500,
            "route_deviation_threshold_nm",
        )?;
        validate_range(
            self.route_prefetch_radius_nm as u64,
            0,
            500,
            "route_prefetch_radius_nm",
        )?;
        validate_range(
            self.prefetch_route_percent as u64,
            0,
            100,
            "prefetch_route_percent",
        )?;
        validate_range(self.airport_radius_nm as u64, 0, 500, "airport_radius_nm")?;
        validate_range(self.near_airport_zoom as u64, 0, 21, "near_airport_zoom")?;

        // Saturation values
        validate_f32_range(self.spring_saturation, 0.0, 2.0, "spring_saturation")?;
        validate_f32_range(self.summer_saturation, 0.0, 2.0, "summer_saturation")?;
        validate_f32_range(self.autumn_saturation, 0.0, 2.0, "autumn_saturation")?;
        validate_f32_range(self.winter_saturation, 0.0, 2.0, "winter_saturation")?;

        // Fallback config
        self.fallback
            .validate()
            .map_err(|e| ConfigError::FieldInvalid {
                field: "fallback".to_string(),
                message: e,
            })?;

        // Rate limit config
        self.rate_limit.validate()?;

        // Log rotation
        validate_log_rotation(&self.log_rotation)?;

        // Zoom rules
        for (i, rule) in self.zoom_rules.iter().enumerate() {
            if rule.zoom_level > 21 {
                return Err(ConfigError::FieldOutOfRange {
                    field: format!("zoom_rules[{}].zoom_level", i),
                    min: 0,
                    max: 21,
                    value: rule.zoom_level as u64,
                });
            }
        }

        Ok(())
    }
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
            use_simbrief_altitude: true,
            simheaven_compat: false,
            zoom_rules: default_zoom_rules(),
            dds_memory_cache_mb: default_dds_memory_cache_mb(),
            chunk_memory_cache_mb: default_chunk_memory_cache_mb(),
            season: Season::Disabled,
            spring_saturation: 0.70,
            summer_saturation: 1.0,
            autumn_saturation: 0.80,
            winter_saturation: 0.55,
            fallback: FallbackConfig::default(),
            rate_limit: RateLimitConfig::default(),
            debug_mode: default_debug_mode(),
            log_rotation: default_log_rotation(),
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

/// Snapshot of config fields commonly needed for prefetch/dynamic zoom.
/// Cloned once from `AutoOrthoConfig` to avoid repeated `config.read()` lock + clone.
#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub tile_provider: String,
    pub max_zoom: u32,
    pub zoom_rules: Vec<ZoomRule>,
    pub enable_dynamic_zoom: bool,
    pub enable_night_exclusion: bool,
    pub night_threshold: f32,
    pub day_threshold: f32,
    pub simbrief_user_id: String,
    pub chunk_memory_cache_mb: u64,
    pub dds_memory_cache_mb: u64,
    pub prefetch_route_percent: u32,
    pub route_prefetch_radius_nm: u32,
    pub airport_radius_nm: u32,
    pub prefetch_airports: bool,
    pub use_simbrief_altitude: bool,
    pub route_consideration_radius_nm: u32,
}

impl ConfigSnapshot {
    /// Calculate the number of DDS tiles that fit in the configured memory.
    pub fn dds_memory_cache_entries(&self) -> usize {
        const DDS_TILE_SIZE_MB: u64 = 22; // Matches AutoOrthoConfig::DDS_TILE_SIZE_MB
        ((self.dds_memory_cache_mb / DDS_TILE_SIZE_MB).max(1)) as usize
    }

    /// Calculate the number of chunks that fit in the configured memory.
    pub fn chunk_memory_cache_entries(&self) -> usize {
        const CHUNK_SIZE_KB: u64 = 30; // Matches AutoOrthoConfig::CHUNK_SIZE_KB
        ((self.chunk_memory_cache_mb * 1024 / CHUNK_SIZE_KB).max(1)) as usize
    }
}

impl From<&AutoOrthoConfig> for ConfigSnapshot {
    fn from(config: &AutoOrthoConfig) -> Self {
        Self {
            tile_provider: config.tile_provider.clone(),
            max_zoom: config.max_zoom,
            zoom_rules: config.zoom_rules.clone(),
            enable_dynamic_zoom: config.enable_dynamic_zoom,
            enable_night_exclusion: config.enable_night_exclusion,
            night_threshold: config.night_threshold,
            day_threshold: config.day_threshold,
            simbrief_user_id: config.simbrief_user_id.clone(),
            chunk_memory_cache_mb: config.chunk_memory_cache_mb,
            dds_memory_cache_mb: config.dds_memory_cache_mb,
            prefetch_route_percent: config.prefetch_route_percent,
            route_prefetch_radius_nm: config.route_prefetch_radius_nm,
            airport_radius_nm: config.airport_radius_nm,
            prefetch_airports: config.prefetch_airports,
            use_simbrief_altitude: config.use_simbrief_altitude,
            route_consideration_radius_nm: config.route_consideration_radius_nm,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_config_in_temp;

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
        let (mut config, _tmp) = test_config_in_temp();
        let path = std::path::Path::new(&config.cache_dir).join("config.toml");

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
    fn test_config_validate_default() {
        let config = AutoOrthoConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_invalid_zoom() {
        let mut config = AutoOrthoConfig::default();
        config.min_zoom = 25;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_log_rotation_valid() {
        let mut config = AutoOrthoConfig::default();
        config.log_rotation = "daily".to_string();
        assert!(config.validate().is_ok());

        config.log_rotation = "hourly".to_string();
        assert!(config.validate().is_ok());

        config.log_rotation = "never".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_log_rotation_invalid() {
        let mut config = AutoOrthoConfig::default();
        config.log_rotation = "invalid".to_string();
        assert!(config.validate().is_err());

        config.log_rotation = "weekly".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_zoom_order() {
        let mut config = AutoOrthoConfig::default();
        config.min_zoom = 15;
        config.max_zoom = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_invalid_port() {
        let mut config = AutoOrthoConfig::default();
        config.xplane_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_debug_mode_default() {
        let config = AutoOrthoConfig::default();
        assert!(!config.debug_mode);
    }

    #[test]
    fn test_debug_mode_set() {
        let mut config = AutoOrthoConfig::default();
        config.debug_mode = true;
        assert!(config.debug_mode);
    }

    #[test]
    fn test_debug_mode_serde() {
        // Test default (false) serialization
        let config = AutoOrthoConfig::default();
        assert!(!config.debug_mode);
        let toml = toml::to_string(&config).unwrap();
        let config2: AutoOrthoConfig = toml::from_str(&toml).unwrap();
        assert!(!config2.debug_mode);

        // Test debug_mode = true roundtrip
        let mut config = AutoOrthoConfig::default();
        config.debug_mode = true;
        let toml = toml::to_string(&config).unwrap();
        let config2: AutoOrthoConfig = toml::from_str(&toml).unwrap();
        assert!(config2.debug_mode);
    }

    #[test]
    fn test_config_snapshot_from_config() {
        let config = AutoOrthoConfig::default();
        let snapshot: ConfigSnapshot = (&config).into();
        assert_eq!(snapshot.tile_provider, config.tile_provider);
        assert_eq!(snapshot.max_zoom, config.max_zoom);
        assert_eq!(snapshot.zoom_rules.len(), config.zoom_rules.len());
        assert_eq!(snapshot.enable_dynamic_zoom, config.enable_dynamic_zoom);
        assert_eq!(
            snapshot.enable_night_exclusion,
            config.enable_night_exclusion
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
// Test comment
