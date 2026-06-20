// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::{validate_f32_range, validate_range};

// Re-exports used by other modules
pub use crate::errors::ConfigError;
pub use crate::errors::RateLimitConfig;
pub use crate::seasons::Season;
pub use crate::tiles::fallback::{FallbackConfig, FallbackLevel};
pub use crate::tiles::zoom::ZoomRule;

// ---------------------------------------------------------------------------
// CacheConfig
// ---------------------------------------------------------------------------

/// Cache configuration — disk and memory cache sizes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheConfig {
    #[serde(default = "default_dds_cache_size_mb")]
    pub dds_cache_size_mb: u64,
    #[serde(default = "default_enable_dds_cache")]
    pub enable_dds_cache: bool,
    #[serde(default = "default_dds_memory_cache_mb")]
    pub dds_memory_cache_mb: u64,
    #[serde(default = "default_chunk_memory_cache_mb")]
    pub chunk_memory_cache_mb: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            dds_cache_size_mb: default_dds_cache_size_mb(),
            enable_dds_cache: default_enable_dds_cache(),
            dds_memory_cache_mb: default_dds_memory_cache_mb(),
            chunk_memory_cache_mb: default_chunk_memory_cache_mb(),
        }
    }
}

impl CacheConfig {
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

    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_range(self.dds_cache_size_mb, 0, 102400, "dds_cache_size_mb")?;
        Ok(())
    }
}

fn default_dds_cache_size_mb() -> u64 {
    4096
}

fn default_enable_dds_cache() -> bool {
    true
}

fn default_dds_memory_cache_mb() -> u64 {
    256
}

fn default_chunk_memory_cache_mb() -> u64 {
    512
}

// ---------------------------------------------------------------------------
// FlightConfig
// ---------------------------------------------------------------------------

/// Flight configuration — SimBrief, prefetch, and route parameters.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlightConfig {
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
    #[serde(default = "default_use_simbrief_altitude")]
    pub use_simbrief_altitude: bool,
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            simbrief_user_id: Default::default(),
            route_consideration_radius_nm: default_route_consideration_radius_nm(),
            route_deviation_threshold_nm: default_route_deviation_threshold_nm(),
            route_prefetch_radius_nm: default_route_prefetch_radius_nm(),
            prefetch_route_percent: default_prefetch_route_percent(),
            prefetch_airports: default_prefetch_airports(),
            airport_radius_nm: default_airport_radius_nm(),
            near_airport_zoom: default_near_airport_zoom(),
            use_simbrief_altitude: default_use_simbrief_altitude(),
        }
    }
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

fn default_near_airport_zoom() -> u32 {
    19
}

fn default_use_simbrief_altitude() -> bool {
    true
}

// ---------------------------------------------------------------------------
// NetworkConfig
// ---------------------------------------------------------------------------

/// Network configuration — X-Plane connection and rate limiting.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    #[serde(default = "default_xplane_host")]
    pub xplane_host: String,
    #[serde(default = "default_xplane_port")]
    pub xplane_port: u16,
    #[serde(default)]
    pub rate_limit: crate::errors::RateLimitConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            xplane_host: default_xplane_host(),
            xplane_port: default_xplane_port(),
            rate_limit: Default::default(),
        }
    }
}

impl NetworkConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_range(self.xplane_port as u64, 1, 65535, "xplane_port")?;
        Ok(())
    }
}

fn default_xplane_host() -> String {
    "127.0.0.1".to_string()
}

fn default_xplane_port() -> u16 {
    49000
}

// ---------------------------------------------------------------------------
// NightConfig
// ---------------------------------------------------------------------------

/// Night exclusion configuration — sun pitch thresholds.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NightConfig {
    #[serde(default = "default_enable_night_exclusion")]
    pub enable_night_exclusion: bool,
    #[serde(default = "default_night_threshold")]
    pub night_threshold: f32,
    #[serde(default = "default_day_threshold")]
    pub day_threshold: f32,
}

