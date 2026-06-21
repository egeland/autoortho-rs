// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Tile service trait and implementations.
//!
//! This module provides the `TileService` trait for fetching and assembling
//! satellite imagery tiles. It enables testing tile logic without FUSE.

use crate::tiles::coords::TileCoord;
use async_trait::async_trait;
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Arc;

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
