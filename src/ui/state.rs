use crate::config::AutoOrthoConfig;

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
    pub current_file: std::sync::Arc<std::sync::Mutex<String>>,
    /// Number of files completed / total files
    pub files_done: std::sync::Arc<std::sync::atomic::AtomicU32>,
    pub files_total: u32,
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

    pub fn current_filename(&self) -> String {
        self.current_file.lock().expect("progress mutex").clone()
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

    // Scenery management
    pub scenery_download_dir: String,
    pub scenery_install_dir: String,
    pub available_regions: Vec<SceneryRegionInfo>,
    pub installed_packs: Vec<InstalledPackInfo>,
    pub scenery_status: Option<String>,
    pub scenery_refreshing: bool,
    /// Region IDs currently being downloaded → (cancel token, progress)
    pub downloading_regions: std::collections::HashMap<String, DownloadState>,

    // Cache status
    pub dds_cache_size_bytes: u64,

    // SimBrief flight plan
    pub simbrief_fetching: bool,
    pub simbrief_route_summary: Option<String>,
    pub simbrief_fixes: Vec<(String, String, f32)>, // (ident, fix_type, altitude_ft) for display
    pub simbrief_show_details: bool,
    pub simbrief_error: Option<String>,
    pub simbrief_flight_plan: Option<crate::xplane::simbrief::FlightPlan>,
    pub simbrief_coverage_warning: Option<String>,

    // Developer test tile state
    pub test_tile_lat: String,
    pub test_tile_lon: String,
    pub test_tile_zoom: u32,
    pub test_tile_status: Option<String>,
    pub test_tile_running: bool,
    /// RGBA pixel data for the test tile preview (width, height, data)
    pub test_tile_image: Option<(u32, u32, Vec<u8>)>,
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

        let scenery_install_dir = config.scenery_install_dir().to_string_lossy().into_owned();

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
            available_regions: Vec::new(),
            installed_packs: Vec::new(),
            scenery_status: None,
            scenery_refreshing: false,
            downloading_regions: Default::default(),
            web_server: ServiceStatus::Stopped,
            web_server_url: None,
            xplane_tracker: ServiceStatus::Stopped,
            dds_cache_size_bytes: 0,
            simbrief_fetching: false,
            simbrief_route_summary: None,
            simbrief_fixes: Vec::new(),
            simbrief_show_details: false,
            simbrief_error: None,
            simbrief_flight_plan: None,
            simbrief_coverage_warning: None,
            test_tile_lat: String::new(),
            test_tile_lon: String::new(),
            test_tile_zoom: 10,
            test_tile_status: None,
            test_tile_running: false,
            test_tile_image: None,
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
        self.config
            .custom_scenery_path()
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
        self.scenery_install_dir = self
            .config
            .scenery_install_dir()
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

        if self.config.min_zoom >= self.config.max_zoom {
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
        state.config.min_zoom = 18;
        state.config.max_zoom = 10;
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
        state.config.min_zoom = 10;
        state.config.max_zoom = 18;
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
}
