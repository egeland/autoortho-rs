// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

pub mod averagers;
pub mod codec;
pub mod dataref;
pub mod simbrief;
pub mod simbrief_adapter;
pub mod traits;
pub mod udp;

pub use averagers::{FlightDataAverager, HeadingAverager};
pub use codec::{RrefCodec, XPlaneError};
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
