// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use iced::{Element, Font, Subscription, Task};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::{oneshot, watch};

/// Saved window geometry to restore on boot: (x, y, width, height)
#[allow(clippy::type_complexity)]
static SAVED_WINDOW_GEOM: std::sync::Mutex<
    Option<(Option<f32>, Option<f32>, Option<f32>, Option<f32>)>,
> = std::sync::Mutex::new(None);

/// Whether saved geometry exists (checked by new() before GEOM is consumed)
static HAS_SAVED_GEOM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Embedded FiraCode Nerd Font — includes thousands of icons + programming ligatures
const FIRA_CODE_NERD: &[u8] = include_bytes!("../../assets/fonts/FiraCodeNerdFont-Regular.ttf");

/// The Font descriptor for our bundled Nerd Font
const NERD_FONT: Font = Font {
    family: iced::font::Family::Name("FiraCode Nerd Font"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

pub mod helpers;
pub mod screens;
pub mod state;

use state::{AppState, Screen, ServiceStatus};

/// Desktop UI application using iced (elm-inspired MVU architecture)
pub struct AutoOrthoApp {
    state: AppState,
    /// Dedicated tokio runtime for backend services
    runtime: Arc<tokio::runtime::Runtime>,
    /// Shutdown signal sender — drop or send true to stop services
    shutdown_tx: Option<watch::Sender<bool>>,
    /// Ignore window move/resize events until this instant has passed
    window_events_locked_until: Option<std::time::Instant>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Screen navigation
    GoToScreen(Screen),

    // Setup wizard messages
    SetXPlanePath(String),
    SetCacheDir(String),
    SetXPlaneHost(String),
    SetXPlanePort(String),
    SetTileProvider(String),
    SetMinZoom(u32),
    SetMaxZoom(u32),
    SetUIScale(f64),

    // Configuration persistence
    SaveConfiguration,
    LoadConfiguration,

    // Runtime control
    StartServices,
    StopServices,
    ServicesStarted(String), // web URL
    ServicesFailed(String),

    // Scenery management
    SetSceneryDownloadDir(String),
    RefreshAvailableRegions,
    RegionsLoaded(Vec<state::SceneryRegionInfo>),
    RegionsLoadFailed(String),
    DownloadRegion(String),           // region ID
    CancelDownload(String),           // region ID
    CleanRegionDownloads(String),     // region ID — delete all files to allow fresh install
    UninstallRegion(String),          // region ID
    DownloadComplete(String, String), // (region_id, status message)
    DownloadFailed(String, String),   // (region_id, error message)

    // Developer tools
    SetTestLat(String),
    SetTestLon(String),
    SetTestZoom(u32),
    FetchTestTile,
    TestTileComplete(String, Option<(u32, u32, Vec<u8>)>), // status, image RGBA
    TestTileFailed(String),

    // Folder pickers
    BrowseXPlanePath,
    BrowseCacheDir,
    BrowseSceneryDownloadDir,
    FolderPicked(String, String), // (field_name, path)

    // Window events
    WindowOpened(iced::window::Id),
    WindowRestoreSize(iced::window::Id, f32, f32),
    WindowRestorePosition,
    WindowMoved(iced::Point),
    WindowResized(iced::Size),
    WindowCloseRequested,

    // Cache management
    SetDdsCacheSizeMb(u64),
    SetEnableDdsCache(bool),
    ClearDdsCache,

    // UI refresh
    Tick,

    // Actions
    OpenMapInBrowser,
    OpenCustomMapEditor,
    OpenWebUI,
    Exit,
}

impl AutoOrthoApp {
    fn new() -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        let has_saved_geom = HAS_SAVED_GEOM.load(std::sync::atomic::Ordering::Relaxed);
        Self {
            state: AppState::new(),
            runtime: Arc::new(runtime),
            shutdown_tx: None,
            window_events_locked_until: if has_saved_geom {
                // Lock events for 2 seconds to let the restore settle
                Some(std::time::Instant::now() + std::time::Duration::from_secs(2))
            } else {
                None
            },
        }
    }

    fn title(&self) -> String {
        let status = if self.state.any_service_running() {
            " [Running]"
        } else {
            ""
        };
        format!("AutoOrtho - Satellite Imagery for X-Plane{}", status)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GoToScreen(screen) => {
                self.state.current_screen = screen;
            }
            Message::SetXPlanePath(path) => {
                self.state.config.xplane_path = path;
                self.state.scenery_install_dir = self
                    .state
                    .config
                    .scenery_install_dir()
                    .to_string_lossy()
                    .into_owned();
            }
            Message::SetCacheDir(dir) => {
                self.state.config.cache_dir = dir;
            }
            Message::SetXPlaneHost(host) => {
                self.state.config.xplane_host = host;
            }
            Message::SetXPlanePort(port_str) => {
                if let Ok(port) = port_str.parse() {
                    self.state.config.xplane_port = port;
                }
            }
            Message::SetTileProvider(provider) => {
                self.state.config.tile_provider = provider;
            }
            Message::SetMinZoom(zoom) => {
                self.state.config.min_zoom = zoom;
            }
            Message::SetMaxZoom(zoom) => {
                self.state.config.max_zoom = zoom;
            }
            Message::SetUIScale(scale) => {
                self.state.config.ui_scale = scale;
            }
            Message::SetDdsCacheSizeMb(mb) => {
                self.state.config.dds_cache_size_mb = mb;
            }
            Message::SetEnableDdsCache(enabled) => {
                self.state.config.enable_dds_cache = enabled;
            }
            Message::ClearDdsCache => {
                let cache_dir = std::path::PathBuf::from(&self.state.config.cache_dir).join("dds");
                if cache_dir.exists() {
                    if let Err(e) = std::fs::remove_dir_all(&cache_dir) {
                        log::warn!("Failed to clear DDS cache: {}", e);
                        self.state.error_message = Some(format!("Failed to clear cache: {}", e));
                    } else {
                        log::info!("DDS cache cleared");
                        self.state.dds_cache_size_bytes = 0;
                    }
                }
            }
            Message::SaveConfiguration => {
                self.state.save_config();
            }
            Message::LoadConfiguration => {
                self.state.load_config();
            }
            Message::StartServices => {
                self.state.web_server = ServiceStatus::Starting;
                self.state.xplane_tracker = ServiceStatus::Starting;
                self.state.clear_error();

                // Create shutdown channel
                let (shutdown_tx, shutdown_rx) = watch::channel(false);
                self.shutdown_tx = Some(shutdown_tx);

                let xplane_host = self.state.config.xplane_host.clone();
                let xplane_port = self.state.config.xplane_port;

                let (result_tx, result_rx) = oneshot::channel();
                let rt = self.runtime.clone();

                rt.spawn(async move {
                    let result =
                        start_all_services(5847, &xplane_host, xplane_port, shutdown_rx).await;
                    let _ = result_tx.send(result);
                });

                return Task::perform(
                    async {
                        result_rx
                            .await
                            .unwrap_or(Err("Runtime channel closed".into()))
                    },
                    |result| match result {
                        Ok(url) => Message::ServicesStarted(url),
                        Err(e) => Message::ServicesFailed(e),
                    },
                );
            }
            Message::StopServices => {
                // Signal shutdown to all services
                if let Some(tx) = self.shutdown_tx.take() {
                    let _ = tx.send(true);
                }
                self.state.web_server = ServiceStatus::Stopped;
                self.state.web_server_url = None;
                self.state.xplane_tracker = ServiceStatus::Stopped;
            }
            Message::ServicesStarted(url) => {
                self.state.web_server = ServiceStatus::Running;
                self.state.web_server_url = Some(url);
                self.state.xplane_tracker = ServiceStatus::Running;
            }
            Message::ServicesFailed(err) => {
                self.state.web_server = ServiceStatus::Error;
                self.state.xplane_tracker = ServiceStatus::Error;
                self.state.set_error(format!("Failed to start: {}", err));
            }
            Message::SetSceneryDownloadDir(v) => {
                self.state.scenery_download_dir = v;
            }
            Message::RefreshAvailableRegions => {
                self.state.scenery_refreshing = true;
                self.state.scenery_status = Some("Fetching available regions...".to_string());

                let install_dir = self.state.scenery_install_dir.clone();
                let download_dir = self.state.scenery_download_dir.clone();
                let (tx, rx) = oneshot::channel();
                let rt = self.runtime.clone();

                rt.spawn(async move {
                    let result = fetch_regions_and_installed(&install_dir, &download_dir).await;
                    let _ = tx.send(result);
                });

                return Task::perform(
                    async { rx.await.unwrap_or(Err("Channel closed".into())) },
                    |result| match result {
                        Ok((regions, _installed)) => Message::RegionsLoaded(regions),
                        Err(e) => Message::RegionsLoadFailed(e),
                    },
                );
            }
            Message::RegionsLoaded(regions) => {
                self.state.scenery_refreshing = false;
                self.state.available_regions = regions;
                self.state.scenery_status = Some(format!(
                    "Found {} regions available for download",
                    self.state.available_regions.len()
                ));
                // Also refresh installed packs
                let packs = crate::scenery::installer::list_installed_packs(std::path::Path::new(
                    &self.state.scenery_install_dir,
                ));
                self.state.installed_packs = packs
                    .into_iter()
                    .map(|p| state::InstalledPackInfo {
                        id: p.id,
                        name: p.name,
                        version: p.ver,
                    })
                    .collect();
            }
            Message::RegionsLoadFailed(err) => {
                self.state.scenery_refreshing = false;
                self.state.scenery_status = Some(format!("Error: {}", err));
            }
            Message::DownloadRegion(region_id) => {
                // Calculate total size from available regions
                let total_bytes = self
                    .state
                    .available_regions
                    .iter()
                    .find(|r| r.id == region_id)
                    .map(|r| r.total_size_bytes)
                    .unwrap_or(0);
                let files_total = self
                    .state
                    .available_regions
                    .iter()
                    .find(|r| r.id == region_id)
                    .map(|r| r.package_count as u32)
                    .unwrap_or(0);

                let cancel = tokio_util::sync::CancellationToken::new();
                let dl_state = state::DownloadState {
                    cancel: cancel.clone(),
                    bytes_downloaded: Arc::new(std::sync::atomic::AtomicU64::new(0)),
                    total_bytes,
                    current_file: Arc::new(std::sync::Mutex::new(String::new())),
                    files_done: Arc::new(std::sync::atomic::AtomicU32::new(0)),
                    files_total,
                };
                self.state
                    .downloading_regions
                    .insert(region_id.clone(), dl_state.clone());
                self.state.scenery_status = Some(format!("Downloading {}...", region_id));

                let download_dir = self.state.scenery_download_dir.clone();
                let install_dir = self.state.scenery_install_dir.clone();
                let rid = region_id.clone();
                let progress_bytes = dl_state.bytes_downloaded.clone();
                let progress_file = dl_state.current_file.clone();
                let progress_files_done = dl_state.files_done.clone();
                let (tx, rx) = oneshot::channel();
                let rt = self.runtime.clone();

                rt.spawn(async move {
                    let result = download_and_install_region(
                        &rid,
                        &download_dir,
                        &install_dir,
                        &cancel,
                        &progress_bytes,
                        &progress_file,
                        &progress_files_done,
                    )
                    .await;
                    let _ = tx.send((rid, result));
                });

                return Task::perform(
                    async {
                        rx.await
                            .unwrap_or(("unknown".into(), Err("Channel closed".into())))
                    },
                    |(rid, result)| match result {
                        Ok(msg) => Message::DownloadComplete(rid, msg),
                        Err(e) => Message::DownloadFailed(rid, e),
                    },
                );
            }
            Message::CancelDownload(region_id) => {
                if let Some(dl) = self.state.downloading_regions.get(&region_id) {
                    dl.cancel.cancel();
                }
                self.state.scenery_status = Some(format!(
                    "Cancelling {}... (partial files kept for resume)",
                    region_id
                ));
            }
            Message::CleanRegionDownloads(region_id) => {
                let download_dir = std::path::Path::new(&self.state.scenery_download_dir);
                match crate::scenery::installer::clean_downloads(download_dir, &region_id) {
                    Ok(bytes) => {
                        if let Some(r) = self
                            .state
                            .available_regions
                            .iter_mut()
                            .find(|r| r.id == region_id)
                        {
                            r.has_partial_download = false;
                        }
                        self.state.scenery_status = Some(format!(
                            "Cleaned {:.1} MB of downloads for {}",
                            bytes as f64 / 1_048_576.0,
                            region_id
                        ));
                    }
                    Err(e) => {
                        self.state.scenery_status = Some(format!("Clean failed: {}", e));
                    }
                }
            }
            Message::UninstallRegion(region_id) => {
                let install_dir = std::path::Path::new(&self.state.scenery_install_dir);
                match crate::scenery::installer::uninstall_region(&region_id, install_dir) {
                    Ok(()) => {
                        self.state.scenery_status = Some(format!("Uninstalled {}", region_id));
                        // Refresh installed list
                        let packs = crate::scenery::installer::list_installed_packs(install_dir);
                        self.state.installed_packs = packs
                            .into_iter()
                            .map(|p| state::InstalledPackInfo {
                                id: p.id,
                                name: p.name,
                                version: p.ver,
                            })
                            .collect();
                    }
                    Err(e) => {
                        self.state.scenery_status = Some(format!("Uninstall failed: {}", e));
                    }
                }
            }
            Message::DownloadComplete(region_id, msg) => {
                self.state.downloading_regions.remove(&region_id);
                self.state.scenery_status = Some(msg);
                let packs = crate::scenery::installer::list_installed_packs(std::path::Path::new(
                    &self.state.scenery_install_dir,
                ));
                self.state.installed_packs = packs
                    .into_iter()
                    .map(|p| state::InstalledPackInfo {
                        id: p.id,
                        name: p.name,
                        version: p.ver,
                    })
                    .collect();
            }
            Message::DownloadFailed(region_id, err) => {
                self.state.downloading_regions.remove(&region_id);
                if err.contains("Cancelled") {
                    // Mark as having partial download for Resume button
                    if let Some(r) = self
                        .state
                        .available_regions
                        .iter_mut()
                        .find(|r| r.id == region_id)
                    {
                        r.has_partial_download = true;
                    }
                    self.state.scenery_status = Some(format!(
                        "{} cancelled. Click Resume to continue, or Clean for a fresh start.",
                        region_id
                    ));
                } else {
                    self.state.scenery_status =
                        Some(format!("Error downloading {}: {}", region_id, err));
                }
            }
            Message::SetTestLat(v) => {
                // Check for preset format: "lat|lon|keep" or "lat|lon|14"
                if let Some((lat, rest)) = v.split_once('|') {
                    if let Some((lon, zoom_str)) = rest.split_once('|') {
                        self.state.test_tile_lat = lat.to_string();
                        self.state.test_tile_lon = lon.to_string();
                        // Only update zoom if it's a number (not "keep")
                        if let Ok(z) = zoom_str.parse::<u32>() {
                            self.state.test_tile_zoom = z;
                        }
                    }
                } else {
                    self.state.test_tile_lat = v;
                }
            }
            Message::SetTestLon(v) => {
                self.state.test_tile_lon = v;
            }
            Message::SetTestZoom(v) => {
                self.state.test_tile_zoom = v;
            }

            Message::FetchTestTile => {
                self.state.test_tile_running = true;
                self.state.test_tile_status = Some("Fetching...".to_string());

                let lat = self.state.test_tile_lat.parse::<f64>().unwrap_or(-33.86);
                let lon = self.state.test_tile_lon.parse::<f64>().unwrap_or(151.21);
                let zoom = self.state.test_tile_zoom;
                let provider_name = self.state.config.tile_provider.clone();

                let (tx, rx) = oneshot::channel();
                let rt = self.runtime.clone();

                rt.spawn(async move {
                    let result = fetch_test_tile(lat, lon, zoom, &provider_name).await;
                    let _ = tx.send(result);
                });

                return Task::perform(
                    async { rx.await.unwrap_or(Err("Channel closed".into())) },
                    |result| match result {
                        Ok((msg, img)) => Message::TestTileComplete(msg, img),
                        Err(e) => Message::TestTileFailed(e),
                    },
                );
            }
            Message::TestTileComplete(msg, image_data) => {
                self.state.test_tile_running = false;
                self.state.test_tile_status = Some(msg);
                self.state.test_tile_image = image_data;
            }
            Message::TestTileFailed(err) => {
                self.state.test_tile_running = false;
                self.state.test_tile_status = Some(format!("Error: {}", err));
                self.state.test_tile_image = None;
            }
            Message::BrowseXPlanePath => {
                return browse_folder("xplane_path", &self.state.config.xplane_path);
            }
            Message::BrowseCacheDir => {
                return browse_folder("cache_dir", &self.state.config.cache_dir);
            }
            Message::BrowseSceneryDownloadDir => {
                return browse_folder("scenery_download_dir", &self.state.scenery_download_dir);
            }
            Message::FolderPicked(field, path) => match field.as_str() {
                "xplane_path" => {
                    self.state.config.xplane_path = path;
                    self.state.scenery_install_dir = self
                        .state
                        .config
                        .scenery_install_dir()
                        .to_string_lossy()
                        .into_owned();
                }
                "cache_dir" => self.state.config.cache_dir = path,
                "scenery_download_dir" => self.state.scenery_download_dir = path,
                _ => {}
            },
            Message::WindowOpened(id) => {
                // Move window first, then schedule a delayed resize so the
                // window is on the correct monitor (correct DPI) before resizing.
                if let Some(geom) = SAVED_WINDOW_GEOM.lock().expect("lock").take() {
                    let has_pos = geom.0.is_some() && geom.1.is_some();
                    let has_size = geom.2.is_some() && geom.3.is_some();

                    let mut tasks: Vec<Task<Message>> = Vec::new();

                    if has_pos {
                        let (x, y) = (geom.0.unwrap(), geom.1.unwrap());
                        log::debug!("Restoring window position: ({}, {})", x, y);
                        tasks.push(iced::window::move_to::<Message>(id, iced::Point::new(x, y)));
                    }

                    if has_size {
                        let (w, h) = (geom.2.unwrap(), geom.3.unwrap());
                        // Delay the resize so the window lands on the target
                        // monitor first — otherwise iced uses the wrong DPI
                        tasks.push(Task::perform(
                            async {
                                // Wait for the move to settle and DPI to update
                                futures_lite::future::yield_now().await;
                                futures_lite::future::yield_now().await;
                                futures_lite::future::yield_now().await;
                            },
                            move |_| Message::WindowRestoreSize(id, w, h),
                        ));
                    }

                    if !tasks.is_empty() {
                        return Task::batch(tasks);
                    }
                }
                self.window_events_locked_until = None;
            }
            Message::WindowRestoreSize(id, w, h) => {
                log::debug!("Restoring window size: ({}, {})", w, h);
                return iced::window::resize::<Message>(id, iced::Size::new(w, h));
            }
            Message::WindowRestorePosition => {}
            Message::WindowMoved(pos) => {
                if self
                    .window_events_locked_until
                    .is_some_and(|t| std::time::Instant::now() < t)
                {
                    return Task::none();
                }
                self.window_events_locked_until = None;
                self.state.config.window_x = Some(pos.x);
                self.state.config.window_y = Some(pos.y);
                let _ = self.state.config.save();
            }
            Message::WindowResized(size) => {
                if self
                    .window_events_locked_until
                    .is_some_and(|t| std::time::Instant::now() < t)
                {
                    return Task::none();
                }
                self.window_events_locked_until = None;
                // Save size divided by UI scale so resize() can apply it cleanly
                let scale = self.state.config.ui_scale as f32;
                self.state.config.window_width = Some(size.width / scale);
                self.state.config.window_height = Some(size.height / scale);
                let _ = self.state.config.save();
            }
            Message::WindowCloseRequested => {
                log::info!(
                    "Window close: saving config with x={:?} y={:?} w={:?} h={:?}",
                    self.state.config.window_x,
                    self.state.config.window_y,
                    self.state.config.window_width,
                    self.state.config.window_height
                );
                let _ = self.state.config.save();
                if let Some(tx) = self.shutdown_tx.take() {
                    let _ = tx.send(true);
                }
                std::process::exit(0);
            }
            Message::Tick => {
                // Save config periodically when downloads are active (debounced window saves too)
                let _ = self.state.config.save();
            }
            Message::OpenMapInBrowser => {
                if let Some(ref url) = self.state.web_server_url {
                    let map_url = format!("{}/map", url);
                    let _ = open::that(&map_url);
                }
            }
            Message::OpenCustomMapEditor => {
                if let Some(ref url) = self.state.web_server_url {
                    let editor_url = format!("{}/custommap", url);
                    let _ = open::that(&editor_url);
                }
            }
            Message::OpenWebUI => {
                if let Some(ref url) = self.state.web_server_url {
                    let _ = open::that(url);
                }
            }
            Message::Exit => {
                let _ = self.state.config.save();
                if let Some(tx) = self.shutdown_tx.take() {
                    let _ = tx.send(true);
                }
                std::process::exit(0);
            }
        }
        Task::none()
    }

    fn scale_factor(&self) -> f32 {
        (self.state.config.ui_scale as f32).clamp(0.5, 2.0)
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            // Track window events
            iced::window::events().map(|(id, event)| match event {
                iced::window::Event::Opened { .. } => Message::WindowOpened(id),
                iced::window::Event::Moved(pos) => Message::WindowMoved(pos),
                iced::window::Event::Resized(size) => Message::WindowResized(size),
                iced::window::Event::CloseRequested => Message::WindowCloseRequested,
                _ => Message::Tick,
            }),
        ];

        // Tick every 500ms while downloads are active to refresh progress bars
        if !self.state.downloading_regions.is_empty() {
            subs.push(
                iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Tick),
            );
        }

        Subscription::batch(subs)
    }

    fn view(&self) -> Element<'_, Message> {
        let screen = match self.state.current_screen {
            Screen::Welcome => screens::welcome::view(),
            Screen::SetupWizard => screens::setup::view(&self.state),
            Screen::Dashboard => screens::dashboard::view(&self.state),
            Screen::Settings => screens::settings::view(&self.state),
            Screen::About => screens::about::view(),
            Screen::Developer => screens::developer::view(&self.state),
            Screen::Scenery => screens::scenery::view(&self.state),
        };

        use iced::Length;
        use iced::widget::{button, column, container, row, rule, text};

        // --- Tab bar ---
        let tabs: &[(Screen, &str, &str)] = &[
            (Screen::Dashboard, helpers::ICON_HOME, "Dashboard"),
            (Screen::Scenery, helpers::ICON_DOWNLOAD, "Scenery"),
            (Screen::Settings, helpers::ICON_SETTINGS, "Settings"),
            (Screen::Developer, helpers::ICON_DEV, "Developer"),
            (Screen::About, helpers::ICON_INFO, "About"),
        ];

        let current = self.state.current_screen;
        let tab_bar = container(
            row(tabs
                .iter()
                .map(|(scr, icon, label)| {
                    let btn = button(text(format!("{} {}", icon, label)).size(13)).padding([8, 16]);

                    if *scr == current {
                        btn.style(button::primary).into()
                    } else {
                        btn.style(button::secondary)
                            .on_press(Message::GoToScreen(*scr))
                            .into()
                    }
                })
                .collect::<Vec<Element<'_, Message>>>())
            .spacing(2)
            .align_y(iced::Alignment::Center),
        )
        .padding([4, 8])
        .width(Length::Fill);

        // --- Status bar ---
        let web_color = if self.state.web_server.is_running() {
            iced::Color::from_rgb(0.3, 0.7, 0.3)
        } else {
            iced::Color::from_rgb(0.5, 0.5, 0.5)
        };
        let xp_color = if self.state.xplane_tracker.is_running() {
            iced::Color::from_rgb(0.3, 0.7, 0.3)
        } else {
            iced::Color::from_rgb(0.5, 0.5, 0.5)
        };

        let downloads_active = self.state.downloading_regions.len();

        let mut status_items = vec![
            text(format!("Web: {}", self.state.web_server.label()))
                .size(11)
                .color(web_color)
                .into(),
            text("  ·  ").size(11).into(),
            text(format!("X-Plane: {}", self.state.xplane_tracker.label()))
                .size(11)
                .color(xp_color)
                .into(),
            text("  ·  ").size(11).into(),
            text(format!("Provider: {}", self.state.config.tile_provider))
                .size(11)
                .into(),
        ];

        if downloads_active > 0 {
            status_items.push(text("  ·  ").size(11).into());
            status_items.push(
                text(format!("{} download(s) active", downloads_active))
                    .size(11)
                    .color(iced::Color::from_rgb(0.3, 0.6, 0.9))
                    .into(),
            );
        }

        if let Some(ref url) = self.state.web_server_url {
            status_items.push(text("  ·  ").size(11).into());
            status_items.push(text(url.clone()).size(11).into());
        }

        let status_bar = container(row(status_items).align_y(iced::Alignment::Center))
            .padding([6, 16])
            .width(Length::Fill);

        // --- Layout: tab bar + screen + status bar ---
        column![
            tab_bar,
            rule::horizontal(1),
            container(screen).height(Length::Fill),
            rule::horizontal(1),
            status_bar,
        ]
        .into()
    }
}

