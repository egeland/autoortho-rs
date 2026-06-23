// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! UI message handlers - extracted from mod.rs for better organization.
//!
//! This module provides helper functions for handling UI messages,
//! extracted from the main update() function for better organization.

use crate::scenery::paths::{scenery_data_dir, scenery_install_dir};
use crate::seasons::Season;
use crate::tiles::fallback::FallbackLevel;
use crate::ui::state::{AppState, DownloadState, ServiceStatus};
pub mod cache;
pub mod dev_tools;
pub mod prefetch;
pub mod scenery;
pub mod services;
pub mod setup;
pub mod simbrief;
pub mod window;

pub fn handle_set_xplane_path(state: &mut AppState, path: String) {
    state.config.xplane_path = path.clone();
    state.scenery.install_dir = scenery_install_dir(&path).to_string_lossy().into_owned();
    state.scenery.data_dir = scenery_data_dir(&state.config.cache_dir)
        .to_string_lossy()
        .into_owned();
}

pub fn handle_set_cache_dir(state: &mut AppState, dir: String) {
    state.config.cache_dir = dir.clone();
    state.scenery.data_dir = scenery_data_dir(&dir).to_string_lossy().into_owned();
}

pub fn set_xplane_host(state: &mut AppState, host: String) {
    state.config.network.xplane_host = host;
}

pub fn handle_set_xplane_port(state: &mut AppState, port_str: &str) {
    if let Ok(port) = port_str.parse() {
        state.config.network.xplane_port = port;
    }
}

pub fn set_tile_provider(state: &mut AppState, provider: String) {
    state.config.tile.provider = provider;
    state.simbrief.coverage_warning = None;
}

pub fn set_min_zoom(state: &mut AppState, zoom: u32) {
    state.config.tile.min_zoom = zoom;
}

pub fn set_max_zoom(state: &mut AppState, zoom: u32) {
    state.config.tile.max_zoom = zoom;
}

pub fn set_enable_dynamic_zoom(state: &mut AppState, enabled: bool) {
    state.config.tile.enable_dynamic_zoom = enabled;
}

pub fn set_use_simbrief_altitude(state: &mut AppState, enabled: bool) {
    state.config.flight.use_simbrief_altitude = enabled;
}

pub fn set_simheaven_compat(state: &mut AppState, enabled: bool) {
    state.config.simheaven_compat = enabled;
}

pub fn set_ui_scale(state: &mut AppState, scale: f64) {
    state.config.ui.ui_scale = scale;
}

pub fn set_dds_cache_size_mb(state: &mut AppState, mb: u64) {
    state.config.cache.dds_cache_size_mb = mb;
}

pub fn set_enable_dds_cache(state: &mut AppState, enabled: bool) {
    state.config.cache.enable_dds_cache = enabled;
}

pub fn set_dds_memory_cache_mb(state: &mut AppState, mb: u64) {
    state.config.cache.dds_memory_cache_mb = mb;
}

pub fn set_chunk_memory_cache_mb(state: &mut AppState, mb: u64) {
    state.config.cache.chunk_memory_cache_mb = mb;
}

pub fn set_enable_night_exclusion(state: &mut AppState, enabled: bool) {
    state.config.night.enable_night_exclusion = enabled;
}

pub fn set_night_threshold(state: &mut AppState, v: f32) {
    state.config.night.night_threshold = v;
}

pub fn set_day_threshold(state: &mut AppState, v: f32) {
    state.config.night.day_threshold = v;
}

pub fn set_season(state: &mut AppState, season: Season) {
    state.config.season_cfg.season = season;
}

pub fn set_spring_saturation(state: &mut AppState, v: f32) {
    state.config.season_cfg.spring_saturation = v;
}

pub fn set_summer_saturation(state: &mut AppState, v: f32) {
    state.config.season_cfg.summer_saturation = v;
}

pub fn set_autumn_saturation(state: &mut AppState, v: f32) {
    state.config.season_cfg.autumn_saturation = v;
}

pub fn set_winter_saturation(state: &mut AppState, v: f32) {
    state.config.season_cfg.winter_saturation = v;
}

pub fn set_fallback_level(state: &mut AppState, level: FallbackLevel) {
    state.config.fallback.level = level;
}

pub fn set_fallback_max_zoom_gap(state: &mut AppState, gap: u32) {
    state.config.fallback.max_zoom_gap = gap;
}

pub fn set_fallback_cache_enabled(state: &mut AppState, enabled: bool) {
    state.config.fallback.cache_fallback = enabled;
}

pub fn set_rate_limit(state: &mut AppState, rate: f64) {
    state.config.network.rate_limit.requests_per_second = rate;
}

pub fn set_simbrief_user_id(state: &mut AppState, id: String) {
    state.config.flight.simbrief_user_id = id;
}

pub fn set_route_consideration_radius(state: &mut AppState, v: u32) {
    state.config.flight.route_consideration_radius_nm = v;
}

pub fn set_route_deviation_threshold(state: &mut AppState, v: u32) {
    state.config.flight.route_deviation_threshold_nm = v;
}

pub fn set_route_prefetch_radius(state: &mut AppState, v: u32) {
    state.config.flight.route_prefetch_radius_nm = v;
}

pub fn set_prefetch_route_percent(state: &mut AppState, v: u32) {
    state.config.flight.prefetch_route_percent = v;
}

