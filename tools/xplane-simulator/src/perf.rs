// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Performance tracking for the X-Plane simulator.

use std::time::{Duration, Instant};

/// Per-tile and aggregate performance statistics.
pub struct PerfStats {
    start: Instant,
    tiles_read: u64,
    total_bytes: u64,
    last_read_latency: Duration,
    latencies: Vec<Duration>,
}

impl PerfStats {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            tiles_read: 0,
            total_bytes: 0,
            last_read_latency: Duration::ZERO,
            latencies: Vec::new(),
        }
    }

    pub fn record_read(&mut self, bytes: usize) {
        self.tiles_read += 1;
        self.total_bytes += bytes as u64;
    }

    pub fn record_latency(&mut self, elapsed: Duration) {
        self.last_read_latency = elapsed;
        self.latencies.push(elapsed);
    }

    pub fn tiles_read(&self) -> u64 {
        self.tiles_read
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    pub fn last_read_latency(&self) -> Duration {
        self.last_read_latency
    }

    pub fn total_time(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn average_latency(&self) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.latencies.iter().sum();
        total / self.latencies.len() as u32
    }

    pub fn p95_latency(&self) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort();
        let idx = (sorted.len() as f64 * 0.95) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    pub fn max_latency(&self) -> Duration {
        self.latencies
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::ZERO)
    }

    pub fn throughput_bytes_per_sec(&self) -> f64 {
        let elapsed = self.total_time().as_secs_f64();
        if elapsed > 0.0 {
            self.total_bytes as f64 / elapsed
        } else {
            0.0
        }
    }
}

impl Default for PerfStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_stats_new() {
        let stats = PerfStats::new();
        assert_eq!(stats.tiles_read(), 0);
        assert_eq!(stats.total_bytes(), 0);
    }

    #[test]
    fn test_record_read() {
        let mut stats = PerfStats::new();
        stats.record_read(1024);
        stats.record_read(2048);
        assert_eq!(stats.tiles_read(), 2);
        assert_eq!(stats.total_bytes(), 3072);
    }

    #[test]
    fn test_throughput() {
        let mut stats = PerfStats::new();
        stats.record_read(1_000_000);
        // Sleep briefly so we have non-zero elapsed time
        std::thread::sleep(Duration::from_millis(10));
        let throughput = stats.throughput_bytes_per_sec();
        assert!(throughput > 0.0);
    }

    #[test]
    fn test_p95_latency() {
        let mut stats = PerfStats::new();
        for i in 0..100 {
            stats.record_latency(Duration::from_millis(i));
        }
        let p95 = stats.p95_latency();
        // p95 should be around 95ms
        assert!(p95 >= Duration::from_millis(90));
        assert!(p95 <= Duration::from_millis(100));
    }

    #[test]
    fn test_average_latency_empty() {
        let stats = PerfStats::new();
        assert_eq!(stats.average_latency(), Duration::ZERO);
    }
}
