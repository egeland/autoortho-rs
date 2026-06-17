// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Service status state for backend services.
//!
//! This module encapsulates the UI state for tracking the status of
//! backend services (web server, X-Plane tracker).

/// Runtime service status
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ServiceStatus {
    #[default]
    Stopped,
    Starting,
    Running,
    Error,
}

impl ServiceStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Stopped => "Stopped",
            Self::Starting => "Starting...",
            Self::Running => "Running",
            Self::Error => "Error",
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

/// Backend service status state.
#[derive(Debug, Clone)]
pub struct ServiceState {
    pub web_server: ServiceStatus,
    pub web_server_url: Option<String>,
    pub xplane_tracker: ServiceStatus,
}

impl Default for ServiceState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceState {
    pub fn new() -> Self {
        Self {
            web_server: ServiceStatus::Stopped,
            web_server_url: None,
            xplane_tracker: ServiceStatus::Stopped,
        }
    }

    /// Set web server status.
    pub fn set_web_server(&mut self, status: ServiceStatus) {
        self.web_server = status;
    }

    /// Set web server URL.
    pub fn set_web_server_url(&mut self, url: Option<String>) {
        self.web_server_url = url;
    }

    /// Set X-Plane tracker status.
    pub fn set_xplane_tracker(&mut self, status: ServiceStatus) {
        self.xplane_tracker = status;
    }

    /// Whether any backend service is running.
    pub fn any_running(&self) -> bool {
        self.web_server.is_running() || self.xplane_tracker.is_running()
    }

    /// Get web server status label.
    pub fn web_server_label(&self) -> &'static str {
        self.web_server.label()
    }

    /// Get X-Plane tracker status label.
    pub fn xplane_tracker_label(&self) -> &'static str {
        self.xplane_tracker.label()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_state_creation() {
        let state = ServiceState::new();
        assert_eq!(state.web_server, ServiceStatus::Stopped);
        assert_eq!(state.xplane_tracker, ServiceStatus::Stopped);
        assert!(!state.any_running());
    }

    #[test]
    fn test_set_web_server() {
        let mut state = ServiceState::new();
        state.set_web_server(ServiceStatus::Running);
        assert_eq!(state.web_server, ServiceStatus::Running);
        assert!(state.any_running());
    }

    #[test]
    fn test_set_web_server_url() {
        let mut state = ServiceState::new();
        state.set_web_server_url(Some("http://127.0.0.1:8080".to_string()));
        assert_eq!(
            state.web_server_url.as_deref(),
            Some("http://127.0.0.1:8080")
        );
    }

    #[test]
    fn test_set_xplane_tracker() {
        let mut state = ServiceState::new();
        state.set_xplane_tracker(ServiceStatus::Running);
        assert_eq!(state.xplane_tracker, ServiceStatus::Running);
        assert!(state.any_running());
    }

    #[test]
    fn test_any_running() {
        let mut state = ServiceState::new();
        assert!(!state.any_running());

        state.set_web_server(ServiceStatus::Running);
        assert!(state.any_running());

        state.set_web_server(ServiceStatus::Stopped);
        assert!(!state.any_running());

        state.set_xplane_tracker(ServiceStatus::Starting);
        assert!(!state.any_running()); // Starting is not running

        state.set_xplane_tracker(ServiceStatus::Running);
        assert!(state.any_running());
    }

    #[test]
    fn test_service_status_labels() {
        assert_eq!(ServiceStatus::Stopped.label(), "Stopped");
        assert_eq!(ServiceStatus::Starting.label(), "Starting...");
        assert_eq!(ServiceStatus::Running.label(), "Running");
        assert_eq!(ServiceStatus::Error.label(), "Error");
    }

    #[test]
    fn test_service_status_is_running() {
        assert!(!ServiceStatus::Stopped.is_running());
        assert!(!ServiceStatus::Starting.is_running());
        assert!(ServiceStatus::Running.is_running());
        assert!(!ServiceStatus::Error.is_running());
    }

    #[test]
    fn test_labels_delegates() {
        let mut state = ServiceState::new();
        assert_eq!(state.web_server_label(), "Stopped");

        state.set_web_server(ServiceStatus::Running);
        assert_eq!(state.web_server_label(), "Running");

        state.set_xplane_tracker(ServiceStatus::Error);
        assert_eq!(state.xplane_tracker_label(), "Error");
    }
}