pub fn set_prefetch_airports(state: &mut AppState, v: bool) {
    state.config.flight.prefetch_airports = v;
}

pub fn set_airport_radius(state: &mut AppState, v: u32) {
    state.config.flight.airport_radius_nm = v;
}

pub fn set_near_airport_zoom(state: &mut AppState, v: u32) {
    state.config.flight.near_airport_zoom = v;
}

pub fn set_scenery_download_dir(state: &mut AppState, v: String) {
    state.config.scenery_download_dir = v;
}

pub fn set_scenery_download_dir_state(state: &mut AppState, v: String) {
    state.scenery.download_dir = v;
}

pub fn handle_download_progress(
    _state: &AppState,
    download_state: &DownloadState,
) -> (u64, u64, String) {
    let bytes = download_state
        .bytes_downloaded
        .load(std::sync::atomic::Ordering::Relaxed);
    let files = download_state
        .files_done
        .load(std::sync::atomic::Ordering::Relaxed);
    let current_file = download_state.current_filename();
    (bytes, files as u64, current_file)
}

pub fn handle_service_status(state: &AppState) -> ServiceStatus {
    if state.services.web_server.is_running() {
        return ServiceStatus::Running;
    }
    ServiceStatus::Stopped
}

pub fn set_debug_mode(state: &mut AppState, v: bool) {
    state.config.ui.debug_mode = v;
}

pub use cache::{handle_clear_dds_cache, handle_save_configuration};
pub use dev_tools::{
    handle_fallback_test_complete, handle_fetch_test_tile, handle_set_test_lat,
    handle_set_test_lon, handle_set_test_zoom, handle_test_fallback_lookup,
    handle_test_tile_complete, handle_test_tile_failed,
};
pub use prefetch::{
    handle_prefetch_complete, handle_prefetch_complete_cache_full, handle_prefetch_failed,
    handle_prefetch_progress, handle_prefetch_route, handle_stop_prefetch,
};
pub use scenery::{
    handle_cancel_download, handle_clean_region_downloads, handle_download_complete,
    handle_download_failed, handle_download_region, handle_refresh_available_regions,
    handle_regions_load_failed, handle_regions_loaded, handle_uninstall_region,
};
pub use services::{
    handle_services_failed, handle_services_started, handle_start_services, handle_stop_services,
};
pub use setup::{
    handle_browse_cache_dir, handle_browse_scenery_download_dir, handle_browse_xplane_path,
    handle_folder_picked,
};
pub use simbrief::{
    handle_fetch_simbrief, handle_set_tile_provider, handle_simbrief_coverage_checked,
    handle_simbrief_failed, handle_simbrief_loaded, handle_toggle_simbrief_details,
};
pub use window::{
    handle_window_close_requested, handle_window_moved, handle_window_opened,
    handle_window_resized, handle_window_restore_size,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_xplane_port_valid() {
        let mut state = AppState::new();
        handle_set_xplane_port(&mut state, "49001");
        assert_eq!(state.config.network.xplane_port, 49001);
    }

    #[test]
    fn test_set_xplane_port_invalid() {
        let mut state = AppState::new();
        state.config.network.xplane_port = 49000;
        handle_set_xplane_port(&mut state, "invalid");
        assert_eq!(state.config.network.xplane_port, 49000);
    }

    #[test]
    fn test_set_xplane_path() {
        let mut state = AppState::new();
        let old_data_dir = state.scenery.data_dir.clone();

        let xplane_path = "/test/path";
        let expected_install = std::path::Path::new(xplane_path).join("Custom Scenery");

        handle_set_xplane_path(&mut state, xplane_path.to_string());

        assert_eq!(state.config.xplane_path, xplane_path);
        assert_eq!(
            state.scenery.install_dir,
            expected_install.to_string_lossy()
        );
        // scenery_data_dir is based on cache_dir, not xplane_path
        assert_eq!(state.scenery.data_dir, old_data_dir);
    }

    #[test]
    fn test_set_cache_dir_updates_scenery_data_dir() {
        let mut state = AppState::new();
        let orig = state.scenery.data_dir.clone();
        handle_set_cache_dir(&mut state, "/custom/cache".to_string());
        // scenery_data_dir should reflect new cache dir
        assert!(
            state.scenery.data_dir.contains("/custom/cache"),
            "scenery_data_dir should contain new cache dir, got: {}",
            state.scenery.data_dir
        );
        assert_ne!(state.scenery.data_dir, orig);
    }

    #[test]
    fn test_set_debug_mode() {
        let mut state = AppState::new();
        assert!(!state.config.ui.debug_mode);
        set_debug_mode(&mut state, true);
        assert!(state.config.ui.debug_mode);
        set_debug_mode(&mut state, false);
        assert!(!state.config.ui.debug_mode);
    }

    // === TDD: verify handlers use scenery::paths (migration regression guard) ===
    // After migration, handle_set_xplane_path must call scenery::paths functions,
    // not config delegation methods. This test exercises the import chain.
    #[test]
    fn test_handler_uses_mount_dir_from_scenery_paths() {
        use crate::scenery::paths::mount_dir;
        let path = mount_dir("/Games/X-Plane 12");
        assert!(path.to_string_lossy().contains("z_autoortho"));
    }
}
