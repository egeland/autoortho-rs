// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Fallback service trait and implementations.
//!
//! This module provides the `FallbackService` trait for abstracting tile
//! fallback resolution. It enables testing fallback logic without real
//! cache directories or filesystem access.

use async_trait::async_trait;
use thiserror::Error;

/// Errors that can occur during fallback operations.
#[derive(Debug, Error)]
pub enum FallbackServiceError {
    #[error("No fallback available")]
    NoFallback,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Fallback error: {0}")]
    General(String),
}

/// Result type for fallback service operations.
pub type FallbackResult<T> = Result<T, FallbackServiceError>;

/// Service trait for tile fallback resolution.
///
/// When a tile is missing at the requested zoom level, the fallback service
/// finds an alternative — a lower-zoom cached tile, a blurred version, or
/// a solid color.
#[async_trait]
pub trait FallbackService: Send + Sync {
    /// Find a fallback tile for the given coordinates and zoom level.
    ///
    /// Returns `Some((dds_data, actual_zoom))` if a fallback was found,
    /// or `None` if no fallback is available at any zoom level.
    async fn find_fallback(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        requested_zoom: u32,
    ) -> Option<(Vec<u8>, u32)>;

    /// Generate a solid-color fallback DDS tile.
    ///
    /// - `size`: Width and height in pixels (square)
    /// - `format`: DDS format (BC1, BC3)
    /// - `color`: RGB color tuple
    async fn solid_fallback(
        &self,
        size: u32,
        format: crate::pipeline::dds::DdsFormat,
        color: [u8; 3],
    ) -> Vec<u8>;

    /// Check if this fallback service is configured to provide fallbacks.
    /// Returns false for `FallbackLevel::Solid` or when cache fallback is disabled.
    async fn needs_fallback(&self) -> bool;
}

/// Production implementation backed by FallbackSystem.
#[cfg(feature = "fuse")]
pub struct FallbackServiceImpl {
    system: std::sync::Arc<parking_lot::Mutex<crate::tiles::fallback::FallbackSystem>>,
}

#[cfg(feature = "fuse")]
impl FallbackServiceImpl {
    /// Create a new FallbackServiceImpl wrapping a FallbackSystem.
    pub fn new(system: crate::tiles::fallback::FallbackSystem) -> Self {
        Self {
            system: std::sync::Arc::new(parking_lot::Mutex::new(system)),
        }
    }

    /// Create from an existing Arc<Mutex<FallbackSystem>>.
    pub fn from_arc(
        system: std::sync::Arc<parking_lot::Mutex<crate::tiles::fallback::FallbackSystem>>,
    ) -> Self {
        Self { system }
    }
}

#[cfg(feature = "fuse")]
#[async_trait]
impl FallbackService for FallbackServiceImpl {
    async fn find_fallback(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        requested_zoom: u32,
    ) -> Option<(Vec<u8>, u32)> {
        let system = self.system.lock();
        system.find_fallback(row, col, maptype, requested_zoom)
    }

    async fn solid_fallback(
        &self,
        size: u32,
        format: crate::pipeline::dds::DdsFormat,
        color: [u8; 3],
    ) -> Vec<u8> {
        // Build solid-color fallback directly (no need for FallbackSystem)
        crate::pipeline::dds::build_fallback_dds(size, size, format, color)
    }

