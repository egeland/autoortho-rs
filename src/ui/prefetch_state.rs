// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Route prefetch state.
//!
//! This module encapsulates the UI state for route prefetching —
//! downloading tiles along a flight route before they're needed.

/// Prefetch status for a single waypoint/fix in the flight plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointPrefetchStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

impl WaypointPrefetchStatus {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::NotStarted => "⏳",
            Self::InProgress => "🔄",
            Self::Completed => "✅",
            Self::Failed => "❌",
        }
    }
}

/// Shared waypoint prefetch progress (read by UI, written by background task).
#[derive(Debug, Default)]
pub struct WaypointPrefetchProgress {
    statuses: parking_lot::Mutex<Vec<WaypointPrefetchStatus>>,
}

impl WaypointPrefetchProgress {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn init(&self, count: usize) {
        *self.statuses.lock() = vec![WaypointPrefetchStatus::NotStarted; count];
    }
    pub fn set(&self, index: usize, status: WaypointPrefetchStatus) {
        let mut s = self.statuses.lock();
        if index < s.len() {
            s[index] = status;
        }
    }
    pub fn get_all(&self) -> Vec<WaypointPrefetchStatus> {
        self.statuses.lock().clone()
    }
}

/// Route prefetch state.
#[derive(Debug, Clone)]
pub struct PrefetchState {
    /// Whether a prefetch operation is currently running
    pub running: bool,
    /// Current status message
    pub status: Option<String>,
    /// Number of waypoints completed
    pub completed: u32,
    /// Total waypoints to prefetch
    pub total: u32,
    /// Cancellation token for the running prefetch
    pub cancel: Option<tokio_util::sync::CancellationToken>,
    /// Shared progress tracker (written by background task, read by Tick)
    pub waypoint_progress: std::sync::Arc<WaypointPrefetchProgress>,
    /// Snapshot of per-waypoint status for display in view
    pub waypoint_status: Vec<WaypointPrefetchStatus>,
}

impl Default for PrefetchState {
    fn default() -> Self {
        Self::new()
    }
}

impl PrefetchState {
    pub fn new() -> Self {
        Self {
            running: false,
            status: None,
            completed: 0,
            total: 0,
            cancel: None,
            waypoint_progress: std::sync::Arc::new(WaypointPrefetchProgress::new()),
            waypoint_status: Vec::new(),
        }
    }

    /// Start a prefetch operation.
    pub fn start(&mut self, total: u32, cancel: tokio_util::sync::CancellationToken) {
        self.running = true;
        self.status = Some("Prefetching...".to_string());
        self.completed = 0;
        self.total = total;
        self.cancel = Some(cancel);
        self.waypoint_progress.init(total as usize);
        self.waypoint_status = vec![WaypointPrefetchStatus::NotStarted; total as usize];
    }

    /// Update prefetch progress.
    pub fn update_progress(&mut self, completed: u32, status: Option<String>) {
        self.completed = completed;
        if let Some(s) = status {
            self.status = Some(s);
        }
    }

    /// Complete the prefetch operation.
    pub fn complete(&mut self) {
        self.running = false;
        self.status = Some("Prefetch complete".to_string());
        self.cancel = None;
    }

    /// Fail the prefetch operation.
    pub fn fail(&mut self, error: String) {
        self.running = false;
        self.status = Some(format!("Error: {}", error));
        self.cancel = None;
    }

    /// Cancel the prefetch operation.
    pub fn cancel(&mut self) {
        if let Some(token) = self.cancel.take() {
            token.cancel();
        }
        self.running = false;
        self.status = Some("Cancelled".to_string());
    }

    /// Update the waypoint status snapshot from the shared progress.
    pub fn sync_waypoint_status(&mut self) {
        self.waypoint_status = self.waypoint_progress.get_all();
    }

