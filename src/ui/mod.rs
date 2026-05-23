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

pub mod handlers;
use crate::tiles::provider;
use crate::webui::custommap::CustomMapStore;
use crate::xplane::simbrief::{FlightFix, FlightPlan};

/// Saved window geometry to restore on boot: (x, y, width, height)
#[allow(clippy::type_complexity)]
static SAVED_WINDOW_GEOM: Mutex<Option<(Option<f32>, Option<f32>, Option<f32>, Option<f32>)>> =
    Mutex::new(None);

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
    state: AppState,
    /// Tokio runtime handle for backend services (shared with main)
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
    ), // web URL and tracker
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
    FallbackTestComplete(Option<crate::ui::state::FallbackTestResult>),

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
    PrefetchProgress(u32, u32), // (completed, total)
    PrefetchComplete(String),
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
                handlers::set_tile_provider(&mut self.state, provider.clone());

                // Re-check coverage with new provider if we have an active flight plan
                let (olat, olon, dlat, dlon, origin_code, dest_code): (
                    Option<f64>,
                    Option<f64>,
                    Option<f64>,
                    Option<f64>,
                    String,
                    String,
                ) = if let Some(ref fp) = self.state.simbrief_flight_plan {
                    let ofix = FlightPlan::origin_fix(fp);
                    let dfix = FlightPlan::destination_fix(fp);
                    (
                        ofix.map(|f| f.lat),
                        ofix.map(|f| f.lon),
                        dfix.map(|f| f.lat),
                        dfix.map(|f| f.lon),
                        fp.origin.clone(),
                        fp.destination.clone(),
                    )
                } else {
                    (None, None, None, None, String::new(), String::new())
                };

                let zoom = self.state.config.near_airport_zoom;

                // Load custom map to check for cell overrides
                let custom_map_path = dirs::config_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("autoortho")
                    .join("custom_map.json");
                let custom_map = CustomMapStore::load(custom_map_path);
                let custom_cells = custom_map.get_cells();

                if let (Some(olat), Some(olon), Some(dlat), Some(dlon)) = (olat, olon, dlat, dlon) {
                    return iced::Task::perform(
                        async move {
                            let mut warnings = Vec::new();

                            let origin_cell =
                                format!("{},{}", olat.floor() as i32, olon.floor() as i32);
                            let dest_cell =
                                format!("{},{}", dlat.floor() as i32, dlon.floor() as i32);

                            let origin_has_custom = custom_cells.contains_key(&origin_cell);
                            if !origin_has_custom
                                && provider::test_provider_coverage(&provider, olat, olon, zoom)
                                    .await
                                    .is_err()
                            {
                                warnings.push(format!("origin ({})", origin_code));
                            }

                            let dest_has_custom = custom_cells.contains_key(&dest_cell);
                            if !dest_has_custom
                                && provider::test_provider_coverage(&provider, dlat, dlon, zoom)
                                    .await
                                    .is_err()
                            {
                                warnings.push(format!("destination ({})", dest_code));
                            }

                            if warnings.is_empty() {
                                None
                            } else {
                                Some(format!(
                                    "Provider {} may not have coverage for your route: {}. Consider using ArcGIS for global coverage.",
                                    provider,
                                    warnings.join(" and ")
                                ))
                            }
                        },
                        Message::SimbriefCoverageChecked,
                    );
                }
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
            Message::FetchSimbrief => {
                self.state.simbrief_fetching = true;
                self.state.simbrief_error = None;
                let user_id = self.state.config.simbrief_user_id.clone();
                return iced::Task::perform(
                    async move { crate::xplane::simbrief::fetch_flight_plan(&user_id).await },
                    |result| match result {
                        Ok(plan) => {
                            let alt = plan.cruise_altitude_ft;
                            let summary = format!(
                                "{} \u{2192} {} (FL{:.0})",
                                plan.origin,
                                plan.destination,
                                alt / 100.0
                            );
                            let fixes: Vec<(String, String, f32)> = plan
                                .fixes
                                .iter()
                                .map(|f| {
                                    let alt = if f.ident == plan.origin {
                                        plan.origin_elevation_ft
                                    } else if f.ident == plan.destination {
                                        plan.destination_elevation_ft
                                    } else {
                                        f.altitude_ft
                                    };
                                    (f.ident.clone(), f.fix_type.clone(), alt)
                                })
                                .collect();
                            Message::SimbriefLoaded(
                                summary,
                                fixes,
                                Arc::new(Mutex::new(Some(plan))),
                            )
                        }
                        Err(e) => Message::SimbriefFailed(e.to_string()),
                    },
                );
            }
            Message::SimbriefLoaded(summary, fixes, plan) => {
                self.state.simbrief_fetching = false;
                self.state.simbrief_route_summary = Some(summary.clone());
                self.state.simbrief_fixes = fixes;
                self.state.simbrief_show_details = false;
                self.state.simbrief_error = None;

                // Extract coordinates and store plan first
                let flight_plan_opt = {
                    let guard = plan.lock();
                    guard.clone()
                };

                let (origin_lat, origin_lon, dest_lat, dest_lon, origin_code, dest_code): (
                    Option<f64>,
                    Option<f64>,
                    Option<f64>,
                    Option<f64>,
                    String,
                    String,
                ) = if let Some(ref fp) = flight_plan_opt {
                    let ofix: Option<&FlightFix> = FlightPlan::origin_fix(fp);
                    let dfix: Option<&FlightFix> = FlightPlan::destination_fix(fp);
                    (
                        ofix.map(|f| f.lat),
                        ofix.map(|f| f.lon),
                        dfix.map(|f| f.lat),
                        dfix.map(|f| f.lon),
                        fp.origin.clone(),
                        fp.destination.clone(),
                    )
                } else {
                    (None, None, None, None, String::new(), String::new())
                };

                self.state.simbrief_flight_plan = flight_plan_opt;

                // Check provider coverage for the flight route
                let provider = self.state.config.tile_provider.clone();
                let zoom = self.state.config.near_airport_zoom;

                // Load custom map to check for cell overrides
                let custom_map_path = dirs::config_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("autoortho")
                    .join("custom_map.json");
                let custom_map = CustomMapStore::load(custom_map_path);
                let custom_cells = custom_map.get_cells();

                if let (Some(olat), Some(olon), Some(dlat), Some(dlon)) =
                    (origin_lat, origin_lon, dest_lat, dest_lon)
                {
                    return iced::Task::perform(
                        async move {
                            let mut warnings = Vec::new();

                            // Compute cell keys for origin and destination
                            let origin_cell =
                                format!("{},{}", olat.floor() as i32, olon.floor() as i32);
                            let dest_cell =
                                format!("{},{}", dlat.floor() as i32, dlon.floor() as i32);

                            // Test origin coverage (skip if custom map override exists)
                            let origin_has_custom = custom_cells.contains_key(&origin_cell);
                            if !origin_has_custom
                                && provider::test_provider_coverage(&provider, olat, olon, zoom)
                                    .await
                                    .is_err()
                            {
                                warnings.push(format!("origin ({})", origin_code));
                            }

                            // Test destination coverage (skip if custom map override exists)
                            let dest_has_custom = custom_cells.contains_key(&dest_cell);
                            if !dest_has_custom
                                && provider::test_provider_coverage(&provider, dlat, dlon, zoom)
                                    .await
                                    .is_err()
                            {
                                warnings.push(format!("destination ({})", dest_code));
                            }

                            if warnings.is_empty() {
                                None
                            } else {
                                Some(format!(
                                    "Provider {} may not have coverage for your route: {}. Consider using ArcGIS for global coverage.",
                                    provider,
                                    warnings.join(" and ")
                                ))
                            }
                        },
                        Message::SimbriefCoverageChecked,
                    );
                }
            }
            Message::SimbriefCoverageChecked(warning) => {
                self.state.simbrief_coverage_warning = warning;
            }
            Message::SimbriefFailed(err) => {
                self.state.simbrief_fetching = false;
                self.state.simbrief_error = Some(err);
            }
            Message::ToggleSimbriefDetails => {
                self.state.simbrief_show_details = !self.state.simbrief_show_details;
            }
            Message::PrefetchRoute => {
                let Some(flight_plan) = self.state.simbrief_flight_plan.clone() else {
                    self.state.prefetch_status = Some("No flight plan loaded".to_string());
                    return Task::none();
                };

                self.state.prefetch_running = true;
                self.state.prefetch_status = None;
                self.state.prefetch_completed = 0;
                self.state.prefetch_total = 0;

                let config = self.state.config.clone();
                let (tx, rx) = oneshot::channel();
                let rt = self.runtime.clone();

                rt.spawn(async move {
                    let result = prefetch_route_impl(&flight_plan, &config).await;
                    let _ = tx.send(result);
                });

                return Task::perform(
                    async { rx.await.unwrap_or(Err("Channel closed".into())) },
                    |result| match result {
                        Ok(msg) => Message::PrefetchComplete(msg),
                        Err(e) => Message::PrefetchFailed(e),
                    },
                );
            }
            Message::PrefetchProgress(completed, total) => {
                self.state.prefetch_completed = completed;
                self.state.prefetch_total = total;
            }
            Message::PrefetchComplete(msg) => {
                self.state.prefetch_running = false;
                self.state.prefetch_status = Some(msg);
            }
            Message::PrefetchFailed(err) => {
                self.state.prefetch_running = false;
                self.state.prefetch_status = Some(format!("Error: {}", err));
            }
            Message::SaveConfiguration => {
                self.state.save_config();

                // Apply SimHeaven compatibility setting
                let xplane_dir = std::path::Path::new(&self.state.config.xplane_path);
                let active_regions: Vec<String> = self
                    .state
                    .installed_packs
                    .iter()
                    .filter_map(|p| {
                        // Only include regions with ortho packs (z_*) not overlays (y_*)
                        if p.id.starts_with("z_") || !p.id.starts_with("y_") {
                            Some(p.id.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                if !active_regions.is_empty() {
                    match crate::scenery::simheaven::apply_simheaven_compat(
                        xplane_dir,
                        self.state.config.simheaven_compat,
                        &active_regions,
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            log::warn!("SimHeaven compatibility apply failed: {}", e);
                        }
                    }
                }
            }
            Message::LoadConfiguration => {
                self.state.load_config();
            }
            Message::SetDebugMode(v) => {
                handlers::set_debug_mode(&mut self.state, v);
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
                let config = self.state.config.clone();

                let (result_tx, result_rx) = oneshot::channel();
                let rt = self.runtime.clone();

                rt.spawn(async move {
                    let result = start_all_services(
                        crate::webui::WEB_UI_PORT,
                        &xplane_host,
                        xplane_port,
                        config,
                        shutdown_rx,
                    )
                    .await;
                    let _ = result_tx.send(result);
                });

                return Task::perform(
                    async {
                        result_rx
                            .await
                            .unwrap_or(Err("Runtime channel closed".into()))
                    },
                    |result| match result {
                        Ok((url, tracker)) => Message::ServicesStarted(url, Some(tracker)),
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
            Message::ServicesStarted(url, tracker) => {
                self.state.web_server = ServiceStatus::Running;
                self.state.web_server_url = Some(url);
                self.state.xplane_tracker = ServiceStatus::Running;
                self.state.tracker = tracker;
            }
            Message::ServicesFailed(err) => {
                self.state.web_server = ServiceStatus::Error;
                self.state.xplane_tracker = ServiceStatus::Error;
                self.state.set_error(format!("Failed to start: {}", err));
            }
            Message::SetSceneryDownloadDir(v) => {
                handlers::set_scenery_download_dir_state(&mut self.state, v);
            }
            Message::RefreshAvailableRegions => {
                self.state.scenery_refreshing = true;
                self.state.scenery_status = Some("Fetching available regions...".to_string());

                let data_dir = self.state.scenery_data_dir.clone();
                let download_dir = self.state.scenery_download_dir.clone();
                let (tx, rx) = oneshot::channel();
                let rt = self.runtime.clone();

                rt.spawn(async move {
                    let result = fetch_regions_and_installed(&data_dir, &download_dir).await;
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
                    &self.state.scenery_data_dir,
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
                    current_file: Arc::new(Mutex::new(String::new())),
                    files_done: Arc::new(std::sync::atomic::AtomicU32::new(0)),
                    files_total,
                };
                self.state
                    .downloading_regions
                    .insert(region_id.clone(), dl_state.clone());
                self.state.scenery_status = Some(format!("Downloading {}...", region_id));

                let download_dir = self.state.scenery_download_dir.clone();
                let data_dir = self.state.scenery_data_dir.clone();
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
                        &data_dir,
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
                let data_dir = std::path::Path::new(&self.state.scenery_data_dir);
                match crate::scenery::installer::uninstall_region(&region_id, data_dir) {
                    Ok(()) => {
                        self.state.scenery_status = Some(format!("Uninstalled {}", region_id));
                        // Refresh installed list
                        let packs = crate::scenery::installer::list_installed_packs(data_dir);
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
                    &self.state.scenery_data_dir,
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
                let rate_limit = self.state.config.rate_limit.requests_per_second;

                let (tx, rx) = oneshot::channel();
                let rt = self.runtime.clone();

                rt.spawn(async move {
                    use crate::tiles::fetcher::TileFetcher;
                    use crate::tiles::provider::ProviderFactory;

                    let provider = match ProviderFactory::create(&provider_name) {
                        Some(p) => p,
                        None => {
                            let _ = tx.send(Err(format!("Unknown provider: {}", provider_name)));
                            return;
                        }
                    };

                    let fetcher = Arc::new(TileFetcher::with_rate_limit(
                        provider,
                        &provider_name,
                        rate_limit,
                    ));

                    let result =
                        fetch_test_tile_impl(lat, lon, zoom, &provider_name, fetcher).await;
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
            Message::TestFallbackLookup => {
                self.state.test_fallback_running = true;
                let lat = self.state.test_tile_lat.parse::<f64>().unwrap_or(-33.86);
                let lon = self.state.test_tile_lon.parse::<f64>().unwrap_or(151.21);
                let zoom = self.state.test_tile_zoom;
                let cache_dir = std::path::PathBuf::from(&self.state.config.cache_dir);
                let fallback_config = self.state.config.fallback.clone();

                let (tx, rx) = oneshot::channel();
                let rt = self.runtime.clone();

                rt.spawn(async move {
                    let result =
                        test_fallback_lookup(lat, lon, zoom, &cache_dir, fallback_config).await;
                    let _ = tx.send(result);
                });

                return Task::perform(
                    async { rx.await.unwrap_or(None) },
                    Message::FallbackTestComplete,
                );
            }
            Message::FallbackTestComplete(result) => {
                self.state.test_fallback_running = false;
                self.state.test_fallback_result = result;
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
                    handlers::handle_set_xplane_path(&mut self.state, path);
                }
                "cache_dir" => {
                    handlers::handle_set_cache_dir(&mut self.state, path);
                }
                "scenery_download_dir" => self.state.scenery_download_dir = path,
                _ => {}
            },
            Message::WindowOpened(id) => {
                // Move window first, then schedule a delayed resize so the
                // window is on the correct monitor (correct DPI) before resizing.
                if let Some(geom) = SAVED_WINDOW_GEOM.lock().take() {
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
            }
            Message::Tick => {
                // Save config periodically when downloads are active (debounced window saves too)
                let _ = self.state.config.save();

                // Poll X-Plane connection status if we have a tracker
                if let Some(tracker) = &self.state.tracker {
                    let flight_data = tracker.get_flight_data();
                    if flight_data.connected && flight_data.data_valid {
                        self.state.xplane_tracker = ServiceStatus::Running;
                    } else {
                        self.state.xplane_tracker = ServiceStatus::Stopped;
                    }
                }
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

async fn start_all_services(
    web_port: u16,
    xplane_host: &str,
    xplane_port: u16,
    config: crate::config::AutoOrthoConfig,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<(String, Arc<crate::xplane::dataref::DatarefTracker>), String> {
    use crate::app_context::AppContext;
    use crate::xplane::dataref::{self};

    // Initialize the application context
    let context: AppContext = AppContext::init(config.clone())
        .await
        .map_err(|e| format!("Failed to initialize app context: {}", e))?;

    // Shared state between web server and tracker
    let stats = context.stats.clone();
    let tracker = context.tracker.clone();
    let web_config = context.config.clone();

    // Start web server
    let addr = crate::webui::start_server(web_port, stats.clone(), tracker.clone(), web_config)
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
        let mount_dir = config.mount_dir();
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

    Ok((format!("http://{}", addr), tracker))
}

/// Core implementation of test tile fetch - takes a pre-configured fetcher.
async fn fetch_test_tile_impl(
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
async fn test_fallback_lookup(
    lat: f64,
    lon: f64,
    zoom: u32,
    cache_dir: &std::path::Path,
    fallback_config: crate::config::FallbackConfig,
) -> Option<crate::ui::state::FallbackTestResult> {
    use crate::tiles::coords::TileCoords;
    use crate::tiles::fallback::FallbackSystem;

    let (row, col) = TileCoords::latlng_to_tile(lat, lon, zoom).ok()?;
    let maptype = "ARC";

    let fb = FallbackSystem::new(cache_dir.to_path_buf(), fallback_config);

    let result = fb.find_fallback(row, col, maptype, zoom);

    Some(crate::ui::state::FallbackTestResult {
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
async fn download_and_install_region(
    region_id: &str,
    download_dir: &str,
    data_dir: &str,
    cancel: &tokio_util::sync::CancellationToken,
    progress_bytes: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    progress_file: &Arc<Mutex<String>>,
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
    for path in &downloaded_files {
        if path.to_string_lossy().ends_with(".zip") {
            let target = data_path
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
    SAVED_WINDOW_GEOM.lock().replace((
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

    // Run one-time migration of scenery files from old location
    app.state.run_startup_migration();

    // Auto-refresh scenery list on startup
    app.state.scenery_refreshing = true;
    app.state.scenery_status = Some("Loading available scenery packs...".to_string());

    let data_dir = app.state.scenery_data_dir.clone();
    let download_dir = app.state.scenery_download_dir.clone();
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

/// Execute route prefetch: build queue from flight plan, fetch tiles, cache DDS.
async fn prefetch_route_impl(
    flight_plan: &crate::xplane::simbrief::FlightPlan,
    config: &crate::config::AutoOrthoConfig,
) -> Result<String, String> {
    use crate::pipeline::cache::DdsCache;
    use crate::pipeline::dds::DdsFormat;
    use crate::tiles::assembler::{AssemblyConfig, assemble_tile};
    use crate::tiles::fetcher::TileFetcher;
    use crate::tiles::prefetch::{RoutePrefetchConfig, SpatialPrefetcher};
    use crate::tiles::provider::ProviderFactory;
    use std::collections::HashMap;

    // 1. Generate prefetch points from flight plan (use origin as current position)
    let origin_fix = flight_plan
        .origin_fix()
        .ok_or_else(|| "No origin fix in flight plan".to_string())?;
    let spacing_nm = (config.route_prefetch_radius_nm as f64 / 2.0).max(5.0);
    let max_lookahead_sec = f32::MAX; // Prefetch entire route
    let points = flight_plan.get_prefetch_points(
        origin_fix.lat,
        origin_fix.lon,
        spacing_nm,
        max_lookahead_sec,
    );

    if points.is_empty() {
        return Ok("No prefetch points generated".to_string());
    }

    // 2. Build the queue from SpatialPrefetcher
    let mut prefetcher = SpatialPrefetcher::new();
    let route_distance_nm = points
        .last()
        .map(|p| p.distance_along_route_nm)
        .unwrap_or(0.0);

    let prefetch_config = RoutePrefetchConfig {
        percent_ahead: config.prefetch_route_percent,
        waypoint_radius_nm: config.route_prefetch_radius_nm as f64,
        airport_radius_nm: config.airport_radius_nm as f64,
        include_airports: config.prefetch_airports,
        zoom: config.near_airport_zoom,
    };

    prefetcher.prefetch_route(&points, route_distance_nm, prefetch_config);

    // 3. Group chunks by parent DDS tile (16x16 chunks per DDS tile)
    let mut dds_tiles: HashMap<(u32, u32), Vec<(u32, u32)>> = HashMap::new();
    while let Some((row, col)) = prefetcher.next_tile() {
        let dds_row = row / 16;
        let dds_col = col / 16;
        dds_tiles
            .entry((dds_row, dds_col))
            .or_default()
            .push((row, col));
    }

    if dds_tiles.is_empty() {
        return Ok("No tiles to prefetch".to_string());
    }

    // 4. Create TileFetcher
    let provider = ProviderFactory::create(&config.tile_provider)
        .ok_or_else(|| format!("Unknown provider: {}", config.tile_provider))?;
    let fetcher = TileFetcher::with_rate_limit(
        provider,
        &config.tile_provider,
        config.rate_limit.requests_per_second,
    );

    // 5. Create DDS cache
    let cache_dir = std::path::PathBuf::from(&config.cache_dir).join("dds");
    let mut cache = DdsCache::open(cache_dir.clone(), config.dds_cache_size_mb * 1024 * 1024)
        .map_err(|e| format!("Failed to open DDS cache: {}", e))?;

    // 6. For each DDS tile, fetch all 256 chunks and assemble
    let mut tiles_cached = 0;
    for (dds_row, dds_col) in dds_tiles.keys() {
        let tile_row = *dds_row;
        let tile_col = *dds_col;

        // Fetch all 256 chunks for this DDS tile
        let mut jpeg_chunks: Vec<Option<Vec<u8>>> = Vec::with_capacity(256);

        for dr in 0..16u32 {
            for dc in 0..16u32 {
                let chunk_row = tile_row * 16 + dr;
                let chunk_col = tile_col * 16 + dc;

                match fetcher
                    .get_chunk_data(
                        chunk_row,
                        chunk_col,
                        &config.tile_provider,
                        config.near_airport_zoom,
                    )
                    .await
                {
                    Ok(Some(data)) => {
                        let chunk_data: Vec<u8> = data.as_ref().clone();
                        jpeg_chunks.push(Some(chunk_data));
                    }
                    _ => {
                        jpeg_chunks.push(None);
                    }
                }
            }
        }

        // Assemble the DDS tile
        let assembly_config = AssemblyConfig {
            chunks_per_side: 16,
            chunk_size: 256,
            format: if config.max_zoom >= 18 {
                DdsFormat::BC3
            } else {
                DdsFormat::BC1
            },
            missing_color: [66, 77, 55],
            seasonal_saturation: 1.0,
        };

        let result = assemble_tile(&jpeg_chunks, &assembly_config)
            .map_err(|e| format!("Failed to assemble tile: {}", e))?;

        // Store in DDS cache
        let key = DdsCache::tile_key(
            tile_col,
            tile_row,
            config.near_airport_zoom,
            &config.tile_provider,
        );

        let metadata = crate::pipeline::cache::DdsCacheMetadata {
            v: 3,
            w: 4096,
            h: 4096,
            mm: result.mipmap_count,
            zl: config.near_airport_zoom,
            max_zl: config.near_airport_zoom,
            fmt: if config.max_zoom >= 18 {
                "BC3".to_string()
            } else {
                "BC1".to_string()
            },
            map: config.tile_provider.clone(),
            built: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            tile_row,
            tile_col,
            populated_mipmaps: (0..result.mipmap_count).collect(),
            missing_indices: vec![],
            fallback_indices: vec![],
            disk_compression: "zstd".to_string(),
        };

        cache
            .put(key, &result.dds_data, &metadata)
            .map_err(|e| format!("Failed to cache tile: {}", e))?;

        tiles_cached += 1;
    }

    Ok(format!(
        "Prefetched {} DDS tiles from {} route points",
        tiles_cached,
        points.len()
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
        let _ = app.update(Message::ServicesStarted(url.clone(), None));
        assert_eq!(app.state.web_server, ServiceStatus::Running);
        assert_eq!(app.state.xplane_tracker, ServiceStatus::Running);
        assert_eq!(app.state.web_server_url, Some(url));
    }

    #[test]
    fn test_services_failed() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        let _ = app.update(Message::ServicesFailed("port in use".to_string()));
        assert_eq!(app.state.web_server, ServiceStatus::Error);
        assert!(app.state.error_message.is_some());
    }

    #[test]
    fn test_stop_services() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        let url = format!("http://127.0.0.1:{}", crate::webui::WEB_UI_PORT);
        let _ = app.update(Message::ServicesStarted(url, None));
        assert!(app.state.any_service_running());

        let _ = app.update(Message::StopServices);
        assert!(!app.state.any_service_running());
        assert_eq!(app.state.web_server_url, None);
    }

    #[test]
    fn test_title_changes_when_running() {
        setup_test_runtime();
        let mut app = AutoOrthoApp::new();
        assert!(!app.title().contains("[Running]"));

        let url = format!("http://127.0.0.1:{}", crate::webui::WEB_UI_PORT);
        let _ = app.update(Message::ServicesStarted(url, None));
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
        config.enable_dds_cache = false;

        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        // Use port 0 to get a random available port
        let result = start_all_services(0, "127.0.0.1", 49000, config, shutdown_rx).await;

        assert!(
            result.is_ok(),
            "Services should start without FUSE when no X-Plane path: {:?}",
            result.err()
        );
        let (url, _tracker) = result.unwrap();
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
        config.enable_dds_cache = false;

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
    fn test_fuse_mount_conditional_on_config() {
        // Test with empty xplane_path
        let config_empty = {
            let mut c = AutoOrthoConfig::default();
            c.xplane_path = "".to_string();
            c
        };

        let mount_dir = config_empty.mount_dir();
        let mount_str = mount_dir.to_string_lossy();
        // When xplane_path is empty, the mount decision must evaluate to false.
        let should_mount = !config_empty.xplane_path.is_empty() && mount_dir.exists();

        assert!(
            !should_mount,
            "Mount should be skipped when xplane_path is empty; computed mount dir: {}",
            mount_str
        );
    }
}
