pub use super::prefetch_state::{WaypointPrefetchProgress, WaypointPrefetchStatus};

use crate::config::AutoOrthoConfig;
use crate::scenery::paths::{
    custom_scenery_path, mount_dir, scenery_data_dir, scenery_install_dir,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Shared tile progress state for the status bar.
/// Updated by DdsFileSystem during tile generation, read by UI.
#[derive(Debug)]
pub struct TileProgress {
    /// Whether a tile is currently being generated
    pub active: AtomicBool,
    /// Current tile row
    pub row: AtomicU32,
    /// Current tile col
    pub col: AtomicU32,
    /// Current zoom level
    pub zoom: AtomicU32,
    /// Chunks decoded so far (out of 256)
    pub chunks_done: AtomicU32,
    /// Total chunks to fetch (256)
    pub chunks_total: AtomicU32,
    /// Provider name being used
    pub provider: parking_lot::Mutex<String>,
}

impl TileProgress {
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            row: AtomicU32::new(0),
            col: AtomicU32::new(0),
            zoom: AtomicU32::new(0),
            chunks_done: AtomicU32::new(0),
            chunks_total: AtomicU32::new(0),
            provider: parking_lot::Mutex::new(String::new()),
        }
    }

    pub fn start(&self, row: u32, col: u32, zoom: u32, provider: &str) {
        self.row.store(row, Ordering::Relaxed);
        self.col.store(col, Ordering::Relaxed);
        self.zoom.store(zoom, Ordering::Relaxed);
        self.chunks_done.store(0, Ordering::Relaxed);
        self.chunks_total.store(256, Ordering::Relaxed);
        *self.provider.lock() = provider.to_string();
        self.active.store(true, Ordering::Relaxed);
    }

    pub fn update_progress(&self, chunks_done: u32) {
        self.chunks_done.store(chunks_done, Ordering::Relaxed);
    }

    pub fn finish(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Get a display string for the current tile (e.g., "1234/5678 @ z14")
    pub fn tile_label(&self) -> String {
        if !self.active.load(Ordering::Relaxed) {
            return String::new();
        }
        let row = self.row.load(Ordering::Relaxed);
        let col = self.col.load(Ordering::Relaxed);
        let zoom = self.zoom.load(Ordering::Relaxed);
        format!("{}/{}/z{}", row, col, zoom)
    }

    /// Get progress as a value 0.0-1.0
    pub fn progress(&self) -> f32 {
        let total = self.chunks_total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let done = self.chunks_done.load(Ordering::Relaxed);
        done as f32 / total as f32
    }
}

impl Default for TileProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    SetupWizard,
    Dashboard,
    Settings,
    About,
    Developer,
    Scenery,
}

pub use super::scenery_state::{
    DownloadState, InstalledPackInfo, SceneryRegionInfo, SharedProgress,
};
pub use super::service_state::ServiceStatus;

/// Application state management (elm-inspired)
#[derive(Clone)]
pub struct AppState {
    pub current_screen: Screen,
    pub config: AutoOrthoConfig,
    pub is_configured: bool,
    pub error_message: Option<String>,

    // Backend service status
    pub services: crate::ui::service_state::ServiceState,

    // X-Plane dataref tracker for checking connection status
    pub tracker: Option<crate::xplane::Tracker>,

    // Scenery management
    pub scenery: crate::ui::scenery_state::SceneryState,

    // Cache status
    pub dds_cache_size_bytes: u64,
    pub dds_cache: Option<std::sync::Arc<parking_lot::Mutex<crate::pipeline::cache::DdsCache>>>,

    // SimBrief flight plan
    pub simbrief_fetching: bool,
    pub simbrief_route_summary: Option<String>,
    pub simbrief_fixes: Vec<(String, String, f32)>, // (ident, fix_type, altitude_ft) for display
    pub simbrief_show_details: bool,
    pub simbrief_error: Option<String>,
    pub simbrief_flight_plan: Option<crate::xplane::simbrief::FlightPlan>,
    pub simbrief_coverage_warning: Option<String>,

