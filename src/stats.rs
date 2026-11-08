// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use parking_lot::Mutex;
use std::sync::Arc;

/// Statistics store for tile downloads and cache
#[derive(Debug, Clone, Default)]
pub struct StatsSnapshot {
    pub tiles_downloaded: u64,
    pub bytes_downloaded: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub tiles_pending: u32,
    pub tiles_completed: u32,
    /// Tile read requests that arrived through the mount (X-Plane driven).
    pub mount_requests: u64,
    /// Unix ms of the last mount tile request (0 = never).
    pub last_mount_request_ms: u64,
}

/// Statistics store for tile downloads and cache.
/// Debug is required because the iced `Message` enum derives it.
#[derive(Debug)]
pub struct StatsStore {
    snapshot: Arc<Mutex<StatsSnapshot>>,
}

impl StatsStore {
    pub fn new() -> Self {
        Self {
            snapshot: Arc::new(Mutex::new(StatsSnapshot {
                tiles_downloaded: 0,
                bytes_downloaded: 0,
                cache_hits: 0,
                cache_misses: 0,
                tiles_pending: 0,
                tiles_completed: 0,
                mount_requests: 0,
                last_mount_request_ms: 0,
            })),
        }
    }

    /// Execute a closure with exclusive access to the snapshot.
    fn with_snapshot<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut StatsSnapshot) -> R,
    {
        f(&mut self.snapshot.lock())
    }

    pub fn record_download(&self, bytes: u64) {
        self.with_snapshot(|snap| {
            snap.tiles_downloaded += 1;
            snap.bytes_downloaded += bytes;
        })
    }

    pub fn record_cache_hit(&self) {
        self.with_snapshot(|snap| {
            snap.cache_hits += 1;
        })
    }

    pub fn record_cache_miss(&self) {
        self.with_snapshot(|snap| {
            snap.cache_misses += 1;
        })
    }

    /// Record a tile read request arriving through the mount.
    ///
    /// Called from the FUSE/Dokan read path only, so prefetch and the tile
    /// healer never light this up — the dashboard uses it to show that
    /// X-Plane is actively requesting tiles.
    pub fn record_mount_request(&self) {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.with_snapshot(|snap| {
            snap.mount_requests += 1;
            snap.last_mount_request_ms = now_ms;
        })
    }

    pub fn set_pending_tiles(&self, count: u32) {
        self.with_snapshot(|snap| {
            snap.tiles_pending = count;
        })
    }

    pub fn set_completed_tiles(&self, count: u32) {
        self.with_snapshot(|snap| {
            snap.tiles_completed = count;
        })
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        self.snapshot.lock().clone()
    }

    pub fn hit_ratio(&self) -> f64 {
        let snap = self.snapshot.lock();
        let total = snap.cache_hits + snap.cache_misses;
        if total > 0 {
            snap.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn clear(&self) {
        self.with_snapshot(|snap| {
            snap.tiles_downloaded = 0;
            snap.bytes_downloaded = 0;
            snap.cache_hits = 0;
            snap.cache_misses = 0;
        })
    }
}

impl Default for StatsStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_creation() {
        let stats = StatsStore::new();
        let snap = stats.snapshot();
        assert_eq!(snap.tiles_downloaded, 0);
    }

    #[test]
    fn test_record_download() {
        let stats = StatsStore::new();
        stats.record_download(1024);
        stats.record_download(2048);

        let snap = stats.snapshot();
        assert_eq!(snap.tiles_downloaded, 2);
        assert_eq!(snap.bytes_downloaded, 3072);
    }

    #[test]
    fn test_cache_hits() {
        let stats = StatsStore::new();
        stats.record_cache_hit();
        stats.record_cache_hit();
        stats.record_cache_miss();

        let snap = stats.snapshot();
        assert_eq!(snap.cache_hits, 2);
        assert_eq!(snap.cache_misses, 1);
    }

    #[test]
    fn test_hit_ratio() {
        let stats = StatsStore::new();
        stats.record_cache_hit();
        stats.record_cache_hit();
        stats.record_cache_miss();

        let ratio = stats.hit_ratio();
        assert!((ratio - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_hit_ratio_no_data() {
        let stats = StatsStore::new();
        assert_eq!(stats.hit_ratio(), 0.0);
    }

    #[test]
    fn test_pending_tiles() {
        let stats = StatsStore::new();
        stats.set_pending_tiles(5);
        let snap = stats.snapshot();
        assert_eq!(snap.tiles_pending, 5);
    }

    #[test]
    fn test_mount_requests_recorded_with_timestamp() {
        // X-Plane tile reads through the mount must be distinguishable from
        // prefetch/healer activity so the dashboard only pulses on sim requests.
        let stats = StatsStore::new();
        assert_eq!(stats.snapshot().mount_requests, 0);
        assert_eq!(stats.snapshot().last_mount_request_ms, 0);

        stats.record_mount_request();
        stats.record_mount_request();
        let snap = stats.snapshot();
        assert_eq!(snap.mount_requests, 2);
        assert!(snap.last_mount_request_ms > 0);
    }

    #[test]
    fn test_clear() {
        let stats = StatsStore::new();
        stats.record_download(1024);
        stats.record_cache_hit();

        stats.clear();

        let snap = stats.snapshot();
        assert_eq!(snap.tiles_downloaded, 0);
        assert_eq!(snap.cache_hits, 0);
    }
}