impl Default for NightConfig {
    fn default() -> Self {
        Self {
            enable_night_exclusion: default_enable_night_exclusion(),
            night_threshold: default_night_threshold(),
            day_threshold: default_day_threshold(),
        }
    }
}

impl NightConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_f32_range(self.night_threshold, -90.0, 0.0, "night_threshold")?;
        validate_f32_range(self.day_threshold, -90.0, 90.0, "day_threshold")?;
        if self.night_threshold > self.day_threshold {
            return Err(ConfigError::FieldInvalid {
                field: "night_threshold".to_string(),
                message: "night_threshold must be <= day_threshold".to_string(),
            });
        }
        Ok(())
    }
}

fn default_enable_night_exclusion() -> bool {
    true
}

fn default_night_threshold() -> f32 {
    -12.0
}

fn default_day_threshold() -> f32 {
    -10.0
}

// ---------------------------------------------------------------------------
// SeasonConfig
// ---------------------------------------------------------------------------

/// Seasonal configuration — current season and per-season saturation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeasonConfig {
    #[serde(default)]
    pub season: crate::seasons::Season,
    #[serde(default = "default_spring_saturation")]
    pub spring_saturation: f32,
    #[serde(default = "default_summer_saturation")]
    pub summer_saturation: f32,
    #[serde(default = "default_autumn_saturation")]
    pub autumn_saturation: f32,
    #[serde(default = "default_winter_saturation")]
    pub winter_saturation: f32,
}

impl Default for SeasonConfig {
    fn default() -> Self {
        Self {
            season: Default::default(),
            spring_saturation: default_spring_saturation(),
            summer_saturation: default_summer_saturation(),
            autumn_saturation: default_autumn_saturation(),
            winter_saturation: default_winter_saturation(),
        }
    }
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

// ---------------------------------------------------------------------------
// UiConfig
// ---------------------------------------------------------------------------

/// UI configuration — display, window, and debug settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
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
    #[serde(default)]
    pub debug_mode: bool,
    #[serde(default = "default_log_rotation")]
    pub log_rotation: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            ui_scale: default_ui_scale(),
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
            debug_mode: false,
            log_rotation: default_log_rotation(),
        }
    }
}

impl UiConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.ui_scale < 0.5 || self.ui_scale > 1.5 {
            return Err(ConfigError::FieldOutOfRange {
                field: "ui_scale".to_string(),
                min: 50,
                max: 150,
                value: (self.ui_scale * 100.0) as u64,
            });
        }
        validate_log_rotation(&self.log_rotation)?;
        Ok(())
    }

    /// Clear saved window position/size.
    pub fn reset_window_position(&mut self) {
        self.window_x = None;
        self.window_y = None;
        self.window_width = None;
        self.window_height = None;
    }
}

fn default_ui_scale() -> f64 {
    1.0
}

fn default_log_rotation() -> String {
    "daily".to_string()
}

fn validate_log_rotation(value: &str) -> Result<(), ConfigError> {
    match value {
        "daily" | "hourly" | "never" => Ok(()),
        _ => Err(ConfigError::FieldInvalid {
            field: "log_rotation".to_string(),
            message: format!(
                "Invalid log_rotation '{}'. Must be 'daily', 'hourly', or 'never'",
                value
            ),
        }),
    }
}

