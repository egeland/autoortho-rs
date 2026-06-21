use crate::xplane::dataref::DatarefTracker;
use crate::xplane::traits::FlightDataTracker;

/// Adapter that makes the existing `DatarefTracker` conform to the
/// `FlightDataTracker` trait required for the new X‑Plane trait seams.
///
/// Now that `DatarefTracker` implements `FlightDataTracker` directly,
/// this adapter is only needed for backward compatibility with code
/// that explicitly constructs a `TrackerAdapter`.
pub struct TrackerAdapter {
    inner: DatarefTracker,
}

impl Default for TrackerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackerAdapter {
    /// Construct a new adapter that wraps the default dataref tracker.
    pub fn new() -> Self {
        Self {
            inner: DatarefTracker::new(),
        }
    }
}

impl FlightDataTracker for TrackerAdapter {
    fn update_from_response(&self, values: &[(i32, f32)]) {
        self.inner.update_from_response(values)
    }

    fn get_flight_data(&self) -> crate::xplane::dataref::FlightData {
        self.inner.get_flight_data()
    }

    fn mark_disconnected(&self) {
        self.inner.mark_disconnected()
    }

    fn clear_averages(&self) {
        self.inner.clear_averages()
    }
}
