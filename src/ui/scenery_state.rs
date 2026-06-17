// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Scenery management state.
//!
//! This module encapsulates the UI state for scenery pack discovery,
//! download, and installation.

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

/// Scenery management state.
#[derive(Debug, Clone)]
pub struct SceneryState {
    /// Directory where scenery downloads are saved
    pub download_dir: String,
    /// X-Plane Custom Scenery install directory
    pub install_dir: String,
    /// Cache data directory (where scenery files are stored)
    pub data_dir: String,
    /// Available regions for download
    pub available_regions: Vec<SceneryRegionInfo>,
    /// Installed scenery packs
    pub installed_packs: Vec<InstalledPackInfo>,
    /// Current scenery operation status message
    pub status: Option<String>,
    /// Whether a scenery refresh is in progress
    pub refreshing: bool,
    /// Active downloads: region_id → DownloadState
    pub downloading: std::collections::HashMap<String, DownloadState>,
}

impl Default for SceneryState {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneryState {
    pub fn new() -> Self {
        Self {
            download_dir: String::new(),
            install_dir: String::new(),
            data_dir: String::new(),
            available_regions: Vec::new(),
            installed_packs: Vec::new(),
            status: None,
            refreshing: false,
            downloading: std::collections::HashMap::new(),
        }
    }

    /// Create with initial directory paths.
    pub fn with_dirs(download_dir: String, install_dir: String, data_dir: String) -> Self {
        Self {
            download_dir,
            install_dir,
            data_dir,
            ..Self::new()
        }
    }

    /// Set the download directory.
    pub fn set_download_dir(&mut self, dir: String) {
        self.download_dir = dir;
    }

    /// Set the install directory.
    pub fn set_install_dir(&mut self, dir: String) {
        self.install_dir = dir;
    }

    /// Set the data directory.
    pub fn set_data_dir(&mut self, dir: String) {
        self.data_dir = dir;
    }

    /// Set the status message.
    pub fn set_status(&mut self, status: Option<String>) {
        self.status = status;
    }

    /// Start a refresh operation.
    pub fn start_refresh(&mut self) {
        self.refreshing = true;
        self.status = Some("Refreshing...".to_string());
    }

    /// Complete a refresh operation.
    pub fn complete_refresh(&mut self) {
        self.refreshing = false;
        self.status = None;
    }

    /// Set available regions.
    pub fn set_available_regions(&mut self, regions: Vec<SceneryRegionInfo>) {
        self.available_regions = regions;
    }

    /// Set installed packs.
    pub fn set_installed_packs(&mut self, packs: Vec<InstalledPackInfo>) {
        self.installed_packs = packs;
    }

    /// Start a download for a region.
    pub fn start_download(&mut self, region_id: String, state: DownloadState) {
        self.downloading.insert(region_id, state);
    }

    /// Remove a completed download.
    pub fn remove_download(&mut self, region_id: &str) {
        self.downloading.remove(region_id);
    }

    /// Whether any download is active.
    pub fn any_downloading(&self) -> bool {
        !self.downloading.is_empty()
    }

    /// Get download progress for a region.
    pub fn download_progress(&self, region_id: &str) -> Option<f32> {
        self.downloading
            .get(region_id)
            .map(|d| d.progress_percent())
    }

