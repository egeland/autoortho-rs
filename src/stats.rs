// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use parking_lot::Mutex;
use std::sync::Arc;

/// Statistics store for tile downloads and cache
#[derive(Debug, Clone)]
pub struct StatsSnapshot {
    pub tiles_downloaded: u64,
    pub bytes_downloaded: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub tiles_pending: u32,
    pub tiles_completed: u32,
}

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
            })),
        }
    }

    pub fn record_download(&self, bytes: u64) {
        let mut snap = self.snapshot.lock();
        snap.tiles_downloaded += 1;
        snap.bytes_downloaded += bytes;
    }

    pub fn record_cache_hit(&self) {
        let mut snap = self.snapshot.lock();
        snap.cache_hits += 1;
    }

    pub fn record_cache_miss(&self) {
        let mut snap = self.snapshot.lock();
        snap.cache_misses += 1;
    }

    pub fn set_pending_tiles(&self, count: u32) {
        let mut snap = self.snapshot.lock();
        snap.tiles_pending = count;
    }

    pub fn set_completed_tiles(&self, count: u32) {
        let mut snap = self.snapshot.lock();
        snap.tiles_completed = count;
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
        let mut snap = self.snapshot.lock();
        snap.tiles_downloaded = 0;
        snap.bytes_downloaded = 0;
        snap.cache_hits = 0;
        snap.cache_misses = 0;
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
