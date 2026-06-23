// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Stats service trait and implementations.
//!
//! This module provides the `StatsService` trait for abstracting telemetry
//! and statistics tracking. It enables testing stats-dependent logic
//! without the real StatsStore.

use async_trait::async_trait;
use thiserror::Error;

/// Errors that can occur during stats operations.
#[derive(Debug, Error)]
pub enum StatsServiceError {
    #[error("Stats error: {0}")]
    General(String),
}

/// Result type for stats service operations.
pub type StatsResult<T> = Result<T, StatsServiceError>;

/// Service trait for statistics and telemetry.
///
/// This trait abstracts over the real `StatsStore` and in-memory fakes,
/// enabling integration tests to verify stats recording without coupling
/// to the concrete implementation.
#[async_trait]
pub trait StatsService: Send + Sync {
    /// Record a tile download with its byte count.
    async fn record_download(&self, bytes: u64);

    /// Record a cache hit.
    async fn record_cache_hit(&self);

    /// Record a cache miss.
    async fn record_cache_miss(&self);

    /// Set the number of pending tiles.
    async fn set_pending_tiles(&self, count: u32);

    /// Set the number of completed tiles.
    async fn set_completed_tiles(&self, count: u32);

    /// Get a snapshot of current statistics.
    async fn snapshot(&self) -> crate::stats::StatsSnapshot;

    /// Calculate the cache hit ratio (0.0–1.0).
    /// Returns 0.0 if no data is available.
    async fn hit_ratio(&self) -> f64;

    /// Reset all statistics to zero.
    async fn clear(&self);
}

/// Production implementation backed by StatsStore.
pub struct StatsServiceImpl {
    store: std::sync::Arc<crate::stats::StatsStore>,
}

