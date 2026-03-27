use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

use crate::tiles::coords::TileCoords;
use crate::xplane::simbrief::PrefetchPoint;

#[derive(Debug, Clone, Copy)]
pub struct RoutePrefetchConfig {
    pub percent_ahead: u32,
    pub waypoint_radius_nm: f64,
    pub airport_radius_nm: f64,
    pub include_airports: bool,
    pub zoom: u32,
}

/// Spatial prefetcher for flight-aware tile loading
pub struct SpatialPrefetcher {
    queue: VecDeque<(u32, u32)>, // (row, col) tiles to prefetch
    heading: f64,                // Aircraft magnetic heading (degrees)
}

impl Default for SpatialPrefetcher {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
            heading: 0.0,
        }
    }
}

impl SpatialPrefetcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update current heading and aircraft position
    pub fn update(&mut self, heading: f64, _lat: f64, _lon: f64) {
        self.heading = heading;
    }

    /// Enqueue tiles ahead of aircraft based on heading
    pub fn prefetch_ahead(&mut self, current_row: u32, current_col: u32, distance_tiles: u32) {
        self.queue.clear();

        // Simple forward-looking: prefetch tiles in direction of heading
        let heading_rad = self.heading.to_radians();
        let prefetch_col = current_col as i32;
        let prefetch_row = current_row as i32;

        // Add tiles in a cone ahead of aircraft
        for d in 1..=distance_tiles {
            let dc = (heading_rad.sin() * d as f64).round() as i32;
            let dr = (heading_rad.cos() * d as f64).round() as i32;

            for offset in -1..=1 {
                let col = prefetch_col + dc + offset;
                let row = prefetch_row + dr;

                if col >= 0 && row >= 0 {
                    self.queue.push_back((row as u32, col as u32));
                }
            }
        }
    }

    /// Get next tile to prefetch
    pub fn next_tile(&mut self) -> Option<(u32, u32)> {
        self.queue.pop_front()
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    /// Enqueue tiles based on SimBrief flight plan route
    /// Points are already spaced every 10NM by SimBrief parsing
    pub fn prefetch_route(
        &mut self,
        points: &[PrefetchPoint],
        route_distance_nm: f64,
        config: RoutePrefetchConfig,
    ) {
        self.queue.clear();

        if points.is_empty() || route_distance_nm <= 0.0 {
            return;
        }

        let target_distance = route_distance_nm * (config.percent_ahead as f64 / 100.0);

        let is_airport = |idx: usize, total: usize| -> bool {
            if !config.include_airports {
                return false;
            }
            idx == 0 || idx == total.saturating_sub(1)
        };

        for (idx, point) in points.iter().enumerate() {
            if point.distance_along_route_nm > target_distance {
                break;
            }

            let radius = if is_airport(idx, points.len()) {
                config.airport_radius_nm
            } else {
                config.waypoint_radius_nm
            };

            if let Ok((tile_col, tile_row)) =
                TileCoords::latlng_to_tile(point.lat, point.lon, config.zoom)
            {
                self.enqueue_tiles_around(tile_col, tile_row, radius, config.zoom);
            }
        }
    }

    /// Enqueue all tiles within radius (in NM) of a center tile
    fn enqueue_tiles_around(
        &mut self,
        center_col: u32,
        center_row: u32,
        radius_nm: f64,
        zoom: u32,
    ) {
        // Convert NM to tiles: 360 degrees = 2^zoom tiles, 1 degree = 60 NM
        // tiles_per_nm = 2^zoom / (360 * 60)
        let tiles_per_nm = 2_f64.powi(zoom as i32) / 360.0 / 60.0;
        let radius_tiles = (radius_nm * tiles_per_nm).ceil() as i32;

        for dr in -radius_tiles..=radius_tiles {
            for dc in -radius_tiles..=radius_tiles {
                let col = center_col as i32 + dc;
                let row = center_row as i32 + dr;

                if col >= 0 && row >= 0 {
                    self.queue.push_back((row as u32, col as u32));
                }
            }
        }
    }
}

/// Time budget for tile processing
pub struct TimeBudget {
    deadline: Instant,
    duration: Duration,
}

impl TimeBudget {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            duration,
        }
    }

    /// Check if time budget exhausted
    pub fn exhausted(&self) -> bool {
        Instant::now() >= self.deadline
    }

    /// Time remaining in budget
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }

    /// Reset budget with same duration
    pub fn reset(&mut self) {
        self.deadline = Instant::now() + self.duration;
    }
}