    async fn needs_fallback(&self) -> bool {
        let system = self.system.lock();
        system.needs_fallback()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Fake fallback service for testing without real cache directories.
    #[derive(Debug, Clone)]
    pub struct FakeFallbackService {
        /// Map of (row, col, maptype, zoom) -> (dds_data, actual_zoom)
        fallbacks: Arc<Mutex<HashMap<(u32, u32, String, u32), (Vec<u8>, u32)>>>,
        /// Whether this service provides fallbacks
        needs_fallback: bool,
    }

    impl FakeFallbackService {
        pub fn new(needs_fallback: bool) -> Self {
            Self {
                fallbacks: Arc::new(Mutex::new(HashMap::new())),
                needs_fallback,
            }
        }

        /// Add a fallback entry for testing.
        pub async fn add_fallback(
            &self,
            row: u32,
            col: u32,
            maptype: &str,
            requested_zoom: u32,
            data: Vec<u8>,
            actual_zoom: u32,
        ) {
            let mut fallbacks = self.fallbacks.lock().await;
            fallbacks.insert(
                (row, col, maptype.to_string(), requested_zoom),
                (data, actual_zoom),
            );
        }
    }

    #[async_trait]
    impl FallbackService for FakeFallbackService {
        async fn find_fallback(
            &self,
            row: u32,
            col: u32,
            maptype: &str,
            requested_zoom: u32,
        ) -> Option<(Vec<u8>, u32)> {
            let fallbacks = self.fallbacks.lock().await;
            fallbacks
                .get(&(row, col, maptype.to_string(), requested_zoom))
                .cloned()
        }

        async fn solid_fallback(
            &self,
            size: u32,
            _format: crate::pipeline::dds::DdsFormat,
            color: [u8; 3],
        ) -> Vec<u8> {
            // Return minimal DDS with color info embedded
            let mut dds = vec![0u8; 148];
            dds[0..4].copy_from_slice(b"DDS ");
            // Store color in header area for test verification
            dds[144] = color[0];
            dds[145] = color[1];
            dds[146] = color[2];
            dds[147] = size as u8;
            dds
        }

        async fn needs_fallback(&self) -> bool {
            self.needs_fallback
        }
    }

    #[tokio::test]
    async fn test_fake_fallback_find() {
        let fb = FakeFallbackService::new(true);
        fb.add_fallback(100, 200, "ARC", 16, vec![0x41, 0x42, 0x43], 14)
            .await;

        let result = fb.find_fallback(100, 200, "ARC", 16).await;
        assert!(result.is_some());
        let (data, zoom) = result.unwrap();
        assert_eq!(data, vec![0x41, 0x42, 0x43]);
        assert_eq!(zoom, 14);
    }

    #[tokio::test]
    async fn test_fake_fallback_not_found() {
        let fb = FakeFallbackService::new(true);

        let result = fb.find_fallback(100, 200, "ARC", 16).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_fake_fallback_solid() {
        let fb = FakeFallbackService::new(true);

        let dds = fb
            .solid_fallback(4096, crate::pipeline::dds::DdsFormat::BC3, [100, 150, 200])
            .await;

        assert_eq!(&dds[0..4], b"DDS ");
        assert_eq!(dds[144], 100);
        assert_eq!(dds[145], 150);
        assert_eq!(dds[146], 200);
    }

    #[tokio::test]
    async fn test_fake_fallback_needs_fallback() {
        let fb_with = FakeFallbackService::new(true);
        assert!(fb_with.needs_fallback().await);

        let fb_without = FakeFallbackService::new(false);
        assert!(!fb_without.needs_fallback().await);
    }

    /// Test that FallbackService works as a trait object.
    #[tokio::test]
    async fn test_fallback_service_trait_object() {
        let fake = FakeFallbackService::new(true);
        let service: Box<dyn FallbackService> = Box::new(fake);

        let result = service.find_fallback(100, 200, "ARC", 16).await;
        assert!(result.is_none()); // No fallbacks registered
    }

    /// Test that we can swap implementations without changing client code.
    #[tokio::test]
    async fn test_fallback_service_impl_swap() {
        async fn check_fallback<S: FallbackService>(service: &S) -> bool {
            service.needs_fallback().await
        }

        let fake = FakeFallbackService::new(true);
        assert!(check_fallback(&fake).await);
    }

    /// Test that production FallbackServiceImpl wraps FallbackSystem correctly.
    #[cfg(feature = "fuse")]
    #[tokio::test]
    async fn test_fallback_service_impl_wraps_system() {
        use crate::tiles::fallback::{FallbackConfig, FallbackSystem};
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let system = FallbackSystem::new(tmp.path().to_path_buf(), FallbackConfig::default());
        let service = FallbackServiceImpl::new(system);

        // No fallback registered, should return None
        let result = service.find_fallback(100, 200, "ARC", 16).await;
        assert!(result.is_none());

        // Solid fallback should work
        let dds = service
            .solid_fallback(256, crate::pipeline::dds::DdsFormat::BC1, [20, 25, 15])
            .await;
        assert_eq!(&dds[0..4], b"DDS ");
    }
}