// ---------------------------------------------------------------------------
// TileConfig
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// AutoOrthoConfig
// ---------------------------------------------------------------------------

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

    /// Load config from the default path with env var overrides, falling back to defaults.
    ///
    /// Env vars use `AUTOORTHO_` prefix with `__` separator:
    /// - `AUTOORTHO_TILE__PROVIDER=BI`
    /// - `AUTOORTHO_CACHE__DDS_CACHE_SIZE_MB=8192`
    pub fn load() -> Self {
        let path = Self::config_path();
        let mut config = match Self::from_file(&path) {
            Ok(config) => {
                info!("Loaded config from {}", path.display());
                config
            }
            Err(e) => {
                warn!("Failed to load config from {}: {}", path.display(), e);
                Self::default()
            }
        };

        config.apply_env_overrides();
        config
    }

    /// Apply `AUTOORTHO_*` environment variable overrides.
    fn apply_env_overrides(&mut self) {
        fn env_str(name: &str) -> Option<String> {
            std::env::var(name).ok().filter(|v| !v.is_empty())
        }
        fn env_val<T: std::str::FromStr>(name: &str) -> Option<T> {
            env_str(name).and_then(|v| v.parse().ok())
        }

        if let Some(v) = env_str("AUTOORTHO_XPLANE_PATH") {
            self.xplane_path = v;
        }
        if let Some(v) = env_str("AUTOORTHO_CACHE_DIR") {
            self.cache_dir = v;
        }
        if let Some(v) = env_str("AUTOORTHO_TILE__PROVIDER") {
            self.tile.provider = v;
        }
        if let Some(v) = env_val("AUTOORTHO_TILE__MIN_ZOOM") {
            self.tile.min_zoom = v;
        }
        if let Some(v) = env_val("AUTOORTHO_TILE__MAX_ZOOM") {
            self.tile.max_zoom = v;
        }
        if let Some(v) = env_val("AUTOORTHO_TILE__ENABLE_DYNAMIC_ZOOM") {
            self.tile.enable_dynamic_zoom = v;
        }
        if let Some(v) = env_str("AUTOORTHO_NETWORK__XPLANE_HOST") {
            self.network.xplane_host = v;
        }
        if let Some(v) = env_val("AUTOORTHO_NETWORK__XPLANE_PORT") {
            self.network.xplane_port = v;
        }
        if let Some(v) = env_val("AUTOORTHO_CACHE__DDS_CACHE_SIZE_MB") {
            self.cache.dds_cache_size_mb = v;
        }
        if let Some(v) = env_val("AUTOORTHO_CACHE__ENABLE_DDS_CACHE") {
            self.cache.enable_dds_cache = v;
        }
        if let Some(v) = env_val("AUTOORTHO_CACHE__DDS_MEMORY_CACHE_MB") {
            self.cache.dds_memory_cache_mb = v;
        }
        if let Some(v) = env_val("AUTOORTHO_CACHE__CHUNK_MEMORY_CACHE_MB") {
            self.cache.chunk_memory_cache_mb = v;
        }
        if let Some(v) = env_str("AUTOORTHO_FLIGHT__SIMBRIEF_USER_ID") {
            self.flight.simbrief_user_id = v;
        }
        if let Some(v) = env_val("AUTOORTHO_FLIGHT__ROUTE_CONSIDERATION_RADIUS_NM") {
            self.flight.route_consideration_radius_nm = v;
        }
        if let Some(v) = env_val("AUTOORTHO_FLIGHT__ROUTE_DEVIATION_THRESHOLD_NM") {
            self.flight.route_deviation_threshold_nm = v;
        }
        if let Some(v) = env_val("AUTOORTHO_FLIGHT__ROUTE_PREFETCH_RADIUS_NM") {
            self.flight.route_prefetch_radius_nm = v;
        }
        if let Some(v) = env_val("AUTOORTHO_FLIGHT__PREFETCH_ROUTE_PERCENT") {
            self.flight.prefetch_route_percent = v;
        }
        if let Some(v) = env_val("AUTOORTHO_FLIGHT__PREFETCH_AIRPORTS") {
            self.flight.prefetch_airports = v;
        }
        if let Some(v) = env_val("AUTOORTHO_FLIGHT__AIRPORT_RADIUS_NM") {
            self.flight.airport_radius_nm = v;
        }
        if let Some(v) = env_val("AUTOORTHO_FLIGHT__NEAR_AIRPORT_ZOOM") {
            self.flight.near_airport_zoom = v;
        }
        if let Some(v) = env_val("AUTOORTHO_FLIGHT__USE_SIMBRIEF_ALTITUDE") {
            self.flight.use_simbrief_altitude = v;
        }
        if let Some(v) = env_val("AUTOORTHO_NIGHT__ENABLE_NIGHT_EXCLUSION") {
            self.night.enable_night_exclusion = v;
        }
        if let Some(v) = env_val("AUTOORTHO_NIGHT__NIGHT_THRESHOLD") {
            self.night.night_threshold = v;
        }
        if let Some(v) = env_val("AUTOORTHO_NIGHT__DAY_THRESHOLD") {
            self.night.day_threshold = v;
        }
        if let Some(v) = env_val("AUTOORTHO_SEASON_CFG__SPRING_SATURATION") {
            self.season_cfg.spring_saturation = v;
        }
        if let Some(v) = env_val("AUTOORTHO_SEASON_CFG__SUMMER_SATURATION") {
            self.season_cfg.summer_saturation = v;
        }
        if let Some(v) = env_val("AUTOORTHO_SEASON_CFG__AUTUMN_SATURATION") {
            self.season_cfg.autumn_saturation = v;
        }
        if let Some(v) = env_val("AUTOORTHO_SEASON_CFG__WINTER_SATURATION") {
            self.season_cfg.winter_saturation = v;
        }
        if let Some(v) = env_val("AUTOORTHO_UI__UI_SCALE") {
            self.ui.ui_scale = v;
        }
        if let Some(v) = env_val("AUTOORTHO_UI__DEBUG_MODE") {
            self.ui.debug_mode = v;
        }
        if let Some(v) = env_str("AUTOORTHO_UI__LOG_ROTATION") {
            self.ui.log_rotation = v;
        }
        if let Some(v) = env_str("AUTOORTHO_SCENERY_DOWNLOAD_DIR") {
            self.scenery_download_dir = v;
        }
        if let Some(v) = env_val("AUTOORTHO_SIMHEAVEN_COMPAT") {
            self.simheaven_compat = v;
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

// ---------------------------------------------------------------------------
// ConfigSnapshot
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_config_in_temp;

    // -- CacheConfig tests --

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert_eq!(config.dds_cache_size_mb, 4096);
        assert!(config.enable_dds_cache);
        assert_eq!(config.dds_memory_cache_mb, 256);
        assert_eq!(config.chunk_memory_cache_mb, 512);
    }

    #[test]
    fn test_dds_memory_cache_entries() {
        let config = CacheConfig::default();
        assert_eq!(config.dds_memory_cache_entries(), 11);
    }

    #[test]
    fn test_dds_memory_cache_entries_custom() {
        let mut config = CacheConfig::default();
        config.dds_memory_cache_mb = 4096;
        assert_eq!(config.dds_memory_cache_entries(), 186);
    }

    #[test]
    fn test_dds_memory_cache_entries_minimum() {
        let mut config = CacheConfig::default();
        config.dds_memory_cache_mb = 0;
        assert_eq!(config.dds_memory_cache_entries(), 1);
    }

    #[test]
    fn test_chunk_memory_cache_entries() {
        let config = CacheConfig::default();
        assert_eq!(config.chunk_memory_cache_entries(), 17476);
    }

    #[test]
    fn test_cache_config_validate_valid() {
        let config = CacheConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cache_config_validate_invalid() {
        let mut config = CacheConfig::default();
        config.dds_cache_size_mb = 200000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cache_config_serde_roundtrip() {
        let config = CacheConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: CacheConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.dds_cache_size_mb, loaded.dds_cache_size_mb);
    }

    // -- FlightConfig tests --

    #[test]
    fn test_flight_config_defaults() {
        let config = FlightConfig::default();
        assert!(config.simbrief_user_id.is_empty());
        assert_eq!(config.route_consideration_radius_nm, 50);
        assert_eq!(config.route_deviation_threshold_nm, 40);
        assert_eq!(config.route_prefetch_radius_nm, 40);
        assert_eq!(config.prefetch_route_percent, 20);
        assert!(config.prefetch_airports);
        assert_eq!(config.airport_radius_nm, 60);
        assert_eq!(config.near_airport_zoom, 19);
        assert!(config.use_simbrief_altitude);
    }

    #[test]
    fn test_flight_config_serde_roundtrip() {
        let config = FlightConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: FlightConfig = toml::from_str(&toml).unwrap();
        assert_eq!(
            config.route_consideration_radius_nm,
            loaded.route_consideration_radius_nm
        );
    }

    // -- NetworkConfig tests --

    #[test]
    fn test_network_config_defaults() {
        let config = NetworkConfig::default();
        assert_eq!(config.xplane_host, "127.0.0.1");
        assert_eq!(config.xplane_port, 49000);
    }

    #[test]
    fn test_network_config_validate_valid() {
        let config = NetworkConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_network_config_validate_invalid_port() {
        let mut config = NetworkConfig::default();
        config.xplane_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_network_config_validate_high_port() {
        let mut config = NetworkConfig::default();
        config.xplane_port = 65535;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_network_config_serde_roundtrip() {
        let config = NetworkConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: NetworkConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.xplane_host, loaded.xplane_host);
        assert_eq!(config.xplane_port, loaded.xplane_port);
    }

    // -- NightConfig tests --

    #[test]
    fn test_night_config_defaults() {
        let config = NightConfig::default();
        assert!(config.enable_night_exclusion);
        assert_eq!(config.night_threshold, -12.0);
        assert_eq!(config.day_threshold, -10.0);
    }

    #[test]
    fn test_night_config_validate_valid() {
        let config = NightConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_night_config_validate_invalid_order() {
        let mut config = NightConfig::default();
        config.night_threshold = 20.0;
        config.day_threshold = 10.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_night_config_validate_threshold_out_of_range() {
        let mut config = NightConfig::default();
        config.night_threshold = -100.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_night_config_serde_roundtrip() {
        let config = NightConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: NightConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.night_threshold, loaded.night_threshold);
    }

    // -- SeasonConfig tests --

    #[test]
    fn test_season_config_defaults() {
        let config = SeasonConfig::default();
        assert_eq!(config.spring_saturation, 0.70);
        assert_eq!(config.summer_saturation, 1.0);
        assert_eq!(config.autumn_saturation, 0.80);
        assert_eq!(config.winter_saturation, 0.55);
    }

    #[test]
    fn test_season_config_serde_roundtrip() {
        let config = SeasonConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: SeasonConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.spring_saturation, loaded.spring_saturation);
    }

    // -- UiConfig tests --

    #[test]
    fn test_ui_config_defaults() {
        let config = UiConfig::default();
        assert_eq!(config.ui_scale, 1.0);
        assert!(config.window_x.is_none());
        assert!(!config.debug_mode);
        assert_eq!(config.log_rotation, "daily");
    }

    #[test]
    fn test_ui_config_validate_valid() {
        let config = UiConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ui_config_validate_invalid_scale() {
        let mut config = UiConfig::default();
        config.ui_scale = 0.3;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ui_config_validate_invalid_log_rotation() {
        let mut config = UiConfig::default();
        config.log_rotation = "weekly".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ui_config_reset_window_position() {
        let mut config = UiConfig::default();
        config.window_x = Some(100.0);
        config.window_y = Some(200.0);
        config.window_width = Some(800.0);
        config.window_height = Some(600.0);
        config.reset_window_position();
        assert!(config.window_x.is_none());
        assert!(config.window_y.is_none());
        assert!(config.window_width.is_none());
        assert!(config.window_height.is_none());
    }

    #[test]
    fn test_ui_config_serde_roundtrip() {
        let config = UiConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: UiConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.ui_scale, loaded.ui_scale);
    }

    // -- TileConfig tests --

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

    // -- AutoOrthoConfig tests --

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
    fn test_load_env_var_override() {
        unsafe {
            std::env::set_var("AUTOORTHO_TILE__PROVIDER", "BI");
        }
        let config = AutoOrthoConfig::load();
        assert_eq!(config.tile.provider, "BI");
        unsafe {
            std::env::remove_var("AUTOORTHO_TILE__PROVIDER");
        }
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