    /// Get total download count.
    pub fn download_count(&self) -> usize {
        self.downloading.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenery_state_creation() {
        let state = SceneryState::new();
        assert!(state.download_dir.is_empty());
        assert!(state.install_dir.is_empty());
        assert!(state.data_dir.is_empty());
        assert!(state.available_regions.is_empty());
        assert!(state.installed_packs.is_empty());
        assert!(!state.refreshing);
        assert!(!state.any_downloading());
    }

    #[test]
    fn test_with_dirs() {
        let state = SceneryState::with_dirs(
            "/downloads".to_string(),
            "/xplane/Custom Scenery".to_string(),
            "/cache/scenery".to_string(),
        );
        assert_eq!(state.download_dir, "/downloads");
        assert_eq!(state.install_dir, "/xplane/Custom Scenery");
        assert_eq!(state.data_dir, "/cache/scenery");
    }

    #[test]
    fn test_set_dirs() {
        let mut state = SceneryState::new();
        state.set_download_dir("/new/downloads".to_string());
        state.set_install_dir("/new/install".to_string());
        state.set_data_dir("/new/data".to_string());
        assert_eq!(state.download_dir, "/new/downloads");
        assert_eq!(state.install_dir, "/new/install");
        assert_eq!(state.data_dir, "/new/data");
    }

    #[test]
    fn test_refresh_lifecycle() {
        let mut state = SceneryState::new();
        assert!(!state.refreshing);

        state.start_refresh();
        assert!(state.refreshing);
        assert_eq!(state.status.as_deref(), Some("Refreshing..."));

        state.complete_refresh();
        assert!(!state.refreshing);
        assert!(state.status.is_none());
    }

    #[test]
    fn test_available_regions() {
        let mut state = SceneryState::new();
        let regions = vec![
            SceneryRegionInfo {
                id: "na".to_string(),
                name: "North America".to_string(),
                version: "1.0".to_string(),
                package_count: 10,
                total_size_bytes: 1_000_000_000,
                has_partial_download: false,
            },
            SceneryRegionInfo {
                id: "eu".to_string(),
                name: "Europe".to_string(),
                version: "1.0".to_string(),
                package_count: 8,
                total_size_bytes: 800_000_000,
                has_partial_download: true,
            },
        ];

        state.set_available_regions(regions);
        assert_eq!(state.available_regions.len(), 2);
        assert_eq!(state.available_regions[0].name, "North America");
        assert!(state.available_regions[1].has_partial_download);
    }

    #[test]
    fn test_installed_packs() {
        let mut state = SceneryState::new();
        let packs = vec![InstalledPackInfo {
            id: "na".to_string(),
            name: "North America".to_string(),
            version: "1.0".to_string(),
        }];

        state.set_installed_packs(packs);
        assert_eq!(state.installed_packs.len(), 1);
    }

    #[test]
    fn test_download_lifecycle() {
        let mut state = SceneryState::new();
        assert!(!state.any_downloading());
        assert_eq!(state.download_count(), 0);

        // Create a mock download state
        let download = DownloadState {
            cancel: tokio_util::sync::CancellationToken::new(),
            bytes_downloaded: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            total_bytes: 1_000_000,
            current_file: std::sync::Arc::new(parking_lot::Mutex::new("file1.zip".to_string())),
            files_done: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            files_total: 5,
            extract_files_done: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            extract_files_total: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            extracting: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pack_current: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            pack_total: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(1)),
        };

        state.start_download("na".to_string(), download);
        assert!(state.any_downloading());
        assert_eq!(state.download_count(), 1);
        assert!(state.download_progress("na").is_some());

        state.remove_download("na");
        assert!(!state.any_downloading());
        assert_eq!(state.download_count(), 0);
    }

    #[test]
    fn test_download_state_progress() {
        let bytes = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(500_000));
        let files_done = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(3));

        let download = DownloadState {
            cancel: tokio_util::sync::CancellationToken::new(),
            bytes_downloaded: bytes.clone(),
            total_bytes: 1_000_000,
            current_file: std::sync::Arc::new(parking_lot::Mutex::new("file.zip".to_string())),
            files_done: files_done.clone(),
            files_total: 6,
            extract_files_done: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            extract_files_total: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            extracting: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pack_current: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            pack_total: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(1)),
        };

        assert!((download.progress_percent() - 50.0).abs() < 0.1);
        assert!((download.downloaded_mb() - 0.476).abs() < 0.01);
        assert!((download.total_mb() - 0.953).abs() < 0.01);
        assert_eq!(download.files_completed(), 3);
    }
}
