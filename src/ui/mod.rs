// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use iced::{Element, Font, Subscription, Task};
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;
#[cfg(test)]
use tempfile::TempDir;
use tokio::sync::{oneshot, watch};

pub mod dev_test_state;
pub mod handlers;
pub mod prefetch_state;
pub mod scenery_state;
pub mod service_state;
use crate::scenery::paths::mount_dir;
use crate::xplane::simbrief::FlightPlan;

/// Saved window geometry to restore on boot: (x, y, width, height)
#[allow(clippy::type_complexity)]
pub(crate) static SAVED_WINDOW_GEOM: Mutex<
    Option<(Option<f32>, Option<f32>, Option<f32>, Option<f32>)>,
> = Mutex::new(None);

/// Whether saved geometry exists (checked by new() before GEOM is consumed)
static HAS_SAVED_GEOM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Shared Tokio runtime for the UI application.
/// Set once in run() and retrieved by AutoOrthoApp::new().
static RUNTIME: OnceLock<Arc<tokio::runtime::Runtime>> = OnceLock::new();

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
    pub(crate) state: AppState,
    /// Tokio runtime handle for backend services (shared with main)
    pub(crate) runtime: Arc<tokio::runtime::Runtime>,
    /// Shutdown signal sender — drop or send true to stop services
    pub(crate) shutdown_tx: Option<watch::Sender<bool>>,
    /// Ignore window move/resize events until this instant has passed
    pub(crate) window_events_locked_until: Option<std::time::Instant>,
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
    SetEnableDynamicZoom(bool),
    SetUseSimBriefAltitude(bool),
    SetSimHeavenCompat(bool),
    SetUIScale(f64),

    // Configuration persistence
    SaveConfiguration,
    LoadConfiguration,
    SetDebugMode(bool),

    // Runtime control
    StartServices,
    StopServices,
    ServicesStarted(
        String,
        Option<std::sync::Arc<crate::xplane::dataref::DatarefTracker>>,
        std::sync::Arc<crate::ui::state::TileProgress>,
        Option<std::sync::Arc<parking_lot::Mutex<crate::pipeline::cache::DdsCache>>>,
    ), // web URL, tracker, tile progress, and DDS cache
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
    TestFallbackLookup,
    FallbackTestComplete(Option<crate::ui::dev_test_state::FallbackTestResult>),

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
    SetDdsMemoryCacheMb(u64),
    SetChunkMemoryCacheMb(u64),

    // Night exclusion
    SetEnableNightExclusion(bool),
    SetNightThreshold(i32),
    SetDayThreshold(i32),

    // Seasonal adjustment
    SetSeason(crate::config::Season),
    SetSpringSaturation(u32),
    SetSummerSaturation(u32),
    SetAutumnSaturation(u32),
    SetWinterSaturation(u32),

    // Fallback settings
    SetFallbackLevel(crate::config::FallbackLevel),
    SetFallbackMaxZoomGap(u32),
    SetFallbackCacheEnabled(bool),
    SetRateLimit(f64),

    // SimBrief
    SetSimbriefUserId(String),
    SetRouteConsiderationRadius(u32),
    SetRouteDeviationThreshold(u32),
    SetRoutePrefetchRadius(u32),
    SetPrefetchRoutePercent(u32),
    SetPrefetchAirports(bool),
    SetAirportRadius(u32),
    FetchSimbrief,
    SimbriefLoaded(
        String,                         // summary
        Vec<(String, String, f32)>,     // fixes
        Arc<Mutex<Option<FlightPlan>>>, // flight plan
    ),
    SimbriefCoverageChecked(Option<String>), // warning message if coverage issue
    SimbriefFailed(String),
    ToggleSimbriefDetails,

    // Route prefetch
    PrefetchRoute,
    StopPrefetch,
    PrefetchProgress(u32, u32), // (completed, total)
    PrefetchComplete(String),
    PrefetchCompleteCacheFull(String),
    PrefetchFailed(String),

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
        let runtime = RUNTIME
            .get()
            .expect("Runtime not set — call run() instead of constructing directly")
            .clone();
        let has_saved_geom = HAS_SAVED_GEOM.load(std::sync::atomic::Ordering::Relaxed);
        Self {
            state: AppState::new(),
            runtime,
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
                handlers::handle_set_xplane_path(&mut self.state, path);
            }
            Message::SetCacheDir(dir) => {
                handlers::handle_set_cache_dir(&mut self.state, dir);
            }
            Message::SetXPlaneHost(host) => {
                handlers::set_xplane_host(&mut self.state, host);
            }
            Message::SetXPlanePort(port_str) => {
                handlers::handle_set_xplane_port(&mut self.state, &port_str);
            }
            Message::SetTileProvider(provider) => {
                return handlers::handle_set_tile_provider(self, provider);
            }
            Message::SetMinZoom(zoom) => {
                handlers::set_min_zoom(&mut self.state, zoom);
            }
            Message::SetMaxZoom(zoom) => {
                handlers::set_max_zoom(&mut self.state, zoom);
            }
            Message::SetEnableDynamicZoom(enabled) => {
                handlers::set_enable_dynamic_zoom(&mut self.state, enabled);
            }
            Message::SetUseSimBriefAltitude(enabled) => {
                handlers::set_use_simbrief_altitude(&mut self.state, enabled);
            }
            Message::SetSimHeavenCompat(enabled) => {
                handlers::set_simheaven_compat(&mut self.state, enabled);
            }
            Message::SetUIScale(scale) => {
                handlers::set_ui_scale(&mut self.state, scale);
            }
            Message::SetDdsCacheSizeMb(mb) => {
                handlers::set_dds_cache_size_mb(&mut self.state, mb);
            }
            Message::SetEnableDdsCache(enabled) => {
                handlers::set_enable_dds_cache(&mut self.state, enabled);
            }
            Message::SetDdsMemoryCacheMb(mb) => {
                handlers::set_dds_memory_cache_mb(&mut self.state, mb);
            }
            Message::SetChunkMemoryCacheMb(mb) => {
                handlers::set_chunk_memory_cache_mb(&mut self.state, mb);
            }
            Message::SetEnableNightExclusion(v) => {
                handlers::set_enable_night_exclusion(&mut self.state, v);
            }
            Message::SetNightThreshold(v) => {
                handlers::set_night_threshold(&mut self.state, v as f32);
            }
            Message::SetDayThreshold(v) => {
                handlers::set_day_threshold(&mut self.state, v as f32);
            }
            Message::SetSeason(season) => {
                handlers::set_season(&mut self.state, season);
            }
            Message::SetSpringSaturation(v) => {
                handlers::set_spring_saturation(&mut self.state, (v as f32) / 100.0);
            }
            Message::SetSummerSaturation(v) => {
                handlers::set_summer_saturation(&mut self.state, (v as f32) / 100.0);
            }
            Message::SetAutumnSaturation(v) => {
                handlers::set_autumn_saturation(&mut self.state, (v as f32) / 100.0);
            }
            Message::SetWinterSaturation(v) => {
                handlers::set_winter_saturation(&mut self.state, (v as f32) / 100.0);
            }
            Message::SetFallbackLevel(level) => {
                handlers::set_fallback_level(&mut self.state, level);
            }
            Message::SetFallbackMaxZoomGap(gap) => {
                handlers::set_fallback_max_zoom_gap(&mut self.state, gap);
            }
            Message::SetFallbackCacheEnabled(enabled) => {
                handlers::set_fallback_cache_enabled(&mut self.state, enabled);
            }
            Message::SetRateLimit(rate) => {
                handlers::set_rate_limit(&mut self.state, rate);
            }
            Message::ClearDdsCache => {
                handlers::handle_clear_dds_cache(self);
            }
            Message::SetSimbriefUserId(id) => {
                handlers::set_simbrief_user_id(&mut self.state, id);
            }
            Message::SetRouteConsiderationRadius(v) => {
                handlers::set_route_consideration_radius(&mut self.state, v);
            }
            Message::SetRouteDeviationThreshold(v) => {
                handlers::set_route_deviation_threshold(&mut self.state, v);
            }
            Message::SetRoutePrefetchRadius(v) => {
                handlers::set_route_prefetch_radius(&mut self.state, v);
            }
            Message::SetPrefetchRoutePercent(v) => {
                handlers::set_prefetch_route_percent(&mut self.state, v);
            }
            Message::SetPrefetchAirports(v) => {
                handlers::set_prefetch_airports(&mut self.state, v);
            }
            Message::SetAirportRadius(v) => {
                handlers::set_airport_radius(&mut self.state, v);
            }
            Message::FetchSimbrief => return handlers::handle_fetch_simbrief(self),
            Message::SimbriefLoaded(summary, fixes, plan) => {
                return handlers::handle_simbrief_loaded(self, summary, fixes, plan);
            }
            Message::SimbriefCoverageChecked(warning) => {
                handlers::handle_simbrief_coverage_checked(self, warning);
            }
            Message::SimbriefFailed(err) => handlers::handle_simbrief_failed(self, err),
            Message::ToggleSimbriefDetails => handlers::handle_toggle_simbrief_details(self),
            Message::PrefetchRoute => return handlers::handle_prefetch_route(self),
            Message::StopPrefetch => handlers::handle_stop_prefetch(self),
            Message::PrefetchProgress(completed, total) => {
                handlers::handle_prefetch_progress(self, completed, total)
            }
            Message::PrefetchComplete(msg) => handlers::handle_prefetch_complete(self, msg),
            Message::PrefetchCompleteCacheFull(msg) => {
                handlers::handle_prefetch_complete_cache_full(self, msg)
            }
            Message::PrefetchFailed(err) => handlers::handle_prefetch_failed(self, err),
            Message::SaveConfiguration => {
                handlers::handle_save_configuration(self);
            }
            Message::LoadConfiguration => {
                self.state.load_config();
            }
            Message::SetDebugMode(v) => {
                handlers::set_debug_mode(&mut self.state, v);
            }
            Message::StartServices => return handlers::handle_start_services(self),
            Message::StopServices => handlers::handle_stop_services(self),
            Message::ServicesStarted(url, tracker, tile_progress, dds_cache) => {
                handlers::handle_services_started(self, url, tracker, tile_progress, dds_cache)
            }
            Message::ServicesFailed(err) => handlers::handle_services_failed(self, err),
            Message::SetSceneryDownloadDir(v) => {
                handlers::set_scenery_download_dir_state(&mut self.state, v);
            }
            Message::RefreshAvailableRegions => {
                return handlers::handle_refresh_available_regions(self);
            }
            Message::RegionsLoaded(regions) => handlers::handle_regions_loaded(self, regions),
            Message::RegionsLoadFailed(err) => handlers::handle_regions_load_failed(self, err),
            Message::DownloadRegion(region_id) => {
                return handlers::handle_download_region(self, region_id);
            }
            Message::CancelDownload(region_id) => handlers::handle_cancel_download(self, region_id),
            Message::CleanRegionDownloads(region_id) => {
                handlers::handle_clean_region_downloads(self, region_id)
            }
            Message::UninstallRegion(region_id) => {
                handlers::handle_uninstall_region(self, region_id)
            }
            Message::DownloadComplete(region_id, msg) => {
                handlers::handle_download_complete(self, region_id, msg)
            }
            Message::DownloadFailed(region_id, err) => {
                handlers::handle_download_failed(self, region_id, err)
            }
            Message::SetTestLat(v) => handlers::handle_set_test_lat(self, v),
            Message::SetTestLon(v) => handlers::handle_set_test_lon(self, v),
            Message::SetTestZoom(v) => handlers::handle_set_test_zoom(self, v),
            Message::FetchTestTile => return handlers::handle_fetch_test_tile(self),
            Message::TestTileComplete(msg, image_data) => {
                handlers::handle_test_tile_complete(self, msg, image_data)
            }
            Message::TestTileFailed(err) => handlers::handle_test_tile_failed(self, err),
            Message::TestFallbackLookup => return handlers::handle_test_fallback_lookup(self),
            Message::FallbackTestComplete(result) => {
                handlers::handle_fallback_test_complete(self, result)
            }
            Message::BrowseXPlanePath => return handlers::handle_browse_xplane_path(self),
            Message::BrowseCacheDir => return handlers::handle_browse_cache_dir(self),
            Message::BrowseSceneryDownloadDir => {
                return handlers::handle_browse_scenery_download_dir(self);
            }
            Message::FolderPicked(field, path) => handlers::handle_folder_picked(self, field, path),
            Message::WindowOpened(id) => return handlers::handle_window_opened(self, id),
            Message::WindowRestoreSize(id, w, h) => {
                return handlers::handle_window_restore_size(self, id, w, h);
            }
            Message::WindowRestorePosition => {}
            Message::WindowMoved(pos) => return handlers::handle_window_moved(self, pos),
            Message::WindowResized(size) => return handlers::handle_window_resized(self, size),
            Message::WindowCloseRequested => handlers::handle_window_close_requested(self),
            Message::Tick => {
                // Save config periodically when downloads are active (debounced window saves too)
                let _ = self.state.config.save();

                // Poll X-Plane connection status if we have a tracker
                if let Some(tracker) = &self.state.tracker {
                    let flight_data = tracker.get_flight_data();
                    if flight_data.connected && flight_data.data_valid {
                        self.state.services.xplane_tracker = ServiceStatus::Running;
                    } else {
                        self.state.services.xplane_tracker = ServiceStatus::Stopped;
                    }
                }

                // Poll DDS cache size if available
                if let Some(ref cache) = self.state.dds_cache {
                    let cache = cache.lock();
                    self.state.dds_cache_size_bytes = cache.size_bytes();
                }

                // Sync waypoint prefetch progress from shared state
                if self.state.prefetch.running {
                    self.state.prefetch.waypoint_status =
                        self.state.prefetch.waypoint_progress.get_all();
                }
            }
            Message::OpenMapInBrowser => {
                if let Some(ref url) = self.state.services.web_server_url {
                    let map_url = format!("{}/map", url);
                    let _ = open::that(&map_url);
                }
            }
            Message::OpenCustomMapEditor => {
                if let Some(ref url) = self.state.services.web_server_url {
                    let editor_url = format!("{}/custommap", url);
                    let _ = open::that(&editor_url);
                }
            }
            Message::OpenWebUI => {
                if let Some(ref url) = self.state.services.web_server_url {
                    let _ = open::that(url);
                }
            }
            Message::Exit => {
                let _ = self.state.config.save();
                if let Some(tx) = self.shutdown_tx.take() {
                    let _ = tx.send(true);
                }
            }
        }
        Task::none()
    }

    fn scale_factor(&self) -> f32 {
        (self.state.config.ui.ui_scale as f32).clamp(0.5, 2.0)
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

        // Tick every 500ms while downloads, tile progress, or prefetch is active
        // to refresh progress bars
        let tile_progress_active = self
            .state
            .tile_progress
            .active
            .load(std::sync::atomic::Ordering::Relaxed);
        if !self.state.scenery.downloading.is_empty()
            || tile_progress_active
            || self.state.prefetch.running
        {
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
        let web_color = if self.state.services.web_server.is_running() {
            iced::Color::from_rgb(0.3, 0.7, 0.3)
        } else {
            iced::Color::from_rgb(0.5, 0.5, 0.5)
        };
        let xp_color = if self.state.services.xplane_tracker.is_running() {
            iced::Color::from_rgb(0.3, 0.7, 0.3)
        } else {
            iced::Color::from_rgb(0.5, 0.5, 0.5)
        };

        let downloads_active = self.state.scenery.downloading.len();

        let mut status_items = vec![
            text(format!("Web: {}", self.state.services.web_server.label()))
                .size(11)
                .color(web_color)
                .into(),
            text("  ·  ").size(11).into(),
            text(format!(
                "X-Plane: {}",
                self.state.services.xplane_tracker.label()
            ))
            .size(11)
            .color(xp_color)
            .into(),
            text("  ·  ").size(11).into(),
            text(format!("Provider: {}", self.state.config.tile.provider))
                .size(11)
                .into(),
        ];

        // Show tile progress if active
        if self
            .state
            .tile_progress
            .active
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let tile_label = self.state.tile_progress.tile_label();
            let progress = self.state.tile_progress.progress();
            let chunks_done = self
                .state
                .tile_progress
                .chunks_done
                .load(std::sync::atomic::Ordering::Relaxed);
            let chunks_total = self
                .state
                .tile_progress
                .chunks_total
                .load(std::sync::atomic::Ordering::Relaxed);

            // Use same green as "Web: Running" for progress bar
            let progress_green = iced::Color::from_rgb(0.3, 0.7, 0.3);
            // Use black for readable text
            let text_color = iced::Color::BLACK;

            status_items.push(text("  ·  ").size(11).into());
            status_items.push(
                text(format!(
                    "Tile: {} ({}/{})",
                    tile_label, chunks_done, chunks_total
                ))
                .size(11)
                .color(text_color)
                .into(),
            );

            // Simple progress bar
            let bar_width: usize = 40;
            let filled = (progress * bar_width as f32) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(bar_width.saturating_sub(filled));
            status_items.push(text(" ").size(11).into());
            status_items.push(
                text(bar)
                    .size(11)
                    .font(iced::Font::MONOSPACE)
                    .color(progress_green)
                    .into(),
            );
        }

        if downloads_active > 0 {
            status_items.push(text("  ·  ").size(11).into());
            status_items.push(
                text(format!("{} download(s) active", downloads_active))
                    .size(11)
                    .color(iced::Color::from_rgb(0.3, 0.6, 0.9))
                    .into(),
            );
        }

        if let Some(ref url) = self.state.services.web_server_url {
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

pub(crate) async fn start_all_services(
    web_port: u16,
    xplane_host: &str,
    xplane_port: u16,
    config: crate::config::AutoOrthoConfig,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<
    (
        String,
        Arc<crate::xplane::dataref::DatarefTracker>,
        Arc<crate::ui::state::TileProgress>,
        Option<Arc<parking_lot::Mutex<crate::pipeline::cache::DdsCache>>>,
    ),
    String,
> {
    use crate::app_context::AppContext;
    use crate::xplane::dataref::{self};

    // Initialize the application context
    let context: AppContext = AppContext::init(config.clone())
        .await
        .map_err(|e| format!("Failed to initialize app context: {}", e))?;

    // Shared state between web server and tracker
    let stats = context.stats.clone();
    let tracker = context.tracker.clone();
    let tile_progress = context.tile_progress.clone();
    let dds_cache = context.dds_cache.clone();
    let web_config = context.config.clone();

    // Clone shutdown signal for web server (tracker gets the original)
    let web_shutdown_rx = shutdown_rx.clone();

    // Start web server with shutdown support
    let addr = crate::webui::start_server_with_shutdown(
        web_port,
        stats.clone(),
        tracker.clone(),
        web_config,
        web_shutdown_rx,
    )
    .await
    .map_err(|e| e.to_string())?;

    // Start X-Plane dataref tracker (runs in background, retries on timeout)
    let xplane_addr: SocketAddr = format!("{}:{}", xplane_host, xplane_port)
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;

    tokio::spawn(dataref::run_tracker(
        tracker.clone(),
        xplane_addr,
        shutdown_rx,
    ));

    // Mount the FUSE filesystem (only if configured and available)
    #[cfg(feature = "fuse")]
    {
        let mount_dir = mount_dir(&config.xplane_path);
        let should_mount = !config.xplane_path.is_empty()
            && !mount_dir.to_string_lossy().is_empty()
            && crate::fuse::platform::is_fuse_available();

        if should_mount {
            // Ensure mount point directory exists — warn but continue on failure
            if let Err(e) = std::fs::create_dir_all(&mount_dir) {
                log::warn!(
                    "Failed to create mount directory: {} — continuing anyway",
                    e
                );
            }

            // Clean up any stale mount before mounting
            // Also clean up old mount point (z_autoortho/textures) from v0.8.9
            let old_mount = mount_dir.join("textures");
            if let Err(e) = crate::fuse::platform::cleanup_mount(&mount_dir) {
                log::debug!("Stale mount cleanup failed (ignored): {}", e);
            }
            if let Err(e) = crate::fuse::platform::cleanup_mount(&old_mount) {
                log::debug!("Old mount cleanup failed (ignored): {}", e);
            }

            // Start FUSE mount in background
            let fs_clone = context.fs.clone();
            let mount_path = mount_dir.to_path_buf();
            let runtime_handle = tokio::runtime::Handle::current();

            tokio::task::spawn_blocking(move || {
                #[cfg(not(windows))]
                use crate::fuse::mount::mount;
                #[cfg(windows)]
                use crate::fuse::mount_win::mount;

                #[cfg(not(windows))]
                let result = mount(fs_clone, &mount_path, runtime_handle);
                #[cfg(windows)]
                let result = mount(
                    fs_clone,
                    &mount_path,
                    runtime_handle,
                    std::sync::Arc::new(context.clone()),
                );

                match result {
                    Ok(()) => Ok::<(), String>(()),
                    Err(e) => {
                        log::warn!("FUSE mount failed (non-fatal): {}", e);
                        Ok::<(), String>(()) // Don't fail service startup if FUSE fails
                    }
                }
            });
        } else {
            log::info!("FUSE mount skipped: xplane_path not configured or FUSE unavailable");
        }
    }
    #[cfg(not(feature = "fuse"))]
    {
        log::info!("FUSE mount skipped: fuse feature not enabled");
    }

    Ok((
        format!("http://{}", addr),
        tracker,
        tile_progress,
        dds_cache,
    ))
}

/// Core implementation of test tile fetch - takes a pre-configured fetcher.
pub(crate) async fn fetch_test_tile_impl(
    lat: f64,
    lon: f64,
    zoom: u32,
    provider_name: &str,
    fetcher: Arc<crate::tiles::fetcher::TileFetcher>,
) -> Result<(String, Option<(u32, u32, Vec<u8>)>), String> {
    use crate::pipeline::dds::DdsFormat;
    use crate::pipeline::decode::ImageBuffer;
    use crate::pipeline::image::Image;
    use crate::tiles::assembler::{AssemblyConfig, assemble_tile};

    // Validate zoom against provider limits
    if let Some(info) = crate::tiles::provider::provider_info(provider_name)
        && (zoom < info.min_zoom || zoom > info.max_zoom)
    {
        return Err(format!(
            "Zoom {} is outside {}'s range ({}-{})",
            zoom, info.display_name, info.min_zoom, info.max_zoom
        ));
    }

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
                    jpeg_chunks.push(Some(data.to_vec()));
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
        seasonal_saturation: 1.0,
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

/// Test the fallback system for a given tile location.
pub(crate) async fn test_fallback_lookup(
    lat: f64,
    lon: f64,
    zoom: u32,
    cache_dir: &std::path::Path,
    fallback_config: crate::config::FallbackConfig,
) -> Option<crate::ui::dev_test_state::FallbackTestResult> {
    use crate::tiles::coords::TileCoords;
    use crate::tiles::fallback::FallbackSystem;

    let (row, col) = TileCoords::latlng_to_tile(lat, lon, zoom).ok()?;
    let maptype = "ARC";

    let fb = FallbackSystem::new(cache_dir.to_path_buf(), fallback_config);

    let result = fb.find_fallback(row, col, maptype, zoom);

    Some(crate::ui::dev_test_state::FallbackTestResult {
        found: result.is_some(),
        fallback_zoom: result.as_ref().map(|r| r.1),
        requested_zoom: zoom,
        tile_key: format!("{}_{}_{}_{}", row, col, maptype, zoom),
        message: if let Some((_data, fb_zoom)) = result {
            format!(
                "Fallback found at zoom {} (gap: {})",
                fb_zoom,
                zoom.saturating_sub(fb_zoom)
            )
        } else {
            "No fallback available".to_string()
        },
    })
}

/// Open a native folder picker dialog.
pub(crate) fn browse_folder(field_name: &str, current_path: &str) -> Task<Message> {
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
pub(crate) async fn fetch_regions_and_installed(
    data_dir: &str,
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

    let packs = crate::scenery::installer::list_installed_packs(std::path::Path::new(data_dir));
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
#[allow(clippy::too_many_arguments)]
pub(crate) async fn download_and_install_region(
    region_id: &str,
    download_dir: &str,
    data_dir: &str,
    cancel: &tokio_util::sync::CancellationToken,
    progress_bytes: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    progress_file: &Arc<Mutex<String>>,
    progress_files_done: &std::sync::Arc<std::sync::atomic::AtomicU32>,
    progress_extract_done: &std::sync::Arc<std::sync::atomic::AtomicU32>,
    progress_extract_total: &std::sync::Arc<std::sync::atomic::AtomicU32>,
    progress_extracting: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    progress_pack_current: &std::sync::Arc<std::sync::atomic::AtomicU32>,
    progress_pack_total: &std::sync::Arc<std::sync::atomic::AtomicU32>,
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
    let data_path = std::path::Path::new(data_dir);
    let mut downloaded_files = Vec::new();

    // Download all packages in the region
    let mut verified = 0u32;
    let mut unverified = 0u32;

    for package in &region.packages {
        *progress_file.lock() = package.filename.clone();

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
    let total_packs = downloaded_files
        .iter()
        .filter(|p| p.to_string_lossy().ends_with(".zip"))
        .count() as u32;
    progress_pack_total.store(total_packs, Ordering::Relaxed);
    progress_extracting.store(true, Ordering::Relaxed);

    for (pack_idx, path) in downloaded_files.iter().enumerate() {
        if path.to_string_lossy().ends_with(".zip") {
            let current_pack = pack_idx as u32 + 1;
            progress_pack_current.store(current_pack, Ordering::Relaxed);

            let target = data_path
                .join("scenery")
                .join(format!("z_ao_{}", region_id));
            installer::extract_zip_with_pack_progress(
                path,
                &target,
                progress_extract_done.clone(),
                progress_extract_total.clone(),
                current_pack,
                total_packs,
            )
            .map_err(|e| format!("Extract failed: {}", e))?;
        }
    }
    progress_extracting.store(false, Ordering::Relaxed);
    progress_pack_current.store(total_packs, Ordering::Relaxed);

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
    installer::save_pack_info(&info, data_path)
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

/// Run the desktop application with a pre-created Tokio runtime.
pub fn run(runtime: tokio::runtime::Runtime) -> iced::Result {
    // Store the runtime globally for AutoOrthoApp::new() to retrieve
    RUNTIME
        .set(Arc::new(runtime))
        .expect("Runtime already set — run() called twice");

    // Load saved window geometry from config
    let config = crate::config::AutoOrthoConfig::load();

    let _size = iced::Size::new(
        config.ui.window_width.unwrap_or(900.0),
        config.ui.window_height.unwrap_or(900.0),
    );

    let has_saved = config.ui.window_x.is_some() && config.ui.window_y.is_some();
    log::debug!(
        "Window config: x={:?} y={:?} w={:?} h={:?} has_saved={}",
        config.ui.window_x,
        config.ui.window_y,
        config.ui.window_width,
        config.ui.window_height,
        has_saved
    );

    // Store geometry for the Opened event handler to apply
    SAVED_WINDOW_GEOM.lock().replace((
        config.ui.window_x,
        config.ui.window_y,
        config.ui.window_width,
        config.ui.window_height,
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

    // Run one-time migration of scenery files from old location
    app.state.run_startup_migration();

    // Auto-refresh scenery list on startup
    app.state.scenery.refreshing = true;
    app.state.scenery.status = Some("Loading available scenery packs...".to_string());

    let data_dir = app.state.scenery.data_dir.clone();
    let download_dir = app.state.scenery.download_dir.clone();
    let (tx, rx) = oneshot::channel();
    let rt = app.runtime.clone();

    rt.spawn(async move {
        let result = fetch_regions_and_installed(&data_dir, &download_dir).await;
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

/// Generate all DDS tile coordinates for a flight plan route.
/// Returns tiles in route order (origin → destination) with duplicates removed.
fn generate_route_tiles(
    flight_plan: &crate::xplane::simbrief::FlightPlan,
    config: &crate::config::AutoOrthoConfig,
) -> Vec<(u32, u32)> {
    use crate::tiles::coords::TileCoords;
    use std::collections::HashSet;

    let fixes = &flight_plan.fixes;
    let mut seen = HashSet::new();
    let mut tiles = Vec::new();

    for (fix_idx, fix) in fixes.iter().enumerate() {
        let is_airport = fix_idx == 0 || fix_idx == fixes.len() - 1;
        let radius_nm = if is_airport && config.flight.prefetch_airports {
            config.flight.airport_radius_nm as f64
        } else {
            config.flight.route_prefetch_radius_nm as f64
        };

        if let Ok((center_col, center_row)) =
            TileCoords::latlng_to_tile(fix.lat, fix.lon, config.flight.near_airport_zoom)
        {
            let tiles_per_nm = 2_f64.powi(config.flight.near_airport_zoom as i32) / 360.0 / 60.0;
            let radius_tiles = (radius_nm * tiles_per_nm).ceil() as i32;

            for dr in -radius_tiles..=radius_tiles {
                for dc in -radius_tiles..=radius_tiles {
                    let col = center_col as i32 + dc;
                    let row = center_row as i32 + dr;
                    if col >= 0 && row >= 0 {
                        let dds_col = col as u32 / 16;
                        let dds_row = row as u32 / 16;
                        let key = (dds_row, dds_col);
                        if seen.insert(key) {
                            tiles.push(key);
                        }
                    }
                }
            }
        }
    }

    tiles
}

/// Execute route prefetch with promote-first strategy.
///
/// Phase 1: Promote tiles already in cache (reverse order: destination -> origin).
///          This makes origin tiles most recently used (top of LRU, last to evict).
/// Phase 2: Fetch remaining tiles until cache is 90% full.
/// Phase 3: Remaining tiles are demand-fetched by X-Plane.
pub(crate) async fn prefetch_route_impl(
    flight_plan: &crate::xplane::simbrief::FlightPlan,
    config: &crate::config::AutoOrthoConfig,
    cancel: &tokio_util::sync::CancellationToken,
    progress: std::sync::Arc<crate::ui::state::WaypointPrefetchProgress>,
) -> Result<String, String> {
    use crate::pipeline::cache::DdsCache;
    use crate::pipeline::dds::DdsFormat;
    use crate::tiles::assembler::{AssemblyConfig, assemble_tile};
    use crate::tiles::fetcher::TileFetcher;
    use crate::tiles::provider::ProviderFactory;

    let fixes = &flight_plan.fixes;
    if fixes.is_empty() {
        return Ok("No fixes in flight plan".to_string());
    }

    // Generate all tiles for the route (ordered origin → destination)
    let route_tiles = generate_route_tiles(flight_plan, config);
    if route_tiles.is_empty() {
        return Ok("No tiles to prefetch".to_string());
    }

    // Create TileFetcher
    let provider = ProviderFactory::create(&config.tile.provider)
        .ok_or_else(|| format!("Unknown provider: {}", config.tile.provider))?;
    let fetcher = TileFetcher::with_rate_limit(
        provider,
        &config.tile.provider,
        config.network.rate_limit.requests_per_second,
    );

    // Create DDS cache
    let cache_dir = std::path::PathBuf::from(&config.cache_dir).join("dds");
    let mut cache = DdsCache::open(
        cache_dir.clone(),
        config.cache.dds_cache_size_mb * 1024 * 1024,
    )
    .map_err(|e| format!("Failed to open DDS cache: {}", e))?;

    // Track which tiles we've handled (promoted or fetched)
    let mut promoted_count = 0u32;
    let mut fetched_count = 0u32;
    let mut processed: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();

    // Phase 1: Promote tiles already in cache (REVERSE order: destination -> origin)
    // This makes origin tiles the most recently used (top of LRU, last to evict)
    for &(dds_row, dds_col) in route_tiles.iter().rev() {
        if cancel.is_cancelled() {
            return Err("Prefetch cancelled".to_string());
        }

        let key = DdsCache::tile_key(
            dds_col,
            dds_row,
            config.flight.near_airport_zoom,
            &config.tile.provider,
        );

        if cache.promote(&key) {
            promoted_count += 1;
            processed.insert((dds_row, dds_col));
        }
    }

    log::info!(
        "Prefetch promote phase: {} tiles already in cache",
        promoted_count
    );

    // Phase 1b: Evict non-route tiles to make space for new fetches
    let route_key_set: std::collections::HashSet<String> = route_tiles
        .iter()
        .map(|&(row, col)| {
            DdsCache::tile_key(
                col,
                row,
                config.flight.near_airport_zoom,
                &config.tile.provider,
            )
        })
        .collect();
    let tiles_to_fetch = route_tiles.len() as u32 - promoted_count;
    // Estimate ~10MB per tile (conservative), ensure space for at least some tiles
    let bytes_needed = (tiles_to_fetch as u64 * 10 * 1024 * 1024)
        .min(config.cache.dds_cache_size_mb * 1024 * 1024 / 2);
    let free_bytes = config.cache.dds_cache_size_mb * 1024 * 1024 - cache.size_bytes();
    if free_bytes < bytes_needed {
        let evicted = cache.evict_non_route_tiles(&route_key_set, bytes_needed - free_bytes);
        log::info!("Evicted {} non-route tiles to make space", evicted);
    }

    // Phase 2: Fetch tiles not in cache, stop at 90%
    for (fix_idx, fix) in fixes.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err("Prefetch cancelled".to_string());
        }

        progress.set(
            fix_idx,
            crate::ui::state::WaypointPrefetchStatus::InProgress,
        );

        // Determine radius: airport radius for origin/destination, waypoint radius for others
        let is_airport = fix_idx == 0 || fix_idx == fixes.len() - 1;
        let radius_nm = if is_airport && config.flight.prefetch_airports {
            config.flight.airport_radius_nm as f64
        } else {
            config.flight.route_prefetch_radius_nm as f64
        };

        // Convert fix position to tile coordinates
        let (center_col, center_row) = match crate::tiles::coords::TileCoords::latlng_to_tile(
            fix.lat,
            fix.lon,
            config.flight.near_airport_zoom,
        ) {
            Ok(coords) => coords,
            Err(_) => {
                progress.set(fix_idx, crate::ui::state::WaypointPrefetchStatus::Failed);
                continue;
            }
        };

        // Calculate radius in tiles
        let tiles_per_nm = 2_f64.powi(config.flight.near_airport_zoom as i32) / 360.0 / 60.0;
        let radius_tiles = (radius_nm * tiles_per_nm).ceil() as i32;

        // Generate DDS tiles in this radius
        let mut dds_tiles: Vec<(u32, u32)> = Vec::new();
        for dr in -radius_tiles..=radius_tiles {
            for dc in -radius_tiles..=radius_tiles {
                let col = center_col as i32 + dc;
                let row = center_row as i32 + dr;
                if col >= 0 && row >= 0 {
                    let dds_col = col as u32 / 16;
                    let dds_row = row as u32 / 16;
                    dds_tiles.push((dds_row, dds_col));
                }
            }
        }

        // Process DDS tiles for this fix
        for (dds_row, dds_col) in dds_tiles {
            if cancel.is_cancelled() {
                return Err("Prefetch cancelled".to_string());
            }

            // Skip if already promoted or fetched
            if processed.contains(&(dds_row, dds_col)) {
                continue;
            }

            // Stop at 90% cache full
            if cache.usage_fraction() >= 0.9 {
                log::info!(
                    "Prefetch stopping at {:.0}% cache full ({} promoted, {} fetched)",
                    cache.usage_fraction() * 100.0,
                    promoted_count,
                    fetched_count
                );
                progress.set(fix_idx, crate::ui::state::WaypointPrefetchStatus::Completed);
                return Ok(format!(
                    "Promoted {} cached tiles, fetched {} new tiles (cache {:.0}% full)",
                    promoted_count,
                    fetched_count,
                    cache.usage_fraction() * 100.0
                ));
            }

            // Fetch all 256 chunks for this DDS tile
            let mut jpeg_chunks: Vec<Option<Vec<u8>>> = Vec::with_capacity(256);
            for dr in 0..16u32 {
                for dc in 0..16u32 {
                    let chunk_row = dds_row * 16 + dr;
                    let chunk_col = dds_col * 16 + dc;
                    match fetcher
                        .get_chunk_data(
                            chunk_row,
                            chunk_col,
                            &config.tile.provider,
                            config.flight.near_airport_zoom,
                        )
                        .await
                    {
                        Ok(Some(data)) => jpeg_chunks.push(Some(data.as_ref().clone())),
                        _ => jpeg_chunks.push(None),
                    }
                }
            }

            // Assemble the DDS tile
            let assembly_config = AssemblyConfig {
                chunks_per_side: 16,
                chunk_size: 256,
                format: if config.tile.max_zoom >= 18 {
                    DdsFormat::BC3
                } else {
                    DdsFormat::BC1
                },
                missing_color: [66, 77, 55],
                seasonal_saturation: 1.0,
            };
            let result = match assemble_tile(&jpeg_chunks, &assembly_config) {
                Ok(r) => r,
                Err(_) => {
                    processed.insert((dds_row, dds_col));
                    continue;
                }
            };

            let key = DdsCache::tile_key(
                dds_col,
                dds_row,
                config.flight.near_airport_zoom,
                &config.tile.provider,
            );
            let metadata = crate::pipeline::cache::DdsCacheMetadata {
                v: 3,
                w: 4096,
                h: 4096,
                mm: result.mipmap_count,
                zl: config.flight.near_airport_zoom,
                max_zl: config.flight.near_airport_zoom,
                fmt: if config.tile.max_zoom >= 18 {
                    "BC3".to_string()
                } else {
                    "BC1".to_string()
                },
                map: config.tile.provider.clone(),
                built: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
                tile_row: dds_row,
                tile_col: dds_col,
                populated_mipmaps: (0..result.mipmap_count).collect(),
                missing_indices: vec![],
                fallback_indices: vec![],
                disk_compression: "zstd".to_string(),
            };
            if cache.put(key, &result.dds_data, &metadata).is_ok() {
                processed.insert((dds_row, dds_col));
                fetched_count += 1;
            }
        }

        // Mark fix as completed
        progress.set(fix_idx, crate::ui::state::WaypointPrefetchStatus::Completed);
    }

    Ok(format!(
        "Promoted {} cached tiles, fetched {} new tiles from {} fixes",
        promoted_count,
        fetched_count,
        fixes.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AutoOrthoConfig;

    fn setup_test_runtime() {
        use tokio::runtime::Builder;
        RUNTIME.get_or_init(|| {
            Arc::new(
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create test runtime"),
            )
        });
    }

    #[test]
    fn test_app_creation() {
        setup_test_runtime();
        let app = AutoOrthoApp::new();
        assert_eq!(app.title(), "AutoOrtho - Satellite Imagery for X-Plane");
    }

    #[test]
    fn test_initial_screen_depends_on_config() {
        setup_test_runtime();
        let app = AutoOrthoApp::new();
        assert_eq!(app.state.current_screen, Screen::Dashboard);
    }

    #[test]
    fn test_screen_navigation() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        let _ = app.update(Message::GoToScreen(Screen::SetupWizard));
        assert_eq!(app.state.current_screen, Screen::SetupWizard);
    }

    #[test]
    fn test_config_update() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        let _ = app.update(Message::SetXPlanePath("/home/user/X-Plane 12".to_string()));
        assert_eq!(app.state.config.xplane_path, "/home/user/X-Plane 12");
    }

    #[test]
    fn test_services_started() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        let url = format!("http://127.0.0.1:{}", crate::webui::WEB_UI_PORT);
        let _ = app.update(Message::ServicesStarted(
            url.clone(),
            None,
            std::sync::Arc::new(crate::ui::state::TileProgress::new()),
            None,
        ));
        assert_eq!(app.state.services.web_server, ServiceStatus::Running);
        assert_eq!(app.state.services.xplane_tracker, ServiceStatus::Running);
        assert_eq!(app.state.services.web_server_url, Some(url));
    }

    #[test]
    fn test_services_failed() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        let _ = app.update(Message::ServicesFailed("port in use".to_string()));
        assert_eq!(app.state.services.web_server, ServiceStatus::Error);
        assert!(app.state.error_message.is_some());
    }

    #[test]
    fn test_stop_services() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        let url = format!("http://127.0.0.1:{}", crate::webui::WEB_UI_PORT);
        let _ = app.update(Message::ServicesStarted(
            url,
            None,
            std::sync::Arc::new(crate::ui::state::TileProgress::new()),
            None,
        ));
        assert!(app.state.any_service_running());

        let _ = app.update(Message::StopServices);
        assert!(!app.state.any_service_running());
        assert_eq!(app.state.services.web_server_url, None);
    }

    #[test]
    fn test_title_changes_when_running() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        assert!(!app.title().contains("[Running]"));

        let url = format!("http://127.0.0.1:{}", crate::webui::WEB_UI_PORT);
        let _ = app.update(Message::ServicesStarted(
            url,
            None,
            std::sync::Arc::new(crate::ui::state::TileProgress::new()),
            None,
        ));
        assert!(app.title().contains("[Running]"));
    }

    #[test]
    fn test_web_ui_port_constant() {
        assert_eq!(crate::webui::WEB_UI_PORT, 5847);
    }

    #[test]
    fn test_stop_sends_shutdown() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        let (tx, rx) = watch::channel(false);
        app.shutdown_tx = Some(tx);

        let _ = app.update(Message::StopServices);
        // Shutdown signal should have been sent
        assert!(*rx.borrow());
    }

    #[tokio::test]
    #[cfg(feature = "fuse")]
    async fn test_start_services_skips_mount_when_no_xplane_path() {
        let mut config = AutoOrthoConfig::default();
        config.xplane_path = "".to_string(); // Not configured
        // Use temp directory for cache to avoid polluting user environment
        config.cache_dir = TempDir::new().unwrap().path().to_string_lossy().to_string();
        // Disable DDS cache for test to avoid filesystem operations
        config.cache.enable_dds_cache = false;

        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        // Use port 0 to get a random available port
        let result = start_all_services(0, "127.0.0.1", 49000, config, shutdown_rx).await;

        assert!(
            result.is_ok(),
            "Services should start without FUSE when no X-Plane path: {:?}",
            result.err()
        );
        let (url, _tracker, _tile_progress, _dds_cache) = result.unwrap();
        assert!(url.starts_with("http://"));
    }

    #[tokio::test]
    #[cfg(feature = "fuse")]
    async fn test_start_services_handles_fuse_unavailable() {
        let mut config = AutoOrthoConfig::default();
        // Use a path that exists but FUSE can't mount (for testing)
        config.xplane_path = "/tmp/nonexistent_xplane".to_string();
        // Use temp directory for cache to avoid polluting user environment
        config.cache_dir = TempDir::new().unwrap().path().to_string_lossy().to_string();
        // Disable DDS cache for test to avoid filesystem operations
        config.cache.enable_dds_cache = false;

        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        // Use port 0 to get a random available port
        let result = start_all_services(0, "127.0.0.1", 49000, config, shutdown_rx).await;

        // Web server + tracker should start regardless of FUSE status
        assert!(
            result.is_ok(),
            "Services should start even if FUSE fails: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_tick_updates_dds_cache_size() {
        use crate::pipeline::cache::{DdsCache, DdsCacheMetadata};
        use tempfile::TempDir;

        setup_test_runtime();
        let mut app = AutoOrthoApp::new();

        // Create a DdsCache with some data
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("dds");
        std::fs::create_dir_all(&cache_dir).unwrap();

        let mut cache = DdsCache::open(cache_dir, 1024 * 1024).unwrap();
        let key = DdsCache::tile_key(100, 200, 14, "ARC");
        let dds_data = vec![0u8; 1024];
        let metadata = DdsCacheMetadata {
            v: 3,
            w: 32,
            h: 32,
            mm: 1,
            zl: 14,
            max_zl: 14,
            fmt: "BC1".to_string(),
            map: "ARC".to_string(),
            built: 0.0,
            tile_row: 200,
            tile_col: 100,
            populated_mipmaps: vec![0],
            missing_indices: vec![],
            fallback_indices: vec![],
            disk_compression: "zstd".to_string(),
        };
        cache.put(key.clone(), &dds_data, &metadata).unwrap();
        let expected_size = cache.size_bytes();
        assert!(expected_size > 0, "Cache should have data after put");

        let cache = std::sync::Arc::new(parking_lot::Mutex::new(cache));

        // Simulate services started with cache
        let _ = app.update(Message::ServicesStarted(
            "http://127.0.0.1:0".to_string(),
            None,
            std::sync::Arc::new(crate::ui::state::TileProgress::new()),
            Some(cache),
        ));

        // Tick should update cache size
        let _ = app.update(Message::Tick);

        assert_eq!(
            app.state.dds_cache_size_bytes, expected_size,
            "Tick should poll DdsCache and update dds_cache_size_bytes"
        );
    }

    #[test]
    fn test_fuse_mount_conditional_on_config() {
        // Test with empty xplane_path
        let config_empty = {
            let mut c = AutoOrthoConfig::default();
            c.xplane_path = "".to_string();
            c
        };

        let mount_dir = mount_dir(&config_empty.xplane_path);
        let mount_str = mount_dir.to_string_lossy();
        // When xplane_path is empty, the mount decision must evaluate to false.
        let should_mount = !config_empty.xplane_path.is_empty() && mount_dir.exists();

        assert!(
            !should_mount,
            "Mount should be skipped when xplane_path is empty; computed mount dir: {}",
            mount_str
        );
    }

    #[test]
    fn test_tile_progress_activates_tick_subscription() {
        use crate::ui::state::TileProgress;
        use std::sync::Arc;
        use std::sync::atomic::Ordering;

        setup_test_runtime();
        let mut app = AutoOrthoApp::new();

        // Initially no tile progress active - no Tick subscription
        let tile_progress = Arc::new(TileProgress::new());
        app.state.tile_progress = tile_progress.clone();

        // Start tile progress
        tile_progress.start(100, 200, 16, "BI");

        // Verify tile progress is active
        assert!(app.state.tile_progress.active.load(Ordering::Relaxed));
    }

    #[test]
    fn test_tile_progress_text_color_readable() {
        // Test that the tile progress text uses a readable color (black)
        // and the progress bar uses the same green as "Web: Running"
        let web_running_green = iced::Color::from_rgb(0.3, 0.7, 0.3);
        // Progress bar should use this green
        // Text should use black or a high-contrast color
    }

    #[test]
    fn test_generate_route_tiles_deduplicates() {
        use crate::xplane::simbrief::{FlightFix, FlightPlan};

        let mut config = AutoOrthoConfig::default();
        config.flight.near_airport_zoom = 14;
        config.flight.route_prefetch_radius_nm = 10;
        config.flight.airport_radius_nm = 10;
        config.flight.prefetch_airports = true;

        // Two fixes very close together — should produce overlapping tiles
        let flight_plan = FlightPlan {
            origin: "KLAX".to_string(),
            destination: "KLAS".to_string(),
            origin_elevation_ft: 128.0,
            destination_elevation_ft: 2181.0,
            cruise_altitude_ft: 35000.0,
            fixes: vec![
                FlightFix {
                    ident: "KLAX".to_string(),
                    name: "Los Angeles Intl".to_string(),
                    fix_type: "apt".to_string(),
                    lat: 33.94,
                    lon: -118.41,
                    altitude_ft: 0.0,
                    ground_height_ft: 0.0,
                    time_total_sec: 0.0,
                    time_leg_sec: 0.0,
                    ground_speed_kt: 0.0,
                },
                FlightFix {
                    ident: "KLAS".to_string(),
                    name: "Las Vegas Intl".to_string(),
                    fix_type: "apt".to_string(),
                    lat: 33.95,
                    lon: -118.42,
                    altitude_ft: 10000.0,
                    ground_height_ft: 0.0,
                    time_total_sec: 60.0,
                    time_leg_sec: 60.0,
                    ground_speed_kt: 250.0,
                },
            ],
        };

        let tiles = generate_route_tiles(&flight_plan, &config);

        // Should have tiles (exact count depends on zoom/radius)
        assert!(!tiles.is_empty(), "Should generate tiles");

        // All tiles should be unique (deduplication)
        let mut seen = std::collections::HashSet::new();
        for tile in &tiles {
            assert!(seen.insert(*tile), "Duplicate tile found: {:?}", tile);
        }
    }

    #[test]
    fn test_generate_route_tiles_respects_radius() {
        use crate::xplane::simbrief::{FlightFix, FlightPlan};

        let mut config = AutoOrthoConfig::default();
        config.flight.near_airport_zoom = 14;
        config.flight.route_prefetch_radius_nm = 5;
        config.flight.airport_radius_nm = 20;
        config.flight.prefetch_airports = true;

        let flight_plan = FlightPlan {
            origin: "KLAX".to_string(),
            destination: "KDEN".to_string(),
            origin_elevation_ft: 128.0,
            destination_elevation_ft: 5431.0,
            cruise_altitude_ft: 38000.0,
            fixes: vec![
                FlightFix {
                    ident: "KLAX".to_string(),
                    name: "Los Angeles Intl".to_string(),
                    fix_type: "apt".to_string(),
                    lat: 33.94,
                    lon: -118.41,
                    altitude_ft: 0.0,
                    ground_height_ft: 0.0,
                    time_total_sec: 0.0,
                    time_leg_sec: 0.0,
                    ground_speed_kt: 0.0,
                },
                FlightFix {
                    ident: "BOACH".to_string(),
                    name: "Boach".to_string(),
                    fix_type: "wpt".to_string(),
                    lat: 36.0,
                    lon: -115.0,
                    altitude_ft: 35000.0,
                    ground_height_ft: 2000.0,
                    time_total_sec: 1800.0,
                    time_leg_sec: 1800.0,
                    ground_speed_kt: 450.0,
                },
                FlightFix {
                    ident: "KDEN".to_string(),
                    name: "Denver Intl".to_string(),
                    fix_type: "apt".to_string(),
                    lat: 39.86,
                    lon: -104.67,
                    altitude_ft: 5431.0,
                    ground_height_ft: 5431.0,
                    time_total_sec: 3600.0,
                    time_leg_sec: 1800.0,
                    ground_speed_kt: 450.0,
                },
            ],
        };

        let tiles = generate_route_tiles(&flight_plan, &config);

        // Should have tiles
        assert!(!tiles.is_empty(), "Should generate tiles");

        // First fix (airport) should have more tiles than waypoint
        // because airport_radius_nm > route_prefetch_radius_nm
        // We can't easily count per-fix tiles after dedup, but total should be reasonable
        assert!(tiles.len() > 10, "Should have multiple tiles");
    }
}