    /// Progress as a fraction (0.0–1.0).
    pub fn progress(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.completed as f32 / self.total as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waypoint_prefetch_status_emoji() {
        assert_eq!(WaypointPrefetchStatus::NotStarted.emoji(), "⏳");
        assert_eq!(WaypointPrefetchStatus::InProgress.emoji(), "🔄");
        assert_eq!(WaypointPrefetchStatus::Completed.emoji(), "✅");
        assert_eq!(WaypointPrefetchStatus::Failed.emoji(), "❌");
    }

    #[test]
    fn test_waypoint_prefetch_progress() {
        let progress = WaypointPrefetchProgress::new();
        progress.init(3);

        let statuses = progress.get_all();
        assert_eq!(statuses.len(), 3);
        assert!(
            statuses
                .iter()
                .all(|s| *s == WaypointPrefetchStatus::NotStarted)
        );

        progress.set(1, WaypointPrefetchStatus::InProgress);
        let statuses = progress.get_all();
        assert_eq!(statuses[1], WaypointPrefetchStatus::InProgress);

        progress.set(0, WaypointPrefetchStatus::Completed);
        progress.set(2, WaypointPrefetchStatus::Failed);
        let statuses = progress.get_all();
        assert_eq!(statuses[0], WaypointPrefetchStatus::Completed);
        assert_eq!(statuses[2], WaypointPrefetchStatus::Failed);
    }

    #[test]
    fn test_prefetch_state_creation() {
        let state = PrefetchState::new();
        assert!(!state.running);
        assert!(state.status.is_none());
        assert_eq!(state.completed, 0);
        assert_eq!(state.total, 0);
        assert!(state.cancel.is_none());
    }

    #[test]
    fn test_prefetch_start() {
        let mut state = PrefetchState::new();
        let token = tokio_util::sync::CancellationToken::new();

        state.start(10, token);
        assert!(state.running);
        assert_eq!(state.total, 10);
        assert_eq!(state.completed, 0);
        assert!(state.cancel.is_some());
        assert_eq!(state.waypoint_status.len(), 10);
    }

    #[test]
    fn test_prefetch_update_progress() {
        let mut state = PrefetchState::new();
        let token = tokio_util::sync::CancellationToken::new();
        state.start(10, token);

        state.update_progress(5, Some("5/10".to_string()));
        assert_eq!(state.completed, 5);
        assert_eq!(state.status.as_deref(), Some("5/10"));
    }

    #[test]
    fn test_prefetch_complete() {
        let mut state = PrefetchState::new();
        let token = tokio_util::sync::CancellationToken::new();
        state.start(10, token);

        state.complete();
        assert!(!state.running);
        assert_eq!(state.status.as_deref(), Some("Prefetch complete"));
        assert!(state.cancel.is_none());
    }

    #[test]
    fn test_prefetch_fail() {
        let mut state = PrefetchState::new();
        let token = tokio_util::sync::CancellationToken::new();
        state.start(10, token);

        state.fail("Network error".to_string());
        assert!(!state.running);
        assert_eq!(state.status.as_deref(), Some("Error: Network error"));
    }

    #[test]
    fn test_prefetch_cancel() {
        let mut state = PrefetchState::new();
        let token = tokio_util::sync::CancellationToken::new();
        state.start(10, token);

        state.cancel();
        assert!(!state.running);
        assert_eq!(state.status.as_deref(), Some("Cancelled"));
        assert!(state.cancel.is_none());
    }

    #[test]
    fn test_prefetch_progress() {
        let mut state = PrefetchState::new();
        assert_eq!(state.progress(), 0.0); // total = 0

        let token = tokio_util::sync::CancellationToken::new();
        state.start(10, token);
        assert_eq!(state.progress(), 0.0);

        state.update_progress(5, None);
        assert!((state.progress() - 0.5).abs() < f32::EPSILON);

        state.update_progress(10, None);
        assert!((state.progress() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sync_waypoint_status() {
        let mut state = PrefetchState::new();
        let token = tokio_util::sync::CancellationToken::new();
        state.start(3, token);

        // Simulate background task updating shared progress
        state
            .waypoint_progress
            .set(0, WaypointPrefetchStatus::Completed);
        state
            .waypoint_progress
            .set(1, WaypointPrefetchStatus::InProgress);

        state.sync_waypoint_status();
        assert_eq!(state.waypoint_status[0], WaypointPrefetchStatus::Completed);
        assert_eq!(state.waypoint_status[1], WaypointPrefetchStatus::InProgress);
        assert_eq!(state.waypoint_status[2], WaypointPrefetchStatus::NotStarted);
    }
}
