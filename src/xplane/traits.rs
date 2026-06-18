use crate::xplane::dataref::FlightData;
use crate::xplane::simbrief::{FlightPlan, PrefetchPoint, SimbriefError};

/// Abstract source of flight‑plan data.
pub trait FlightPlanSource {
    /// Load a flight plan from the underlying source (e.g. SimBrief API).
    fn fetch_flight_plan(&self) -> Result<FlightPlan, SimbriefError>;

    /// Parse a raw OFP XML document into a `FlightPlan`.
    fn parse_ofp(&self, xml: &str) -> Result<FlightPlan, SimbriefError>;

    /// Derive prefetch points from a flight plan.
    fn get_prefetch_points(&self, plan: &FlightPlan) -> Vec<PrefetchPoint>;
}

/// Abstract tracker for X‑Plane telemetry updates.
pub trait FlightDataTracker {
    /// Process a batch of telemetry updates (e.g. from UDP).
    fn update_from_response(&self, values: &[(i32, f32)]);

    /// Get aggregated flight data.
    fn get_flight_data(&self) -> FlightData;

    /// Mark the tracker as disconnected (e.g. lost telemetry).
    fn mark_disconnected(&self);
}
