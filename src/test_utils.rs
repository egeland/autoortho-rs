// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Common test utilities to reduce boilerplate across test modules.

use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary directory for testing, panics on failure.
pub fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Create a temporary directory with a specific prefix.
pub fn temp_dir_with_prefix(prefix: &str) -> TempDir {
    TempDir::with_prefix(prefix).expect("Failed to create temp directory")
}

/// Create a test config with default values in a temp directory.
/// Returns the config and the temp directory (must be kept alive).
pub fn test_config_in_temp() -> (crate::config::AutoOrthoConfig, TempDir) {
    let tmp = temp_dir();
    let mut config = crate::config::AutoOrthoConfig::default();
    config.cache_dir = tmp.path().join("cache").to_string_lossy().into_owned();
    config.scenery_download_dir = tmp.path().join("downloads").to_string_lossy().into_owned();
    (config, tmp)
}

/// Create a test AppContext with default config in a temp directory.
/// Returns the AppContext and temp directory (must be kept alive for async context).
#[cfg(test)]
pub async fn test_app_context() -> (crate::app_context::AppContext, TempDir) {
    let (config, tmp) = test_config_in_temp();
    let context = crate::app_context::AppContext::init(config)
        .await
        .expect("Failed to create test AppContext");
    (context, tmp)
}

/// Assert that a file exists at the given path.
pub fn assert_file_exists(path: &PathBuf) {
    assert!(path.exists(), "File does not exist: {:?}", path);
}

/// Assert that a file does not exist at the given path.
pub fn assert_file_not_exists(path: &PathBuf) {
    assert!(!path.exists(), "File should not exist: {:?}", path);
}

// --- Mock tile providers for tests ---

use crate::tiles::provider::{TileProvider, TileProviderError};
use std::future::Future;
use std::pin::Pin;

/// Minimal valid JPEG (22 bytes, 1x1 pixel). Returned by MockProvider.
pub const MINIMAL_JPEG: &[u8] = &[
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
    0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
];

/// Mock tile provider that returns a valid minimal JPEG for any request.
pub struct MockProvider;

impl TileProvider for MockProvider {
    fn fetch(
        &self,
        _row: u32,
        _col: u32,
        _zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async { Ok(MINIMAL_JPEG.to_vec()) })
    }

    fn name(&self) -> &str {
        "Mock"
    }
}

/// Mock tile provider that always fails with a network error.
pub struct FailingProvider;

impl TileProvider for FailingProvider {
    fn fetch(
        &self,
        _row: u32,
        _col: u32,
        _zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async { Err(TileProviderError::NetworkError("test failure".into())) })
    }

    fn name(&self) -> &str {
        "Failing"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_dir_creation() {
        let dir = temp_dir();
        assert!(dir.path().exists());
    }

    #[test]
    fn test_temp_dir_with_prefix() {
        let dir = temp_dir_with_prefix("autoortho_test");
        let path_str = dir.path().to_string_lossy();
        assert!(
            path_str.contains("autoortho_test"),
            "Prefix not found in: {}",
            path_str
        );
    }

    #[tokio::test]
    async fn test_test_config_in_temp() {
        let (config, _tmp) = test_config_in_temp();
        assert!(!config.cache_dir.is_empty());
        assert!(!config.scenery_download_dir.is_empty());
    }
}
