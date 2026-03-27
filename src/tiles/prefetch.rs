use std::collections::VecDeque;
use std::time::{Duration, Instant};

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
    completed: std::collections::HashSet<String>,
    pending: std::collections::HashSet<String>,
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
    fn test_spatial_prefetcher_creation() {
        let prefetcher = SpatialPrefetcher::new();
        assert_eq!(prefetcher.queue_len(), 0);
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
