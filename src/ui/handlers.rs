// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

//! UI message handlers - extracted from mod.rs for better organization.
//!
//! This module provides helper functions for handling UI messages,
//! extracted from the main update() function for better organization.

use crate::ui::state::{AppState, DownloadState, ServiceStatus};

pub fn handle_set_xplane_port(state: &mut AppState, port_str: &str) {
    if let Ok(port) = port_str.parse() {
        state.config.xplane_port = port;
    }
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
}
