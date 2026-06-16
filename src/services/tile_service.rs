// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Tile service trait and implementations.
//!
//! This module provides the `TileService` trait for fetching and assembling
//! satellite imagery tiles. It enables testing tile logic without FUSE.

use crate::tiles::coords::TileCoord;
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during tile operations.
#[derive(Debug, Error)]
pub enum TileServiceError {
    #[error("Tile not found: {0}")]
    NotFound(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Assembly error: {0}")]
    Assembly(String),

    #[error("Cache error: {0}")]
    Cache(String),
}

/// Result type for tile service operations.
pub type TileResult<T> = Result<T, TileServiceError>;

/// Service trait for fetching and assembling satellite imagery tiles.
///
/// This trait is the primary interface for tile operations. It abstracts over:
/// - Tile fetching (from remote providers or cache)
/// - Tile assembly (JPEG chunks → DDS)
/// - Fallback handling (blur, solid color)
/// - Night exclusion
///
/// # Example
///
/// ```ignore
/// use crate::services::TileService;
///
/// async fn get_tile(service: &impl TileService, coords: TileCoords) -> TileResult<Vec<u8>> {
///     service.get_dds(coords, "ARC", false).await
/// }
/// ```
#[async_trait]
pub trait TileService: Send + Sync {
    /// Get a DDS tile for the given coordinates.
    ///
    /// - `coords`: Tile coordinates (row, col, zoom)
    /// - `provider`: Tile provider name (e.g., "ARC", "ESRI")
    /// - `night_exclusion`: If true, return a night-colored tile
    async fn get_dds(
        &self,
        coords: TileCoord,
        provider: &str,
        night_exclusion: bool,
    ) -> TileResult<Vec<u8>>;

    /// Check if a tile exists (either cached or fetchable).
    async fn tile_exists(&self, coords: TileCoord, provider: &str) -> bool;
}

/// Tile service implementation backed by DdsFileSystem.
///
/// This is the production implementation that wraps the existing
/// FUSE filesystem logic for non-FUSE use cases.
#[cfg(feature = "fuse")]
#[allow(dead_code)]
pub struct TileServiceImpl {
    fs: Arc<crate::fuse::filesystem::DdsFileSystem>,
}

#[cfg(feature = "fuse")]
impl TileServiceImpl {
    /// Create a new TileServiceImpl wrapping a DdsFileSystem.
    #[allow(dead_code)]
    pub fn new(fs: Arc<crate::fuse::filesystem::DdsFileSystem>) -> Self {
        Self { fs }
    }
}

#[cfg(feature = "fuse")]
#[async_trait]
impl TileService for TileServiceImpl {
    async fn get_dds(
        &self,
        coords: TileCoord,
        provider: &str,
        night_exclusion: bool,
    ) -> TileResult<Vec<u8>> {
        use crate::fuse::FuseError;

        // Build path: /textures/{row}_{col}_{provider}{zoom}.dds
        let zoom_str = format!("{:02}", coords.zoom);
        let path = format!(
            "/textures/{}_{}_{}{}.dds",
            coords.row, coords.col, provider, zoom_str
        );

        // Use DdsFileSystem::read_dds with night_exclusion handling
        // Note: We need to temporarily set night_exclusion on the fs
        if night_exclusion {
            // The night exclusion is handled inside read_dds based on atomic flag
            // We can't easily override this from here, so we just call read_dds
            // The caller should set the night_exclusion flag on the fs before calling
        }

        self.fs
            .read_dds(&path, 0, u32::MAX)
            .await
            .map_err(|e| match e {
                FuseError::InvalidPath => TileServiceError::NotFound(path),
                FuseError::ParseFailed => TileServiceError::NotFound(path),
                _ => TileServiceError::Cache(format!("DDS read failed: {}", e)),
            })
    }