    // Developer test tile and fallback test state
    pub dev_test: crate::ui::dev_test_state::DevTestState,

    // Route prefetch state
    pub prefetch: crate::ui::prefetch_state::PrefetchState,

    // Tile progress (shared with DdsFileSystem)
    pub tile_progress: Arc<TileProgress>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("current_screen", &self.current_screen)
            .field("config", &self.config)
            .field("is_configured", &self.is_configured)
            .field("error_message", &self.error_message)
            .field("services", &self.services)
            .field("tracker", &self.tracker.as_ref().map(|_| "..."))
            .field("scenery", &self.scenery)
            .field("dds_cache_size_bytes", &self.dds_cache_size_bytes)
            .field("dds_cache", &self.dds_cache.as_ref().map(|_| "..."))
            .field("simbrief_fetching", &self.simbrief_fetching)
            .field("simbrief_route_summary", &self.simbrief_route_summary)
            .field("prefetch", &self.prefetch)
            .field("tile_progress", &self.tile_progress)
            .finish()
    }
}

impl AppState {
    pub fn new() -> Self {
        let config = AutoOrthoConfig::load();
        let is_configured = !config.xplane_path.is_empty();

        let scenery_download_dir = if config.scenery_download_dir.is_empty() {
            dirs::download_dir()
                .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
                .map(|p| p.join("autoortho-scenery").to_string_lossy().into_owned())
                .unwrap_or_else(|| "downloads".to_string())
        } else {
            config.scenery_download_dir.clone()
        };

        let scenery_install_dir = scenery_install_dir(&config.xplane_path)
            .to_string_lossy()
            .into_owned();
        let scenery_data_dir = scenery_data_dir(&config.cache_dir)
            .to_string_lossy()
            .into_owned();

        Self {
            current_screen: if is_configured {
                Screen::Dashboard
            } else {
                Screen::Welcome
            },
            config,
            is_configured,
            error_message: None,
            scenery: crate::ui::scenery_state::SceneryState::with_dirs(
                scenery_download_dir,
                scenery_install_dir,
                scenery_data_dir,
            ),
            services: crate::ui::service_state::ServiceState::new(),
            tracker: None,
            dds_cache_size_bytes: 0,
            dds_cache: None,
            simbrief_fetching: false,
            simbrief_route_summary: None,
            simbrief_fixes: Vec::new(),
            simbrief_show_details: false,
            simbrief_error: None,
            simbrief_flight_plan: None,
            simbrief_coverage_warning: None,
            dev_test: crate::ui::dev_test_state::DevTestState::new(),
            prefetch: crate::ui::prefetch_state::PrefetchState::new(),
            tile_progress: Arc::new(TileProgress::new()),
        }
    }

    /// Run one-time migration of scenery files from the old location
    /// (`{xplane}/Custom Scenery/z_autoortho/`) to the new data directory.
    /// This is called during app startup, not in `new()`, to avoid side
    /// effects in unit tests.
    pub fn run_startup_migration(&self) {
        let config = crate::config::AutoOrthoConfig::load();
        let old_dir = mount_dir(&config.xplane_path);
        let new_dir = std::path::Path::new(&self.scenery.data_dir);
        match crate::scenery::installer::migrate_scenery(&old_dir, new_dir) {
            Ok(count) if count > 0 => {
                log::info!(
                    "Migrated {} items from old scenery location to {}",
                    count,
                    self.scenery.data_dir
                );
            }
            Ok(_) => {}
            Err(e) => {
                log::warn!("Failed to migrate scenery files: {}", e);
            }
        }
    }

    /// Whether any backend service is running
    pub fn any_service_running(&self) -> bool {
        self.services.any_running()
    }

