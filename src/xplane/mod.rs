// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

pub mod averagers;
pub mod codec;
pub mod dataref;
pub mod simbrief;
pub mod simbrief_adapter;
pub mod traits;
pub mod udp;
pub mod udp_loop;

pub use averagers::{FlightDataAverager, HeadingAverager};
pub use codec::{RrefCodec, XPlaneError};
pub use dataref::{DatarefTracker, FlightDataStore};
pub use simbrief_adapter::SimBriefAdapter;
pub use traits::{FlightDataTracker, FlightPlanSource};

use std::sync::Arc;

/// Wrapper around `Arc<dyn FlightDataTracker>` that implements `Debug`.
#[derive(Clone)]
pub struct Tracker(pub Arc<dyn FlightDataTracker>);

impl std::fmt::Debug for Tracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Tracker")
            .field(&"<dyn FlightDataTracker>")
            .finish()
    }
}

impl std::ops::Deref for Tracker {
    type Target = dyn FlightDataTracker;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xplane::dataref::FlightDataStore;

    #[test]
    fn test_tracker_debug_output() {
        let store = FlightDataStore::new();
        let tracker = Tracker(Arc::new(store));
        let debug = format!("{:?}", tracker);
        assert!(debug.contains("Tracker"));
        assert!(debug.contains("<dyn FlightDataTracker>"));
    }

    #[test]
    fn test_tracker_deref_access() {
        let store = FlightDataStore::new();
        let tracker = Tracker(Arc::new(store));
        // Deref to FlightDataTracker trait — call get_flight_data
        let data = tracker.get_flight_data();
        assert!(!data.data_valid);
    }
}
