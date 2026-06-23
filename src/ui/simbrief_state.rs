// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! SimBrief flight plan UI state.

use crate::xplane::simbrief::FlightPlan;

/// SimBrief flight plan state, grouped for locality.
#[derive(Debug, Clone, Default)]
pub struct SimBriefState {
    pub fetching: bool,
    pub route_summary: Option<String>,
    /// (ident, fix_type, altitude_ft) for display
    pub fixes: Vec<(String, String, f32)>,
    pub show_details: bool,
    pub error: Option<String>,
    pub flight_plan: Option<FlightPlan>,
    pub coverage_warning: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simbrief_state_defaults() {
        let state = SimBriefState::default();
        assert!(!state.fetching);
        assert!(state.route_summary.is_none());
        assert!(state.fixes.is_empty());
        assert!(!state.show_details);
        assert!(state.error.is_none());
        assert!(state.flight_plan.is_none());
        assert!(state.coverage_warning.is_none());
    }
}