/// Start all backend services: web server + X-Plane dataref tracker.
///
/// The web server and tracker share the same StatsStore and DatarefTracker
/// so the web UI shows live data from X-Plane.
async fn start_all_services(
    web_port: u16,
    xplane_host: &str,
    xplane_port: u16,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<String, String> {
    use crate::stats::StatsStore;
    use crate::xplane::dataref::{self, DatarefTracker};

    // Shared state between web server and tracker
    let stats = Arc::new(StatsStore::new());
    let tracker = Arc::new(DatarefTracker::new());

    // Start web server
    let addr = crate::webui::start_server(web_port, stats.clone(), tracker.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Start X-Plane dataref tracker (runs in background, retries on timeout)
    let xplane_addr: SocketAddr = format!("{}:{}", xplane_host, xplane_port)
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;

    tokio::spawn(dataref::run_tracker(tracker, xplane_addr, shutdown_rx));

    Ok(format!("http://{}", addr))
}

/// Fetch a test tile, save DDS, return status message + RGBA image data for preview.
async fn fetch_test_tile(
    lat: f64,
    lon: f64,
    zoom: u32,
    provider_name: &str,
) -> Result<(String, Option<(u32, u32, Vec<u8>)>), String> {
    use crate::pipeline::dds::DdsFormat;
    use crate::pipeline::decode::ImageBuffer;
    use crate::pipeline::image::Image;
    use crate::tiles::assembler::{AssemblyConfig, assemble_tile};
    use crate::tiles::fetcher::TileFetcher;
    use crate::tiles::provider::ProviderFactory;

    // Validate zoom against provider limits
    if let Some(info) = crate::tiles::provider::provider_info(provider_name)
        && (zoom < info.min_zoom || zoom > info.max_zoom)
    {
        return Err(format!(
            "Zoom {} is outside {}'s range ({}-{})",
            zoom, info.display_name, info.min_zoom, info.max_zoom
        ));
    }

    let provider = ProviderFactory::create(provider_name)
        .ok_or_else(|| format!("Unknown provider: {}", provider_name))?;

    let fetcher = Arc::new(TileFetcher::new(provider));

    // Convert lat/lon to fractional tile coordinates, then center the 2x2 grid
    let n = 2_f64.powi(zoom as i32);
    let tile_x = (lon + 180.0) / 360.0 * n;
    let lat_rad = lat.to_radians();
    let tile_y =
        (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n;

    // Offset by -1 so the target point is at the center of the 2x2 grid
    let base_col = (tile_x - 1.0).floor().max(0.0) as u32;
    let base_row = (tile_y - 1.0).floor().max(0.0) as u32;

    let chunks_per_side = 2u32;
    let total = (chunks_per_side * chunks_per_side) as usize;

    // Fetch chunks centered on the target
    let mut jpeg_chunks: Vec<Option<Vec<u8>>> = Vec::with_capacity(total);
    let mut fetched = 0u32;

    for dy in 0..chunks_per_side {
        for dx in 0..chunks_per_side {
            let c = base_col + dx;
            let r = base_row + dy;
            match fetcher.get_chunk_data(r, c, provider_name, zoom).await {
                Ok(Some(data)) => {
                    fetched += 1;
                    jpeg_chunks.push(Some(data));
                }
                _ => jpeg_chunks.push(None),
            }
        }
    }

    // If no chunks were fetched at all, return an error
    if fetched == 0 {
        return Err(format!(
            "All {} chunks failed to download. Provider '{}' may require authentication or be unavailable. Try ARC or BI instead.",
            total, provider_name
        ));
    }

    // Compose preview image, tracking decode successes
    let tile_px = chunks_per_side * 256;
    let mut tile_image = Image::new(tile_px, tile_px);
    let fallback = Image::new_filled(256, 256, [66, 77, 55, 255]);
    let mut decoded = 0u32;

    for (i, chunk) in jpeg_chunks.iter().enumerate() {
        let cx = (i as u32 % chunks_per_side) * 256;
        let cy = (i as u32 / chunks_per_side) * 256;
        match chunk {
            Some(data) => {
                if let Ok(buf) = ImageBuffer::from_jpeg(data)
                    && let Ok(img) = Image::from_raw(buf.width, buf.height, buf.data)
                {
                    tile_image.paste(cx, cy, &img).ok();
                    decoded += 1;
                    continue;
                }
                tile_image.paste(cx, cy, &fallback).ok();
            }
            None => {
                tile_image.paste(cx, cy, &fallback).ok();
            }
        }
    }

    // If data was received but couldn't be decoded as images, it's likely
    // an error page (HTML) or wrong format
    if decoded == 0 && fetched > 0 {
        return Err(format!(
            "Downloaded {} chunks but none were valid images. Provider '{}' likely returned error pages. Try ARC or BI instead.",
            fetched, provider_name
        ));
    }

    let preview_data = (tile_image.width, tile_image.height, tile_image.data.clone());

    // Assemble DDS
    let config = AssemblyConfig {
        chunks_per_side,
        chunk_size: 256,
        format: DdsFormat::BC3,
        missing_color: [66, 77, 55],
    };

    let start = std::time::Instant::now();
    let result = assemble_tile(&jpeg_chunks, &config).map_err(|e| e.to_string())?;
    let elapsed = start.elapsed();

    std::fs::write("test_output.dds", &result.dds_data).map_err(|e| e.to_string())?;

    let msg = format!(
        "Fetched {}/{} chunks, DDS {:.1} MB, {:.0}ms",
        fetched,
        total,
        result.dds_data.len() as f64 / 1_048_576.0,
        elapsed.as_millis(),
    );

    Ok((msg, Some(preview_data)))
}

/// Open a native folder picker dialog.
fn browse_folder(field_name: &str, current_path: &str) -> Task<Message> {
    let field = field_name.to_string();
    let start_dir = current_path.to_string();

    Task::perform(
        async move {
            let mut dialog = rfd::AsyncFileDialog::new().set_title("Choose folder");

            // Start in the current directory if it exists
            let start = std::path::Path::new(&start_dir);
            if start.exists() {
                dialog = dialog.set_directory(start);
            } else if let Some(parent) = start.parent()
                && parent.exists()
            {
                dialog = dialog.set_directory(parent);
            }

            dialog.pick_folder().await
        },
        move |result| match result {
            Some(handle) => {
                Message::FolderPicked(field.clone(), handle.path().to_string_lossy().into_owned())
            }
            None => Message::Tick,
        },
    )
}

/// Fetch available regions from GitHub and list installed packs.
async fn fetch_regions_and_installed(
    install_dir: &str,
    download_dir: &str,
) -> Result<(Vec<state::SceneryRegionInfo>, Vec<state::InstalledPackInfo>), String> {
    let regions = crate::scenery::discovery::discover_regions()
        .await
        .map_err(|e| e.to_string())?;

    let dl_path = std::path::Path::new(download_dir);
    let ui_regions: Vec<state::SceneryRegionInfo> = regions
        .iter()
        .map(|r| state::SceneryRegionInfo {
            id: r.id.clone(),
            name: r.name.clone(),
            version: r.version.clone(),
            package_count: r.packages.len(),
            total_size_bytes: r.packages.iter().map(|p| p.size_bytes).sum(),
            has_partial_download: crate::scenery::installer::has_partial_downloads(dl_path, &r.id),
        })
        .collect();

    let packs = crate::scenery::installer::list_installed_packs(std::path::Path::new(install_dir));
    let ui_packs: Vec<state::InstalledPackInfo> = packs
        .into_iter()
        .map(|p| state::InstalledPackInfo {
            id: p.id,
            name: p.name,
            version: p.ver,
        })
        .collect();

    Ok((ui_regions, ui_packs))
}

/// Download and install a scenery region.
async fn download_and_install_region(
    region_id: &str,
    download_dir: &str,
    install_dir: &str,
    cancel: &tokio_util::sync::CancellationToken,
    progress_bytes: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    progress_file: &std::sync::Arc<std::sync::Mutex<String>>,
    progress_files_done: &std::sync::Arc<std::sync::atomic::AtomicU32>,
) -> Result<String, String> {
    use crate::scenery::discovery;
    use crate::scenery::installer;

    // Discover to get download URLs
    let regions = discovery::discover_regions()
        .await
        .map_err(|e| e.to_string())?;

    let region = regions
        .iter()
        .find(|r| r.id == region_id)
        .ok_or_else(|| format!("Region '{}' not found", region_id))?;

    let download_path = std::path::Path::new(download_dir);
    let install_path = std::path::Path::new(install_dir);
    let mut downloaded_files = Vec::new();

    // Download all packages in the region
    let mut verified = 0u32;
    let mut unverified = 0u32;

    for package in &region.packages {
        *progress_file.lock().expect("progress mutex") = package.filename.clone();

        let path = installer::download_package(package, download_path, cancel, progress_bytes)
            .await
            .map_err(|e| format!("{}", e))?;

        // Download and verify SHA256 hash if available
        let has_hash = installer::download_hash_file(package, download_path)
            .await
            .unwrap_or(false);

        if has_hash {
            match installer::verify_file_hash(&path) {
                Ok(true) => verified += 1,
                Ok(false) => unverified += 1,
                Err(e) => {
                    return Err(format!(
                        "Verification failed for {}: {}",
                        package.filename, e
                    ));
                }
            }
        } else {
            unverified += 1;
        }

        downloaded_files.push(path);
        progress_files_done.fetch_add(1, Ordering::Relaxed);
    }

    // Extract ZIP files
    for path in &downloaded_files {
        if path.to_string_lossy().ends_with(".zip") {
            let target = install_path
                .join("z_autoortho")
                .join("scenery")
                .join(format!("z_ao_{}", region_id));
            installer::extract_zip(path, &target).map_err(|e| format!("Extract failed: {}", e))?;
        }
    }

    // Save metadata
    let info = installer::PackInfo {
        id: region_id.to_string(),
        name: region.name.clone(),
        ver: region.version.clone(),
        ortho_prefix: format!("z_{}_", region_id),
        overlay_prefix: format!("y_{}_overlays", region_id),
        ortho_dirs: vec![],
        info_ver: "v1".to_string(),
    };
    installer::save_pack_info(&info, install_path)
        .map_err(|e| format!("Failed to save metadata: {}", e))?;

    let verify_msg = if verified > 0 && unverified == 0 {
        format!(", {} verified", verified)
    } else if verified > 0 {
        format!(", {} verified, {} unverified", verified, unverified)
    } else {
        String::new()
    };

    Ok(format!(
        "Installed {} v{} ({} packages{})",
        region.name,
        region.version,
        region.packages.len(),
        verify_msg
    ))
}

/// Run the desktop application
pub fn run() -> iced::Result {
    // Load saved window geometry from config
    let config = crate::config::AutoOrthoConfig::load();

    let _size = iced::Size::new(
        config.window_width.unwrap_or(900.0),
        config.window_height.unwrap_or(900.0),
    );

    let has_saved = config.window_x.is_some() && config.window_y.is_some();
    log::debug!(
        "Window config: x={:?} y={:?} w={:?} h={:?} has_saved={}",
        config.window_x,
        config.window_y,
        config.window_width,
        config.window_height,
        has_saved
    );

    // Store geometry for the Opened event handler to apply
    SAVED_WINDOW_GEOM.lock().expect("lock").replace((
        config.window_x,
        config.window_y,
        config.window_width,
        config.window_height,
    ));
    HAS_SAVED_GEOM.store(has_saved, std::sync::atomic::Ordering::Relaxed);

    // Always start centered at default size — the Opened handler will move/resize
    iced::application(boot, AutoOrthoApp::update, AutoOrthoApp::view)
        .title(AutoOrthoApp::title)
        .subscription(AutoOrthoApp::subscription)
        .scale_factor(AutoOrthoApp::scale_factor)
        .default_font(NERD_FONT)
        .exit_on_close_request(false)
        .window(iced::window::Settings {
            size: iced::Size::new(900.0, 900.0),
            min_size: Some(iced::Size::new(700.0, 500.0)),
            ..Default::default()
        })
        .centered()
        .run()
}

/// Boot function — creates the app and returns initial tasks.
fn boot() -> (AutoOrthoApp, Task<Message>) {
    let mut app = AutoOrthoApp::new();

    // Auto-refresh scenery list on startup
    app.state.scenery_refreshing = true;
    app.state.scenery_status = Some("Loading available scenery packs...".to_string());

    let install_dir = app.state.scenery_install_dir.clone();
    let download_dir = app.state.scenery_download_dir.clone();
    let (tx, rx) = oneshot::channel();
    let rt = app.runtime.clone();

    rt.spawn(async move {
        let result = fetch_regions_and_installed(&install_dir, &download_dir).await;
        let _ = tx.send(result);
    });

    let scenery_task = Task::perform(
        async { rx.await.unwrap_or(Err("Channel closed".into())) },
        |result| match result {
            Ok((regions, _installed)) => Message::RegionsLoaded(regions),
            Err(e) => Message::RegionsLoadFailed(e),
        },
    );

    // Load the bundled Nerd Font
    let font_task = iced::font::load(FIRA_CODE_NERD).map(|_| Message::Tick);

    (app, Task::batch([font_task, scenery_task]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = AutoOrthoApp::new();
        assert_eq!(app.title(), "AutoOrtho - Satellite Imagery for X-Plane");
    }

    #[test]
    fn test_initial_screen_depends_on_config() {
        let app = AutoOrthoApp::new();
        assert_eq!(app.state.current_screen, Screen::Dashboard);
    }

    #[test]
    fn test_screen_navigation() {
        let mut app = AutoOrthoApp::new();
        let _ = app.update(Message::GoToScreen(Screen::SetupWizard));
        assert_eq!(app.state.current_screen, Screen::SetupWizard);
    }

    #[test]
    fn test_config_update() {
        let mut app = AutoOrthoApp::new();
        let _ = app.update(Message::SetXPlanePath("/home/user/X-Plane 12".to_string()));
        assert_eq!(app.state.config.xplane_path, "/home/user/X-Plane 12");
    }

    #[test]
    fn test_services_started() {
        let mut app = AutoOrthoApp::new();
        let _ = app.update(Message::ServicesStarted(
            "http://127.0.0.1:5847".to_string(),
        ));
        assert_eq!(app.state.web_server, ServiceStatus::Running);
        assert_eq!(app.state.xplane_tracker, ServiceStatus::Running);
        assert_eq!(
            app.state.web_server_url,
            Some("http://127.0.0.1:5847".to_string())
        );
    }

    #[test]
    fn test_services_failed() {
        let mut app = AutoOrthoApp::new();
        let _ = app.update(Message::ServicesFailed("port in use".to_string()));
        assert_eq!(app.state.web_server, ServiceStatus::Error);
        assert!(app.state.error_message.is_some());
    }

    #[test]
    fn test_stop_services() {
        let mut app = AutoOrthoApp::new();
        let _ = app.update(Message::ServicesStarted(
            "http://127.0.0.1:5847".to_string(),
        ));
        assert!(app.state.any_service_running());

        let _ = app.update(Message::StopServices);
        assert!(!app.state.any_service_running());
        assert_eq!(app.state.web_server_url, None);
    }

    #[test]
    fn test_title_changes_when_running() {
        let mut app = AutoOrthoApp::new();
        assert!(!app.title().contains("[Running]"));

        let _ = app.update(Message::ServicesStarted(
            "http://127.0.0.1:5847".to_string(),
        ));
        assert!(app.title().contains("[Running]"));
    }

    #[test]
    fn test_stop_sends_shutdown() {
        let mut app = AutoOrthoApp::new();
        let (tx, rx) = watch::channel(false);
        app.shutdown_tx = Some(tx);

        let _ = app.update(Message::StopServices);
        // Shutdown signal should have been sent
        assert!(*rx.borrow());
    }
}
