// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Developer test state for tile testing and fallback testing.
//!
//! This module encapsulates the UI state for the Developer screen's
//! test tile and test fallback features.

/// Result of a fallback test.
#[derive(Debug, Clone)]
pub struct FallbackTestResult {
    pub found: bool,
    pub fallback_zoom: Option<u32>,
    pub requested_zoom: u32,
    pub tile_key: String,
    pub message: String,
}

/// Developer test tile and fallback test state.
#[derive(Debug, Clone)]
pub struct DevTestState {
    // Test tile state
    pub test_tile_lat: String,
    pub test_tile_lon: String,
    pub test_tile_zoom: u32,
    pub test_tile_status: Option<String>,
    pub test_tile_running: bool,
    /// RGBA pixel data for the test tile preview (width, height, data)
    pub test_tile_image: Option<(u32, u32, Vec<u8>)>,

    // Fallback test state
    pub test_fallback_running: bool,
    pub test_fallback_result: Option<FallbackTestResult>,
}

impl Default for DevTestState {
    fn default() -> Self {
        Self::new()
    }
}

impl DevTestState {
    pub fn new() -> Self {
        Self {
            test_tile_lat: String::new(),
            test_tile_lon: String::new(),
            test_tile_zoom: 10,
            test_tile_status: None,
            test_tile_running: false,
            test_tile_image: None,
            test_fallback_running: false,
            test_fallback_result: None,
        }
    }

    /// Set tile test coordinates.
    pub fn set_tile_coords(&mut self, lat: String, lon: String) {
        self.test_tile_lat = lat;
        self.test_tile_lon = lon;
    }

    /// Set tile test zoom level.
    pub fn set_tile_zoom(&mut self, zoom: u32) {
        self.test_tile_zoom = zoom;
    }

    /// Mark tile test as started.
    pub fn start_tile_test(&mut self) {
        self.test_tile_running = true;
        self.test_tile_status = Some("Testing...".to_string());
        self.test_tile_image = None;
    }

    /// Mark tile test as completed with result.
    pub fn complete_tile_test(&mut self, status: String, image: Option<(u32, u32, Vec<u8>)>) {
        self.test_tile_running = false;
        self.test_tile_status = Some(status);
        self.test_tile_image = image;
    }

    /// Mark tile test as failed.
    pub fn fail_tile_test(&mut self, error: String) {
        self.test_tile_running = false;
        self.test_tile_status = Some(format!("Error: {}", error));
        self.test_tile_image = None;
    }

    /// Mark fallback test as started.
    pub fn start_fallback_test(&mut self) {
        self.test_fallback_running = true;
        self.test_fallback_result = None;
    }

    /// Mark fallback test as completed with result.
    pub fn complete_fallback_test(&mut self, result: FallbackTestResult) {
        self.test_fallback_running = false;
        self.test_fallback_result = Some(result);
    }

    /// Mark fallback test as failed.
    pub fn fail_fallback_test(&mut self, error: String) {
        self.test_fallback_running = false;
        self.test_fallback_result = Some(FallbackTestResult {
            found: false,
            fallback_zoom: None,
            requested_zoom: 0,
            tile_key: String::new(),
            message: format!("Error: {}", error),
        });
    }

    /// Whether any test is currently running.
    pub fn any_running(&self) -> bool {
        self.test_tile_running || self.test_fallback_running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_test_state_creation() {
        let state = DevTestState::new();
        assert!(!state.test_tile_running);
        assert!(!state.test_fallback_running);
        assert!(!state.any_running());
        assert_eq!(state.test_tile_zoom, 10);
    }

    #[test]
    fn test_set_tile_coords() {
        let mut state = DevTestState::new();
        state.set_tile_coords("40.0".to_string(), "-74.0".to_string());
        assert_eq!(state.test_tile_lat, "40.0");
        assert_eq!(state.test_tile_lon, "-74.0");
    }

    #[test]
    fn test_set_tile_zoom() {
        let mut state = DevTestState::new();
        state.set_tile_zoom(16);
        assert_eq!(state.test_tile_zoom, 16);
    }

    #[test]
    fn test_tile_test_lifecycle() {
        let mut state = DevTestState::new();

        state.start_tile_test();
        assert!(state.test_tile_running);
        assert!(state.any_running());

        state.complete_tile_test("Success".to_string(), Some((100, 100, vec![0u8; 40000])));
        assert!(!state.test_tile_running);
        assert!(!state.any_running());
        assert_eq!(state.test_tile_status.as_deref(), Some("Success"));
        assert!(state.test_tile_image.is_some());
    }

    #[test]
    fn test_tile_test_failure() {
        let mut state = DevTestState::new();

        state.start_tile_test();
        state.fail_tile_test("Network error".to_string());

        assert!(!state.test_tile_running);
        assert_eq!(
            state.test_tile_status.as_deref(),
            Some("Error: Network error")
        );
        assert!(state.test_tile_image.is_none());
    }

    #[test]
    fn test_fallback_test_lifecycle() {
        let mut state = DevTestState::new();

        state.start_fallback_test();
        assert!(state.test_fallback_running);
        assert!(state.any_running());

        state.complete_fallback_test(FallbackTestResult {
            found: true,
            fallback_zoom: Some(14),
            requested_zoom: 16,
            tile_key: "100_200_BI_16".to_string(),
            message: "Found at zoom 14".to_string(),
        });

        assert!(!state.test_fallback_running);
        assert!(!state.any_running());
        let result = state.test_fallback_result.as_ref().unwrap();
        assert!(result.found);
        assert_eq!(result.fallback_zoom, Some(14));
    }

    #[test]
    fn test_fallback_test_failure() {
        let mut state = DevTestState::new();

        state.start_fallback_test();
        state.fail_fallback_test("No fallback found".to_string());

        assert!(!state.test_fallback_running);
        let result = state.test_fallback_result.as_ref().unwrap();
        assert!(!result.found);
    }

    #[test]
    fn test_any_running() {
        let mut state = DevTestState::new();
        assert!(!state.any_running());

        state.start_tile_test();
        assert!(state.any_running());

        state.complete_tile_test("done".to_string(), None);
        assert!(!state.any_running());

        state.start_fallback_test();
        assert!(state.any_running());
    }
}