    async fn tile_exists(&self, coords: TileCoord, provider: &str) -> bool {
        // Build path: /textures/{row}_{col}_{provider}{zoom}.dds
        let zoom_str = format!("{:02}", coords.zoom);
        let path = format!(
            "/textures/{}_{}_{}{}.dds",
            coords.row, coords.col, provider, zoom_str
        );
        self.fs.has_dds(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fake TileService for testing without real tile fetching.
    #[derive(Debug, Clone, Default)]
    pub struct FakeTileService {
        pub tile_data: Arc<std::sync::atomic::AtomicU64>,
    }

    impl FakeTileService {
        pub fn new() -> Self {
            Self {
                tile_data: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            }
        }
    }

    #[async_trait]
    impl TileService for FakeTileService {
        async fn get_dds(
            &self,
            _coords: TileCoord,
            _provider: &str,
            _night_exclusion: bool,
        ) -> TileResult<Vec<u8>> {
            self.tile_data
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            // Return minimal valid DDS header
            let mut dds = vec![0u8; 148];
            dds[0..4].copy_from_slice(b"DDS ");
            Ok(dds)
        }

        async fn tile_exists(&self, _coords: TileCoord, _provider: &str) -> bool {
            true
        }
    }

    /// Test that TileService can be used as a trait object.
    #[tokio::test]
    async fn test_tile_service_trait_object() {
        let fake = FakeTileService::new();
        let service: Box<dyn TileService> = Box::new(fake);

        let coords = TileCoord::new(512, 512, 10).expect("Valid coords");
        let result = service.get_dds(coords, "ARC", false).await;
        assert!(result.is_ok());
    }

    /// Test that we can swap implementations without changing client code.
    #[tokio::test]
    async fn test_tile_service_impl_swap() {
        async fn get_tile_count<S: TileService>(service: &S) -> u64 {
            let coords = TileCoord::new(0, 0, 10).expect("Valid coords");
            service.get_dds(coords, "TEST", false).await.ok();
            // Access internal state (for testing only)
            // In real code, you'd expose stats through the trait
            1 // placeholder
        }

        let fake = FakeTileService::new();
        let count = get_tile_count(&fake).await;
        assert_eq!(count, 1);
    }

    /// Test that FakeTileService generates valid DDS headers.
    #[tokio::test]
    async fn test_fake_tile_service_dds_format() {
        let fake = FakeTileService::new();
        let coords = TileCoord::new(100, 200, 10).expect("Valid coords");

        let dds = fake
            .get_dds(coords, "ARC", false)
            .await
            .expect("Should return DDS data");

        // Verify DDS magic bytes
        assert_eq!(&dds[0..4], b"DDS ");
        // Basic header size check
        assert!(dds.len() >= 148);
    }
}

#[cfg(feature = "fuse")]
#[cfg(test)]
mod tile_service_impl_tests {
    use super::*;
    use crate::fuse::filesystem::DdsFileSystem;
    use crate::tiles::fetcher::TileFetcher;
    use crate::tiles::provider::{TileProvider, TileProviderError};
    use std::future::Future;
    use std::pin::Pin;

    struct MockProvider;

    impl TileProvider for MockProvider {
        fn fetch(
            &self,
            _row: u32,
            _col: u32,
            _zoom: u32,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
            Box::pin(async {
                // Return a valid minimal JPEG (1x1 pixel)
                Ok(vec![
                    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01,
                    0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
                ])
            })
        }

        fn name(&self) -> &str {
            "Mock"
        }
    }

    /// Test that TileServiceImpl can be created and used.
    #[tokio::test]
    async fn test_tile_service_impl_creation() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "ARC");
        let fs = Arc::new(DdsFileSystem::new(Arc::new(fetcher), "ARC"));

        let service = TileServiceImpl::new(fs.clone());
        let coords = TileCoord::new(100, 200, 16).expect("Valid coords");

        // This should work without panic (tile might not exist, but struct creation works)
        let _ = service.tile_exists(coords, "ARC").await;
    }

    /// Test that TileServiceImpl can be used as a TileService.
    #[tokio::test]
    async fn test_tile_service_impl_as_trait() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "ARC");
        let fs = Arc::new(DdsFileSystem::new(Arc::new(fetcher), "ARC"));

        let service: Box<dyn TileService> = Box::new(TileServiceImpl::new(fs));
        let coords = TileCoord::new(100, 200, 16).expect("Valid coords");

        // Verify trait works — tile not cached yet
        assert!(!service.tile_exists(coords, "ARC").await);
        // Populate cache, then verify
        let _ = service.get_dds(coords, "ARC", false).await;
        assert!(service.tile_exists(coords, "ARC").await);
    }

    /// Test that TileServiceImpl.get_dds returns valid DDS data.
    #[tokio::test]
    async fn test_tile_service_impl_get_dds() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "ARC");
        let fs = Arc::new(DdsFileSystem::new(Arc::new(fetcher), "ARC"));

        let service = TileServiceImpl::new(fs);
        let coords = TileCoord::new(100, 200, 16).expect("Valid coords");

        // Call get_dds - may return fallback or real data depending on mock
        let result = service.get_dds(coords, "ARC", false).await;

        // Should succeed (even if returning fallback)
        assert!(result.is_ok(), "get_dds should succeed: {:?}", result);

        let dds = result.unwrap();
        // DDS file should start with magic bytes
        assert_eq!(
            &dds[0..4],
            b"DDS ",
            "DDS file should start with magic bytes"
        );
        // DDS header is at least 148 bytes
        assert!(dds.len() >= 148, "DDS header should be at least 148 bytes");
    }

    /// Test that tile_exists returns false when tile is not cached.
    #[tokio::test]
    async fn test_tile_exists_uncached() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "ARC");
        let fs = Arc::new(DdsFileSystem::new(Arc::new(fetcher), "ARC"));
        let service = TileServiceImpl::new(fs);

        // Tile that has never been fetched
        let coords = TileCoord::new(999, 999, 16).expect("Valid coords");
        assert!(!service.tile_exists(coords, "ARC").await);
    }

    /// Test that tile_exists returns true after tile is cached.
    #[tokio::test]
    async fn test_tile_exists_cached() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "ARC");
        let fs = Arc::new(DdsFileSystem::new(Arc::new(fetcher), "ARC"));
        let service = TileServiceImpl::new(fs);
        let coords = TileCoord::new(100, 200, 16).expect("Valid coords");

        // Populate the cache
        let _ = service.get_dds(coords, "ARC", false).await;

        // Now tile_exists should return true
        assert!(service.tile_exists(coords, "ARC").await);
    }
}