    /// Whether the scenery install directory looks like X-Plane's Custom Scenery folder
    pub fn scenery_dir_valid(&self) -> bool {
        if self.config.xplane_path.is_empty() {
            return false;
        }
        custom_scenery_path(&self.config.xplane_path)
            .join("scenery_packs.ini")
            .exists()
    }

    /// Persist configuration to file
    pub fn save_config(&mut self) {
        // Sync scenery download dir into config before saving
        self.config.scenery_download_dir = self.scenery.download_dir.clone();

        match self.config.save() {
            Ok(()) => {
                self.is_configured = true;
                self.error_message = None;
            }
            Err(e) => {
                self.set_error(format!("Failed to save config: {}", e));
            }
        }
    }

    /// Load configuration from file
    pub fn load_config(&mut self) {
        self.config = AutoOrthoConfig::load();
        self.scenery
            .set_download_dir(self.config.scenery_download_dir.clone());
        self.scenery.set_install_dir(
            scenery_install_dir(&self.config.xplane_path)
                .to_string_lossy()
                .into_owned(),
        );
        self.is_configured = true;
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn validate_config(&mut self) -> bool {
        self.clear_error();

        if self.config.xplane_path.is_empty() {
            self.set_error("X-Plane path cannot be empty".to_string());
            return false;
        }

        if self.config.tile.min_zoom >= self.config.tile.max_zoom {
            self.set_error("Min zoom must be less than max zoom".to_string());
            return false;
        }

        if self.config.network.xplane_port == 0 {
            self.set_error("X-Plane port must be greater than 0".to_string());
            return false;
        }

        true
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(
            state.current_screen == Screen::Dashboard || state.current_screen == Screen::Welcome
        );
        assert_eq!(state.services.web_server, ServiceStatus::Stopped);
        assert_eq!(state.services.xplane_tracker, ServiceStatus::Stopped);
    }

    #[test]
    fn test_screen_navigation() {
        let mut state = AppState::new();
        state.current_screen = Screen::Settings;
        assert_eq!(state.current_screen, Screen::Settings);
    }

    #[test]
    fn test_error_message() {
        let mut state = AppState::new();
        state.set_error("Test error".to_string());
        assert_eq!(state.error_message, Some("Test error".to_string()));

        state.clear_error();
        assert_eq!(state.error_message, None);
    }

    #[test]
    fn test_validate_xplane_path_empty() {
        let mut state = AppState::new();
        state.config.xplane_path = String::new();
        assert!(!state.validate_config());
        assert!(state.error_message.is_some());
    }

    #[test]
    fn test_validate_zoom_levels() {
        let mut state = AppState::new();
        state.config.tile.min_zoom = 18;
        state.config.tile.max_zoom = 10;
        assert!(!state.validate_config());
    }

    #[test]
    fn test_validate_xplane_port() {
        let mut state = AppState::new();
        state.config.network.xplane_port = 0;
        assert!(!state.validate_config());
    }

    #[test]
    fn test_validate_config_success() {
        let mut state = AppState::new();
        state.config.xplane_path = "/home/user/X-Plane 12".to_string();
        state.config.tile.min_zoom = 10;
        state.config.tile.max_zoom = 18;
        state.config.network.xplane_port = 49000;
        assert!(state.validate_config());
        assert_eq!(state.error_message, None);
    }

    #[test]
    fn test_service_status() {
        assert!(!ServiceStatus::Stopped.is_running());
        assert!(ServiceStatus::Running.is_running());
        assert_eq!(ServiceStatus::Running.label(), "Running");
    }

    #[test]
    fn test_any_service_running() {
        let mut state = AppState::new();
        assert!(!state.any_service_running());

        state.services.set_web_server(ServiceStatus::Running);
        assert!(state.any_service_running());
    }

    #[test]
    fn test_prefetch_state_defaults() {
        let state = AppState::new();
        assert!(!state.prefetch.running);
        assert!(state.prefetch.status.is_none());
        assert_eq!(state.prefetch.completed, 0);
        assert_eq!(state.prefetch.total, 0);
    }
}
