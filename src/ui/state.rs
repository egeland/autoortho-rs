use crate::config::AutoOrthoConfig;
use crate::scenery::paths::{
    custom_scenery_path, mount_dir, scenery_data_dir, scenery_install_dir,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Prefetch status for a single waypoint/fix in the flight plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointPrefetchStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

impl WaypointPrefetchStatus {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::NotStarted => "⏳",
            Self::InProgress => "🔄",
            Self::Completed => "✅",
            Self::Failed => "❌",
        }
    }
}

/// Shared waypoint prefetch progress (read by UI, written by background task).
#[derive(Debug, Default)]
pub struct WaypointPrefetchProgress {
    statuses: parking_lot::Mutex<Vec<WaypointPrefetchStatus>>,
}

impl WaypointPrefetchProgress {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn init(&self, count: usize) {
        *self.statuses.lock() = vec![WaypointPrefetchStatus::NotStarted; count];
    }
    pub fn set(&self, index: usize, status: WaypointPrefetchStatus) {
        let mut s = self.statuses.lock();
        if index < s.len() {
            s[index] = status;
        }
    }
    pub fn get_all(&self) -> Vec<WaypointPrefetchStatus> {
        self.statuses.lock().clone()
    }
}

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

/// Runtime service status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Error,
}

impl ServiceStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Stopped => "Stopped",
            Self::Starting => "Starting...",
            Self::Running => "Running",
            Self::Error => "Error",
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

/// Scenery region available for download (UI-friendly clone).
#[derive(Debug, Clone)]
pub struct SceneryRegionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub package_count: usize,
    pub total_size_bytes: u64,
    /// Whether partial .tmp download files exist for this region
    pub has_partial_download: bool,
}

/// Installed scenery pack info (UI-friendly clone).
#[derive(Debug, Clone)]
pub struct InstalledPackInfo {
    pub id: String,
    pub name: String,
    pub version: String,
}

/// Shared progress counter for a download (thread-safe).
pub type SharedProgress = std::sync::Arc<std::sync::atomic::AtomicU64>;

/// State of an active download.
#[derive(Debug, Clone)]
pub struct DownloadState {
    pub cancel: tokio_util::sync::CancellationToken,
    /// Bytes downloaded so far (atomic, updated by download task)
    pub bytes_downloaded: SharedProgress,
    /// Total bytes expected
    pub total_bytes: u64,
    /// Current file being downloaded
    pub current_file: std::sync::Arc<parking_lot::Mutex<String>>,
    /// Number of files completed / total files
    pub files_done: std::sync::Arc<std::sync::atomic::AtomicU32>,
    pub files_total: u32,
    /// Extraction progress - files extracted / total files in current zip
    pub extract_files_done: std::sync::Arc<std::sync::atomic::AtomicU32>,
    pub extract_files_total: std::sync::Arc<std::sync::atomic::AtomicU32>,
    /// Whether extraction is in progress
    pub extracting: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Pack progress - current pack number / total packs
    pub pack_current: std::sync::Arc<std::sync::atomic::AtomicU32>,
    pub pack_total: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl DownloadState {
    pub fn progress_percent(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        let downloaded = self
            .bytes_downloaded
            .load(std::sync::atomic::Ordering::Relaxed);
        (downloaded as f64 / self.total_bytes as f64 * 100.0) as f32
    }

    pub fn downloaded_mb(&self) -> f64 {
        let downloaded = self
            .bytes_downloaded
            .load(std::sync::atomic::Ordering::Relaxed);
        downloaded as f64 / 1_048_576.0
    }

    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / 1_048_576.0
    }

