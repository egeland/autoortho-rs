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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xplane::simbrief::FlightFix;

    #[test]
    fn test_adapter_new() {
        let adapter = SimBriefAdapter::new("test_user");
        assert_eq!(adapter.user_id, "test_user");
    }

    #[test]
    fn test_adapter_new_into_string() {
        let adapter = SimBriefAdapter::new(String::from("owned_string"));
        assert_eq!(adapter.user_id, "owned_string");
    }

    #[test]
    fn test_parse_ofp_returns_empty_plan() {
        let adapter = SimBriefAdapter::new("test");
        let plan = adapter.parse_ofp("<xml/></arg_value>").unwrap();
        assert!(plan.origin.is_empty());
        assert!(plan.destination.is_empty());
        assert_eq!(plan.origin_elevation_ft, 0.0);
        assert_eq!(plan.destination_elevation_ft, 0.0);
        assert_eq!(plan.cruise_altitude_ft, 0.0);
        assert!(plan.fixes.is_empty());
    }

    #[test]
    fn test_get_prefetch_points_empty_plan() {
        let adapter = SimBriefAdapter::new("test");
        let plan = FlightPlan {
            origin: "KLAX".into(),
            destination: "KJFK".into(),
            origin_elevation_ft: 128.0,
            destination_elevation_ft: 13.0,
            cruise_altitude_ft: 35000.0,
            fixes: vec![],
        };
        let points = adapter.get_prefetch_points(&plan);
        // Empty fixes = no prefetch points
        assert!(points.is_empty());
    }

    #[test]
    fn test_get_prefetch_points_with_fixes() {
        let adapter = SimBriefAdapter::new("test");
        let plan = FlightPlan {
            origin: "KLAX".into(),
            destination: "KSFO".into(),
            origin_elevation_ft: 128.0,
            destination_elevation_ft: 13.0,
            cruise_altitude_ft: 35000.0,
            fixes: vec![
                FlightFix {
                    ident: "KLAX".into(),
                    name: "Los Angeles Intl".into(),
                    fix_type: "apt".into(),
                    lat: 33.94,
                    lon: -118.41,
                    altitude_ft: 128.0,
                    ground_height_ft: 128.0,
                    time_total_sec: 0.0,
                    time_leg_sec: 0.0,
                    ground_speed_kt: 0.0,
                },
                FlightFix {
                    ident: "KSFO".into(),
                    name: "San Francisco Intl".into(),
                    fix_type: "apt".into(),
                    lat: 37.62,
                    lon: -122.38,
                    altitude_ft: 13.0,
                    ground_height_ft: 13.0,
                    time_total_sec: 3600.0,
                    time_leg_sec: 3600.0,
                    ground_speed_kt: 250.0,
                },
            ],
        };
        let points = adapter.get_prefetch_points(&plan);
        // Should generate some prefetch points along the route
        assert!(!points.is_empty());
    }
}
