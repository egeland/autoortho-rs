use crate::xplane::simbrief::{FlightPlan, PrefetchPoint, SimbriefError};
use crate::xplane::traits::FlightPlanSource;
use tokio::runtime::Runtime;

/// Adapter that makes the SimBrief client comply with the
/// `FlightPlanSource` trait required for the new X‑Plane trait seams.
pub struct SimBriefAdapter {
    user_id: String,
}

impl SimBriefAdapter {
    /// Construct a new adapter for the given SimBrief user ID.
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
        }
    }
}

impl FlightPlanSource for SimBriefAdapter {
    /// Synchronously fetch a flight plan from SimBrief by blocking on the async
    /// `fetch_flight_plan` function. This indirection lets the rest of the code
    /// depend only on the trait, not on the async implementation detail.
    fn fetch_flight_plan(&self) -> Result<FlightPlan, SimbriefError> {
        let rt = Runtime::new().unwrap();
        rt.block_on(crate::xplane::simbrief::fetch_flight_plan(&self.user_id))
    }

    /// Parse a raw OFP XML document into a `FlightPlan`.
    /// This stub simply returns an empty `FlightPlan`; real parsing would be
    /// performed by delegating to the existing XML‑parsing logic in
    /// `simbrief.rs`.
    fn parse_ofp(&self, _xml: &str) -> Result<FlightPlan, SimbriefError> {
        Ok(FlightPlan {
            origin: String::new(),
            destination: String::new(),
            origin_elevation_ft: 0.0,
            destination_elevation_ft: 0.0,
            cruise_altitude_ft: 0.0,
            fixes: Vec::new(),
        })
    }

    /// Derive prefetch points from a flight plan.
    fn get_prefetch_points(&self, plan: &FlightPlan) -> Vec<PrefetchPoint> {
        plan.get_prefetch_points(33.94, -118.41, 10.0, 3600.0)
    }
}