    pub fn files_completed(&self) -> u32 {
        self.files_done.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn extract_progress_percent(&self) -> f32 {
        let total = self
            .extract_files_total
            .load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let done = self
            .extract_files_done
            .load(std::sync::atomic::Ordering::Relaxed);
        (done as f32 / total as f32) * 100.0
    }

    pub fn is_extracting(&self) -> bool {
        self.extracting.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn extract_files_completed(&self) -> u32 {
        self.extract_files_done
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn extract_files_total(&self) -> u32 {
        self.extract_files_total
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn pack_progress_percent(&self) -> f32 {
        let total = self.pack_total.load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let current = self.pack_current.load(std::sync::atomic::Ordering::Relaxed);
        (current as f32 / total as f32) * 100.0
    }

    pub fn pack_current(&self) -> u32 {
        self.pack_current.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn pack_total(&self) -> u32 {
        self.pack_total.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn current_filename(&self) -> String {
        self.current_file.lock().clone()
    }
}

/// Application state management (elm-inspired)
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_screen: Screen,
    pub config: AutoOrthoConfig,
    pub is_configured: bool,
    pub error_message: Option<String>,

    // Runtime service status
    pub web_server: ServiceStatus,
    pub web_server_url: Option<String>,
    pub xplane_tracker: ServiceStatus,

    // X-Plane dataref tracker for checking connection status
    pub tracker: Option<std::sync::Arc<crate::xplane::dataref::DatarefTracker>>,

    // Scenery management
    pub scenery_download_dir: String,
    pub scenery_install_dir: String,
    pub scenery_data_dir: String,
    pub available_regions: Vec<SceneryRegionInfo>,
    pub installed_packs: Vec<InstalledPackInfo>,
    pub scenery_status: Option<String>,
    pub scenery_refreshing: bool,
    /// Region IDs currently being downloaded → (cancel token, progress)
    pub downloading_regions: std::collections::HashMap<String, DownloadState>,

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
    pub prefetch_running: bool,
    pub prefetch_status: Option<String>,
    pub prefetch_completed: u32,
    pub prefetch_total: u32,
    pub prefetch_cancel: Option<tokio_util::sync::CancellationToken>,
    /// Shared progress tracker (written by background task, read by Tick)
    pub waypoint_prefetch_progress: Arc<WaypointPrefetchProgress>,
    /// Snapshot of per-waypoint status for display in view
    pub prefetch_waypoint_status: Vec<WaypointPrefetchStatus>,

    // Tile progress (shared with DdsFileSystem)
    pub tile_progress: Arc<TileProgress>,
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
            scenery_download_dir,
            scenery_install_dir,
            scenery_data_dir,
            available_regions: Vec::new(),
            installed_packs: Vec::new(),
            scenery_status: None,
            scenery_refreshing: false,
            downloading_regions: Default::default(),
            web_server: ServiceStatus::Stopped,
            web_server_url: None,
            xplane_tracker: ServiceStatus::Stopped,
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
            prefetch_running: false,
            prefetch_status: None,
            prefetch_completed: 0,
            prefetch_total: 0,
            prefetch_cancel: None,
            waypoint_prefetch_progress: Arc::new(WaypointPrefetchProgress::new()),
            prefetch_waypoint_status: Vec::new(),
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
        let new_dir = std::path::Path::new(&self.scenery_data_dir);
        match crate::scenery::installer::migrate_scenery(&old_dir, new_dir) {
            Ok(count) if count > 0 => {
                log::info!(
                    "Migrated {} items from old scenery location to {}",
                    count,
                    self.scenery_data_dir
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
        self.web_server.is_running() || self.xplane_tracker.is_running()
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
        self.config.scenery_download_dir = self.scenery_download_dir.clone();

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
        self.scenery_download_dir = self.config.scenery_download_dir.clone();
        self.scenery_install_dir = scenery_install_dir(&self.config.xplane_path)
            .to_string_lossy()
            .into_owned();
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

        if self.config.xplane_port == 0 {
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
        assert_eq!(state.web_server, ServiceStatus::Stopped);
        assert_eq!(state.xplane_tracker, ServiceStatus::Stopped);
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
        state.config.xplane_port = 0;
        assert!(!state.validate_config());
    }

    #[test]
    fn test_validate_config_success() {
        let mut state = AppState::new();
        state.config.xplane_path = "/home/user/X-Plane 12".to_string();
        state.config.tile.min_zoom = 10;
        state.config.tile.max_zoom = 18;
        state.config.xplane_port = 49000;
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

        state.web_server = ServiceStatus::Running;
        assert!(state.any_service_running());
    }

    #[test]
    fn test_prefetch_state_defaults() {
        let state = AppState::new();
        assert!(!state.prefetch_running);
        assert!(state.prefetch_status.is_none());
        assert_eq!(state.prefetch_completed, 0);
        assert_eq!(state.prefetch_total, 0);
    }
}