impl StatsServiceImpl {
    /// Create a new StatsServiceImpl wrapping an existing StatsStore.
    pub fn new(store: std::sync::Arc<crate::stats::StatsStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl StatsService for StatsServiceImpl {
    async fn record_download(&self, bytes: u64) {
        self.store.record_download(bytes);
    }

    async fn record_cache_hit(&self) {
        self.store.record_cache_hit();
    }

    async fn record_cache_miss(&self) {
        self.store.record_cache_miss();
    }

    async fn set_pending_tiles(&self, count: u32) {
        self.store.set_pending_tiles(count);
    }

    async fn set_completed_tiles(&self, count: u32) {
        self.store.set_completed_tiles(count);
    }

    async fn snapshot(&self) -> crate::stats::StatsSnapshot {
        self.store.snapshot()
    }

    async fn hit_ratio(&self) -> f64 {
        self.store.hit_ratio()
    }

    async fn clear(&self) {
        self.store.clear();
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Fake stats service for testing without real StatsStore.
    #[derive(Debug, Clone)]
    pub struct FakeStatsService {
        snapshot: Arc<Mutex<crate::stats::StatsSnapshot>>,
    }

    impl FakeStatsService {
        pub fn new() -> Self {
            Self {
                snapshot: Arc::new(Mutex::new(crate::stats::StatsSnapshot::default())),
            }
        }
    }

    impl Default for FakeStatsService {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl StatsService for FakeStatsService {
        async fn record_download(&self, bytes: u64) {
            let mut snap = self.snapshot.lock().await;
            snap.tiles_downloaded += 1;
            snap.bytes_downloaded += bytes;
        }

        async fn record_cache_hit(&self) {
            let mut snap = self.snapshot.lock().await;
            snap.cache_hits += 1;
        }

        async fn record_cache_miss(&self) {
            let mut snap = self.snapshot.lock().await;
            snap.cache_misses += 1;
        }

        async fn set_pending_tiles(&self, count: u32) {
            let mut snap = self.snapshot.lock().await;
            snap.tiles_pending = count;
        }

        async fn set_completed_tiles(&self, count: u32) {
            let mut snap = self.snapshot.lock().await;
            snap.tiles_completed = count;
        }

        async fn snapshot(&self) -> crate::stats::StatsSnapshot {
            self.snapshot.lock().await.clone()
        }

        async fn hit_ratio(&self) -> f64 {
            let snap = self.snapshot.lock().await;
            let total = snap.cache_hits + snap.cache_misses;
            if total > 0 {
                snap.cache_hits as f64 / total as f64
            } else {
                0.0
            }
        }

        async fn clear(&self) {
            let mut snap = self.snapshot.lock().await;
            *snap = crate::stats::StatsSnapshot::default();
        }
    }

    #[tokio::test]
    async fn test_fake_stats_record_download() {
        let stats = FakeStatsService::new();
        stats.record_download(1024).await;
        stats.record_download(2048).await;

        let snap = stats.snapshot().await;
        assert_eq!(snap.tiles_downloaded, 2);
        assert_eq!(snap.bytes_downloaded, 3072);
    }

    #[tokio::test]
    async fn test_fake_stats_cache_hits() {
        let stats = FakeStatsService::new();
        stats.record_cache_hit().await;
        stats.record_cache_hit().await;
        stats.record_cache_miss().await;

        let snap = stats.snapshot().await;
        assert_eq!(snap.cache_hits, 2);
        assert_eq!(snap.cache_misses, 1);
    }

    #[tokio::test]
    async fn test_fake_stats_hit_ratio() {
        let stats = FakeStatsService::new();
        stats.record_cache_hit().await;
        stats.record_cache_hit().await;
        stats.record_cache_miss().await;

        let ratio = stats.hit_ratio().await;
        assert!((ratio - 0.666).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_fake_stats_hit_ratio_no_data() {
        let stats = FakeStatsService::new();
        assert_eq!(stats.hit_ratio().await, 0.0);
    }

    #[tokio::test]
    async fn test_fake_stats_pending_tiles() {
        let stats = FakeStatsService::new();
        stats.set_pending_tiles(5).await;

        let snap = stats.snapshot().await;
        assert_eq!(snap.tiles_pending, 5);
    }

    #[tokio::test]
    async fn test_fake_stats_clear() {
        let stats = FakeStatsService::new();
        stats.record_download(1024).await;
        stats.record_cache_hit().await;

        stats.clear().await;

        let snap = stats.snapshot().await;
        assert_eq!(snap.tiles_downloaded, 0);
        assert_eq!(snap.cache_hits, 0);
    }

    /// Test that StatsService works as a trait object.
    #[tokio::test]
    async fn test_stats_service_trait_object() {
        let fake = FakeStatsService::new();
        let service: Box<dyn StatsService> = Box::new(fake);

        service.record_download(512).await;
        let snap = service.snapshot().await;
        assert_eq!(snap.tiles_downloaded, 1);
    }

    /// Test that we can swap implementations without changing client code.
    #[tokio::test]
    async fn test_stats_service_impl_swap() {
        async fn get_hit_ratio<S: StatsService>(stats: &S) -> f64 {
            stats.hit_ratio().await
        }

        let fake = FakeStatsService::new();
        assert_eq!(get_hit_ratio(&fake).await, 0.0);
    }

    /// Test that production StatsServiceImpl wraps StatsStore correctly.
    #[tokio::test]
    async fn test_stats_service_impl_wraps_store() {
        let store = std::sync::Arc::new(crate::stats::StatsStore::new());
        let service = StatsServiceImpl::new(store.clone());

        service.record_download(1024).await;
        service.record_cache_hit().await;

        let snap = service.snapshot().await;
        assert_eq!(snap.tiles_downloaded, 1);
        assert_eq!(snap.bytes_downloaded, 1024);
        assert_eq!(snap.cache_hits, 1);

        // Verify underlying store is updated
        let store_snap = store.snapshot();
        assert_eq!(store_snap.tiles_downloaded, 1);
    }
}
