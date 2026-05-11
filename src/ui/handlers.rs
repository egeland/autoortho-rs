// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

//! UI message handlers - extracted from mod.rs for better organization.
//!
//! This module provides helper functions for handling UI messages,
//! extracted from the main update() function for better organization.

use crate::config::{FallbackLevel, Season};
use crate::ui::state::{AppState, DownloadState, ServiceStatus};

pub fn handle_set_xplane_path(state: &mut AppState, path: String) {
    state.config.xplane_path = path;
    state.scenery_install_dir = state
        .config
        .scenery_install_dir()
        .to_string_lossy()
        .into_owned();
    state.scenery_data_dir = state
        .config
        .scenery_data_dir()
        .to_string_lossy()
        .into_owned();
}

pub fn handle_set_cache_dir(state: &mut AppState, dir: String) {
    state.config.cache_dir = dir;
    state.scenery_data_dir = state
        .config
        .scenery_data_dir()
        .to_string_lossy()
        .into_owned();
}

pub fn set_xplane_host(state: &mut AppState, host: String) {
    state.config.xplane_host = host;
}

pub fn handle_set_xplane_port(state: &mut AppState, port_str: &str) {
    if let Ok(port) = port_str.parse() {
        state.config.xplane_port = port;
    }
}

pub fn set_tile_provider(state: &mut AppState, provider: String) {
    state.config.tile_provider = provider;
    state.simbrief_coverage_warning = None;
}

pub fn set_min_zoom(state: &mut AppState, zoom: u32) {
    state.config.min_zoom = zoom;
}

pub fn set_max_zoom(state: &mut AppState, zoom: u32) {
    state.config.max_zoom = zoom;
}

pub fn set_enable_dynamic_zoom(state: &mut AppState, enabled: bool) {
    state.config.enable_dynamic_zoom = enabled;
}

pub fn set_use_simbrief_altitude(state: &mut AppState, enabled: bool) {
    state.config.use_simbrief_altitude = enabled;
}

pub fn set_simheaven_compat(state: &mut AppState, enabled: bool) {
    state.config.simheaven_compat = enabled;
}

pub fn set_ui_scale(state: &mut AppState, scale: f64) {
    state.config.ui_scale = scale;
}

pub fn set_dds_cache_size_mb(state: &mut AppState, mb: u64) {
    state.config.dds_cache_size_mb = mb;
}

pub fn set_enable_dds_cache(state: &mut AppState, enabled: bool) {
    state.config.enable_dds_cache = enabled;
}

pub fn set_dds_memory_cache_mb(state: &mut AppState, mb: u64) {
    state.config.dds_memory_cache_mb = mb;
}

pub fn set_chunk_memory_cache_mb(state: &mut AppState, mb: u64) {
    state.config.chunk_memory_cache_mb = mb;
}

pub fn set_enable_night_exclusion(state: &mut AppState, enabled: bool) {
    state.config.enable_night_exclusion = enabled;
}

pub fn set_night_threshold(state: &mut AppState, v: f32) {
    state.config.night_threshold = v;
}

pub fn set_day_threshold(state: &mut AppState, v: f32) {
    state.config.day_threshold = v;
}

pub fn set_season(state: &mut AppState, season: Season) {
    state.config.season = season;
}

pub fn set_spring_saturation(state: &mut AppState, v: f32) {
    state.config.spring_saturation = v;
}

pub fn set_summer_saturation(state: &mut AppState, v: f32) {
    state.config.summer_saturation = v;
}

pub fn set_autumn_saturation(state: &mut AppState, v: f32) {
    state.config.autumn_saturation = v;
}

pub fn set_winter_saturation(state: &mut AppState, v: f32) {
    state.config.winter_saturation = v;
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
    state.config.rate_limit.requests_per_second = rate;
}

pub fn set_simbrief_user_id(state: &mut AppState, id: String) {
    state.config.simbrief_user_id = id;
}

pub fn set_route_consideration_radius(state: &mut AppState, v: u32) {
    state.config.route_consideration_radius_nm = v;
}

pub fn set_route_deviation_threshold(state: &mut AppState, v: u32) {
    state.config.route_deviation_threshold_nm = v;
}

pub fn set_route_prefetch_radius(state: &mut AppState, v: u32) {
    state.config.route_prefetch_radius_nm = v;
}

pub fn set_prefetch_route_percent(state: &mut AppState, v: u32) {
    state.config.prefetch_route_percent = v;
}

pub fn set_prefetch_airports(state: &mut AppState, v: bool) {
    state.config.prefetch_airports = v;
}

pub fn set_airport_radius(state: &mut AppState, v: u32) {
    state.config.airport_radius_nm = v;
}

pub fn set_near_airport_zoom(state: &mut AppState, v: u32) {
    state.config.near_airport_zoom = v;
}

pub fn set_scenery_download_dir(state: &mut AppState, v: String) {
    state.config.scenery_download_dir = v;
}

pub fn set_scenery_download_dir_state(state: &mut AppState, v: String) {
    state.scenery_download_dir = v;
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
    if state.web_server.is_running() {
        return ServiceStatus::Running;
    }
    ServiceStatus::Stopped
}

pub fn set_debug_mode(state: &mut AppState, v: bool) {
    state.config.debug_mode = v;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_xplane_port_valid() {
        let mut state = AppState::new();
        handle_set_xplane_port(&mut state, "49001");
        assert_eq!(state.config.xplane_port, 49001);
    }

    #[test]
    fn test_set_xplane_port_invalid() {
        let mut state = AppState::new();
        state.config.xplane_port = 49000;
        handle_set_xplane_port(&mut state, "invalid");
        assert_eq!(state.config.xplane_port, 49000);
    }

    #[test]
    fn test_set_xplane_path() {
        let mut state = AppState::new();
        let old_data_dir = state.scenery_data_dir.clone();

        handle_set_xplane_path(&mut state, "/test/path".to_string());

        assert_eq!(state.config.xplane_path, "/test/path");
        assert_eq!(state.scenery_install_dir, "/test/path/Custom Scenery");
        // scenery_data_dir is based on cache_dir, not xplane_path
        assert_eq!(state.scenery_data_dir, old_data_dir);
    }

    #[test]
    fn test_set_cache_dir_updates_scenery_data_dir() {
        let mut state = AppState::new();
        let orig = state.scenery_data_dir.clone();
        handle_set_cache_dir(&mut state, "/custom/cache".to_string());
        // scenery_data_dir should reflect new cache dir
        assert!(
            state.scenery_data_dir.contains("/custom/cache"),
            "scenery_data_dir should contain new cache dir, got: {}",
            state.scenery_data_dir
        );
        assert_ne!(state.scenery_data_dir, orig);
    }

    #[test]
    fn test_set_debug_mode() {
        let mut state = AppState::new();
        assert!(!state.config.debug_mode);
        set_debug_mode(&mut state, true);
        assert!(state.config.debug_mode);
        set_debug_mode(&mut state, false);
        assert!(!state.config.debug_mode);
    }
}