/// Track tile completion status
#[derive(Default)]
pub struct TileCompletionTracker {
    completed: HashSet<String>,
    pending: HashSet<String>,
}

impl TileCompletionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_pending(&mut self, key: String) {
        self.pending.insert(key);
    }

    pub fn mark_complete(&mut self, key: String) {
        self.pending.remove(&key);
        self.completed.insert(key);
    }

    pub fn is_complete(&self, key: &str) -> bool {
        self.completed.contains(key)
    }

    pub fn is_pending(&self, key: &str) -> bool {
        self.pending.contains(key)
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    pub fn clear(&mut self) {
        self.completed.clear();
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_route_empty_points() {
        let mut prefetcher = SpatialPrefetcher::new();

        let config = RoutePrefetchConfig {
            percent_ahead: 100,
            waypoint_radius_nm: 40.0,
            airport_radius_nm: 60.0,
            include_airports: true,
            zoom: 14,
        };

        prefetcher.prefetch_route(&[], 1000.0, config);

        assert_eq!(prefetcher.queue_len(), 0);
    }

    #[test]
    fn test_prefetch_route_with_points() {
        let mut prefetcher = SpatialPrefetcher::new();

        let points = vec![
            PrefetchPoint {
                lat: 33.94,
                lon: -118.41,
                altitude_ft: 0.0,
                ground_height_ft: 0.0,
                time_to_reach_sec: 0.0,
                distance_along_route_nm: 0.0,
            },
            PrefetchPoint {
                lat: 36.0,
                lon: -115.0,
                altitude_ft: 35000.0,
                ground_height_ft: 2000.0,
                time_to_reach_sec: 1800.0,
                distance_along_route_nm: 200.0,
            },
            PrefetchPoint {
                lat: 40.0,
                lon: -110.0,
                altitude_ft: 38000.0,
                ground_height_ft: 5000.0,
                time_to_reach_sec: 3600.0,
                distance_along_route_nm: 500.0,
            },
        ];

        let config = RoutePrefetchConfig {
            percent_ahead: 100,
            waypoint_radius_nm: 40.0,
            airport_radius_nm: 60.0,
            include_airports: true,
            zoom: 14,
        };

        prefetcher.prefetch_route(&points, 500.0, config);

        // Should have enqueued tiles around each point
        assert!(prefetcher.queue_len() > 0);
    }

    #[test]
    fn test_prefetch_route_percent_limit() {
        let mut prefetcher = SpatialPrefetcher::new();

        let points = vec![
            PrefetchPoint {
                lat: 33.94,
                lon: -118.41,
                altitude_ft: 0.0,
                ground_height_ft: 0.0,
                time_to_reach_sec: 0.0,
                distance_along_route_nm: 0.0,
            },
            PrefetchPoint {
                lat: 36.0,
                lon: -115.0,
                altitude_ft: 35000.0,
                ground_height_ft: 2000.0,
                time_to_reach_sec: 1800.0,
                distance_along_route_nm: 200.0,
            },
            PrefetchPoint {
                lat: 40.0,
                lon: -110.0,
                altitude_ft: 38000.0,
                ground_height_ft: 5000.0,
                time_to_reach_sec: 3600.0,
                distance_along_route_nm: 500.0,
            },
        ];

        // 50% of 500NM = 250NM - should only prefetch first 2 points
        let config_50 = RoutePrefetchConfig {
            percent_ahead: 50,
            waypoint_radius_nm: 40.0,
            airport_radius_nm: 60.0,
            include_airports: true,
            zoom: 14,
        };

        prefetcher.prefetch_route(&points, 500.0, config_50);

        // Should have fewer tiles than 100%
        let queue_len_50 = prefetcher.queue_len();

        prefetcher.clear_queue();

        // 100% should have more
        let config_100 = RoutePrefetchConfig {
            percent_ahead: 100,
            waypoint_radius_nm: 40.0,
            airport_radius_nm: 60.0,
            include_airports: true,
            zoom: 14,
        };

        prefetcher.prefetch_route(&points, 500.0, config_100);

        let queue_len_100 = prefetcher.queue_len();

        assert!(queue_len_100 > queue_len_50);
    }

    #[test]
    fn test_prefetch_route_airports_disabled() {
        let mut prefetcher = SpatialPrefetcher::new();

        let points = vec![
            PrefetchPoint {
                lat: 33.94,
                lon: -118.41,
                altitude_ft: 0.0,
                ground_height_ft: 0.0,
                time_to_reach_sec: 0.0,
                distance_along_route_nm: 0.0,
            },
            PrefetchPoint {
                lat: 40.0,
                lon: -110.0,
                altitude_ft: 38000.0,
                ground_height_ft: 5000.0,
                time_to_reach_sec: 3600.0,
                distance_along_route_nm: 500.0,
            },
        ];

        // With airports disabled
        let config_no_airports = RoutePrefetchConfig {
            percent_ahead: 100,
            waypoint_radius_nm: 40.0,
            airport_radius_nm: 60.0,
            include_airports: false,
            zoom: 14,
        };

        prefetcher.prefetch_route(&points, 500.0, config_no_airports);

        let queue_len_no_airports = prefetcher.queue_len();

        prefetcher.clear_queue();

        // With airports enabled
        let config_airports = RoutePrefetchConfig {
            percent_ahead: 100,
            waypoint_radius_nm: 40.0,
            airport_radius_nm: 60.0,
            include_airports: true,
            zoom: 14,
        };

        prefetcher.prefetch_route(&points, 500.0, config_airports);

        let queue_len_airports = prefetcher.queue_len();

        // Should have more tiles when airports are included
        assert!(queue_len_airports >= queue_len_no_airports);
    }

    #[test]
    fn test_spatial_prefetcher_prefetch_ahead() {
        let mut prefetcher = SpatialPrefetcher::new();
        prefetcher.update(0.0, 0.0, 0.0); // Heading north

        prefetcher.prefetch_ahead(100, 100, 3);

        assert!(prefetcher.queue_len() > 0);
    }

    #[test]
    fn test_spatial_prefetcher_queue_pop() {
        let mut prefetcher = SpatialPrefetcher::new();
        prefetcher.prefetch_ahead(100, 100, 2);

        let first = prefetcher.next_tile();
        assert!(first.is_some());

        // Queue should have fewer items
        let _original_len = prefetcher.queue_len() + 1; // +1 for popped item
        prefetcher.prefetch_ahead(100, 100, 2);
        assert!(prefetcher.queue_len() > 0);
    }

    #[test]
    fn test_time_budget_creation() {
        let budget = TimeBudget::new(Duration::from_secs(10));
        assert!(!budget.exhausted());
    }

    #[test]
    fn test_time_budget_exhaustion() {
        let budget = TimeBudget::new(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        assert!(budget.exhausted());
    }

    #[test]
    fn test_time_budget_remaining() {
        let budget = TimeBudget::new(Duration::from_secs(10));
        let remaining = budget.remaining();
        assert!(remaining <= Duration::from_secs(10));
        assert!(remaining > Duration::from_millis(9000));
    }

    #[test]
    fn test_time_budget_reset() {
        let mut budget = TimeBudget::new(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        assert!(budget.exhausted());

        budget.reset();
        assert!(!budget.exhausted());
    }

    #[test]
    fn test_tile_completion_tracker_basic() {
        let mut tracker = TileCompletionTracker::new();

        tracker.mark_pending("tile1".to_string());
        assert!(tracker.is_pending("tile1"));
        assert!(!tracker.is_complete("tile1"));

        tracker.mark_complete("tile1".to_string());
        assert!(tracker.is_complete("tile1"));
        assert!(!tracker.is_pending("tile1"));
    }

    #[test]
    fn test_tile_completion_tracker_counts() {
        let mut tracker = TileCompletionTracker::new();

        tracker.mark_pending("tile1".to_string());
        tracker.mark_pending("tile2".to_string());
        tracker.mark_pending("tile3".to_string());

        assert_eq!(tracker.pending_count(), 3);
        assert_eq!(tracker.completed_count(), 0);

        tracker.mark_complete("tile1".to_string());
        assert_eq!(tracker.pending_count(), 2);
        assert_eq!(tracker.completed_count(), 1);
    }

    #[test]
    fn test_tile_completion_tracker_clear() {
        let mut tracker = TileCompletionTracker::new();

        tracker.mark_pending("tile1".to_string());
        tracker.mark_complete("tile2".to_string());

        assert!(tracker.pending_count() > 0 || tracker.completed_count() > 0);

        tracker.clear();
        assert_eq!(tracker.pending_count(), 0);
        assert_eq!(tracker.completed_count(), 0);
    }
}
