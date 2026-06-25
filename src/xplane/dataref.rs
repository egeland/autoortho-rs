//! Flight data store: thread-safe snapshot of current flight data with averaging.
//!
//! Provides `FlightDataStore` — the central position source for all consumers
//! (prefetch, webui, ui). Handles averaging internally.

use crate::xplane::{FlightDataAverager, HeadingAverager};
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub use super::udp_loop::datarefs;

const METERS_TO_FEET: f32 = 3.28084;
const AVERAGER_WINDOW: usize = 60;

/// Atomic snapshot of current flight data.
#[derive(Debug, Clone)]
pub struct FlightData {
    /// Latitude in degrees (-90..90)
    pub lat: f64,
    /// Longitude in degrees (-180..180)
    pub lon: f64,
    /// Altitude AGL in meters
    pub alt_agl_m: f32,
    /// Magnetic heading (0-360)
    pub heading: f32,
    /// Ground speed in m/s
    pub ground_speed_mps: f32,
    /// Sim local time (seconds since midnight)
    pub local_time_sec: f32,
    /// Pressure altitude in feet
    pub pressure_alt_ft: f32,
    /// Sun elevation angle in degrees (-90 to 90, -999 if invalid)
    pub sun_pitch: f32,
    /// Whether we're connected and receiving data
    pub connected: bool,
    /// Whether position data has been validated at least once
    pub data_valid: bool,
    /// Timestamp of last update
    pub last_update: Option<Instant>,
}

impl Default for FlightData {
    fn default() -> Self {
        Self {
            lat: 0.0,
            lon: 0.0,
            alt_agl_m: 0.0,
            heading: 0.0,
            ground_speed_mps: 0.0,
            local_time_sec: 0.0,
            pressure_alt_ft: 0.0,
            sun_pitch: -999.0,
            connected: false,
            data_valid: false,
            last_update: None,
        }
    }
}

impl FlightData {
    /// Create a new FlightData builder.
    pub fn builder() -> FlightDataBuilder {
        FlightDataBuilder::default()
    }

    /// Altitude AGL in feet
    pub fn alt_agl_ft(&self) -> f32 {
        self.alt_agl_m * METERS_TO_FEET
    }

    /// Validate that position data is reasonable
    pub fn is_position_valid(&self) -> bool {
        // Convert validation bounds from feet to meters since alt_agl_m is stored in meters
        // Assuming X-Plane's y_agl dataref is in FEET with reasonable range of -500 to 50000 feet
        const ALT_AGL_MIN_FEET: f32 = -500.0;
        const ALT_AGL_MAX_FEET: f32 = 50000.0;
        const ALT_AGL_MIN_METERS: f32 = ALT_AGL_MIN_FEET / METERS_TO_FEET;
        const ALT_AGL_MAX_METERS: f32 = ALT_AGL_MAX_FEET / METERS_TO_FEET;

        (-90.0..=90.0).contains(&self.lat)
            && (-180.0..=180.0).contains(&self.lon)
            && self.alt_agl_m > ALT_AGL_MIN_METERS
            && self.alt_agl_m < ALT_AGL_MAX_METERS
    }
}

/// Builder for FlightData.
#[derive(Debug, Clone, Default)]
pub struct FlightDataBuilder {
    lat: f64,
    lon: f64,
    alt_agl_m: f32,
    heading: f32,
    ground_speed_mps: f32,
    local_time_sec: f32,
    pressure_alt_ft: f32,
    sun_pitch: f32,
}

impl FlightDataBuilder {
    pub fn lat(mut self, lat: f64) -> Self {
        self.lat = lat;
        self
    }

    pub fn lon(mut self, lon: f64) -> Self {
        self.lon = lon;
        self
    }

    pub fn alt_agl_m(mut self, alt_agl_m: f32) -> Self {
        self.alt_agl_m = alt_agl_m;
        self
    }

    pub fn heading(mut self, heading: f32) -> Self {
        self.heading = heading;
        self
    }

    pub fn ground_speed_mps(mut self, ground_speed_mps: f32) -> Self {
        self.ground_speed_mps = ground_speed_mps;
        self
    }

    pub fn local_time_sec(mut self, local_time_sec: f32) -> Self {
        self.local_time_sec = local_time_sec;
        self
    }

    pub fn pressure_alt_ft(mut self, pressure_alt_ft: f32) -> Self {
        self.pressure_alt_ft = pressure_alt_ft;
        self
    }

    pub fn sun_pitch(mut self, sun_pitch: f32) -> Self {
        self.sun_pitch = sun_pitch;
        self
    }

    pub fn build(self) -> FlightData {
        let mut data = FlightData::default();
        data.lat = self.lat;
        data.lon = self.lon;
        data.alt_agl_m = self.alt_agl_m;
        data.heading = self.heading;
        data.ground_speed_mps = self.ground_speed_mps;
        data.local_time_sec = self.local_time_sec;
        data.pressure_alt_ft = self.pressure_alt_ft;
        data.sun_pitch = self.sun_pitch;
        data.data_valid = data.is_position_valid();
        data.connected = true;
        data.last_update = Some(Instant::now());
        data
    }
}

/// Averaged flight data over a time window, suitable for stable predictions.
#[derive(Debug, Clone)]
pub struct FlightAverages {
    pub lat: f64,
    pub lon: f64,
    pub alt_ft: f32,
    pub heading: f32,
    pub ground_speed_mps: f32,
    pub vertical_speed_fpm: f32,
}

/// Thread-safe flight data store that maintains current and averaged flight data.
#[derive(Debug)]
pub struct FlightDataStore {
    data: Arc<RwLock<FlightData>>,
    lat_avg: Arc<RwLock<FlightDataAverager>>,
    lon_avg: Arc<RwLock<FlightDataAverager>>,
    alt_avg: Arc<RwLock<FlightDataAverager>>,
    hdg_avg: Arc<RwLock<HeadingAverager>>,
    spd_avg: Arc<RwLock<FlightDataAverager>>,
}

impl FlightDataStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(FlightData::default())),
            lat_avg: Arc::new(RwLock::new(FlightDataAverager::new(AVERAGER_WINDOW))),
            lon_avg: Arc::new(RwLock::new(FlightDataAverager::new(AVERAGER_WINDOW))),
            alt_avg: Arc::new(RwLock::new(FlightDataAverager::new(AVERAGER_WINDOW))),
            hdg_avg: Arc::new(RwLock::new(HeadingAverager::new(AVERAGER_WINDOW))),
            spd_avg: Arc::new(RwLock::new(FlightDataAverager::new(AVERAGER_WINDOW))),
        }
    }

    /// Get current flight data snapshot.
    pub fn get_flight_data(&self) -> FlightData {
        self.data.read().expect("flight data lock poisoned").clone()
    }

    /// Get averaged flight data for stable predictions.
    pub fn get_averages(&self) -> Option<FlightAverages> {
        let lat = self.lat_avg.read().expect("lat avg lock poisoned");
        let lon = self.lon_avg.read().expect("lon avg lock poisoned");
        let alt = self.alt_avg.read().expect("alt avg lock poisoned");
        let hdg = self.hdg_avg.read().expect("hdg avg lock poisoned");
        let spd = self.spd_avg.read().expect("spd avg lock poisoned");

        if lat.count() < 5 {
            return None;
        }

        Some(FlightAverages {
            lat: lat.average() as f64,
            lon: lon.average() as f64,
            alt_ft: alt.average(),
            heading: hdg.average(),
            ground_speed_mps: spd.average(),
            vertical_speed_fpm: 0.0, // TODO: compute from altitude delta
        })
    }

    /// Update flight data with named fields.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &self,
        lat: f64,
        lon: f64,
        alt_agl_m: f32,
        heading: f32,
        ground_speed_mps: f32,
        local_time_sec: f32,
        pressure_alt_ft: f32,
        sun_pitch: f32,
    ) {
        let data = FlightData::builder()
            .lat(lat)
            .lon(lon)
            .alt_agl_m(alt_agl_m)
            .heading(heading)
            .ground_speed_mps(ground_speed_mps)
            .local_time_sec(local_time_sec)
            .pressure_alt_ft(pressure_alt_ft)
            .sun_pitch(sun_pitch)
            .build();

        // Update averagers if data is valid
        if data.data_valid {
            self.lat_avg
                .write()
                .expect("lat avg lock poisoned")
                .add(data.lat as f32);
            self.lon_avg
                .write()
                .expect("lon avg lock poisoned")
                .add(data.lon as f32);
            self.alt_avg
                .write()
                .expect("alt avg lock poisoned")
                .add(data.alt_agl_ft());
            self.hdg_avg
                .write()
                .expect("hdg avg lock poisoned")
                .add(data.heading);
            self.spd_avg
                .write()
                .expect("spd avg lock poisoned")
                .add(data.ground_speed_mps);
        }

        *self.data.write().expect("flight data lock poisoned") = data;
    }

    /// Update flight data from a decoded RREF response (index-based).
    ///
    /// This method maintains backward compatibility with the FlightDataTracker trait.
    pub fn update_from_response(&self, values: &[(i32, f32)]) {
        let mut data = self.data.write().expect("flight data lock poisoned");

        for &(index, value) in values {
            match index {
                0 => data.lat = value as f64,
                1 => data.lon = value as f64,
                2 => {
                    // X-Plane's y_agl dataref returns height AGL in FEET
                    // Convert to meters for internal storage
                    data.alt_agl_m = value / METERS_TO_FEET;
                }
                3 => data.heading = value,
                4 => data.ground_speed_mps = value,
                5 => data.local_time_sec = value,
                6 => data.pressure_alt_ft = value,
                7 => data.sun_pitch = value,
                _ => {}
            }
        }

        data.connected = true;
        data.data_valid = data.is_position_valid();
        data.last_update = Some(Instant::now());

        // Update averagers
        if data.data_valid {
            self.lat_avg
                .write()
                .expect("lat avg lock poisoned")
                .add(data.lat as f32);
            self.lon_avg
                .write()
                .expect("lon avg lock poisoned")
                .add(data.lon as f32);
            self.alt_avg
                .write()
                .expect("alt avg lock poisoned")
                .add(data.alt_agl_ft());
            self.hdg_avg
                .write()
                .expect("hdg avg lock poisoned")
                .add(data.heading);
            self.spd_avg
                .write()
                .expect("spd avg lock poisoned")
                .add(data.ground_speed_mps);
        }
    }

    /// Clear averaged data (e.g., on reconnect).
    pub fn clear_averages(&self) {
        self.lat_avg.write().expect("lat avg lock poisoned").clear();
        self.lon_avg.write().expect("lon avg lock poisoned").clear();
        self.alt_avg.write().expect("alt avg lock poisoned").clear();
        self.hdg_avg.write().expect("hdg avg lock poisoned").clear();
        self.spd_avg.write().expect("spd avg lock poisoned").clear();
    }

    /// Mark as disconnected.
    pub fn mark_disconnected(&self) {
        let mut data = self.data.write().expect("flight data lock poisoned");
        data.connected = false;
    }
}

impl Default for FlightDataStore {
    fn default() -> Self {
        Self::new()
    }
}

impl super::traits::FlightDataTracker for FlightDataStore {
    fn update_from_response(&self, values: &[(i32, f32)]) {
        FlightDataStore::update_from_response(self, values)
    }

    fn get_flight_data(&self) -> FlightData {
        FlightDataStore::get_flight_data(self)
    }

    fn mark_disconnected(&self) {
        FlightDataStore::mark_disconnected(self)
    }

    fn clear_averages(&self) {
        FlightDataStore::clear_averages(self)
    }
}

/// Type alias for backward compatibility.
pub type DatarefTracker = FlightDataStore;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_data_default() {
        let data = FlightData::default();
        assert!(!data.connected);
        assert!(!data.data_valid);
        assert_eq!(data.sun_pitch, -999.0);
    }

    #[test]
    fn test_flight_data_alt_conversion() {
        let mut data = FlightData::default();
        data.alt_agl_m = 1000.0;
        assert!((data.alt_agl_ft() - 3280.84).abs() < 0.1);
    }

    #[test]
    fn test_position_validation() {
        let mut data = FlightData::default();
        data.lat = 37.7749;
        data.lon = -122.4194;
        data.alt_agl_m = 100.0;
        assert!(data.is_position_valid());

        data.lat = 100.0; // Invalid
        assert!(!data.is_position_valid());
    }

    #[test]
    fn test_flight_data_builder() {
        let data = FlightData::builder()
            .lat(37.7749)
            .lon(-122.4194)
            .alt_agl_m(100.0)
            .heading(270.0)
            .ground_speed_mps(50.0)
            .local_time_sec(43200.0)
            .pressure_alt_ft(3000.0)
            .sun_pitch(45.0)
            .build();

        assert!((data.lat - 37.7749).abs() < 0.001);
        assert!((data.lon - (-122.4194)).abs() < 0.001);
        assert!((data.alt_agl_m - 100.0).abs() < 0.1);
        assert!((data.heading - 270.0).abs() < 0.1);
        assert!(data.connected);
        assert!(data.data_valid);
        assert!(data.last_update.is_some());
    }

    #[test]
    fn test_tracker_update_from_response() {
        let tracker = FlightDataStore::new();

        let values = vec![
            (0, 37.7749f32),   // lat
            (1, -122.4194f32), // lon
            (2, 1000.0f32),    // alt_agl_m
            (3, 270.0f32),     // heading
            (4, 50.0f32),      // ground_speed
        ];

        tracker.update_from_response(&values);
        let data = tracker.get_flight_data();

        assert!(data.connected);
        assert!(data.data_valid);
        assert!((data.lat - 37.7749).abs() < 0.001);
        assert!((data.heading - 270.0).abs() < 0.1);
    }

    #[test]
    fn test_tracker_update_named_fields() {
        let tracker = FlightDataStore::new();

        tracker.update(
            37.7749,   // lat
            -122.4194, // lon
            304.8,     // alt_agl_m (1000 feet)
            270.0,     // heading
            50.0,      // ground_speed
            43200.0,   // local_time
            3000.0,    // pressure_alt
            45.0,      // sun_pitch
        );

        let data = tracker.get_flight_data();
        assert!(data.connected);
        assert!(data.data_valid);
        assert!((data.lat - 37.7749).abs() < 0.001);
        assert!((data.heading - 270.0).abs() < 0.1);
    }

    #[test]
    fn test_tracker_averages_require_min_samples() {
        let tracker = FlightDataStore::new();

        // Not enough samples yet
        let values = vec![(0, 37.0f32), (1, -122.0f32), (2, 100.0f32)];
        tracker.update_from_response(&values);
        assert!(tracker.get_averages().is_none());

        // Add more samples
        for _ in 0..10 {
            tracker.update_from_response(&values);
        }
        assert!(tracker.get_averages().is_some());
    }

    #[test]
    fn test_tracker_disconnect_clears_averages() {
        let tracker = FlightDataStore::new();

        let values = vec![(0, 37.0f32), (1, -122.0f32), (2, 100.0f32)];
        for _ in 0..10 {
            tracker.update_from_response(&values);
        }
        assert!(tracker.get_averages().is_some());

        tracker.mark_disconnected();
        tracker.clear_averages();
        assert!(tracker.get_averages().is_none());
    }

    #[test]
    fn test_tracker_handles_invalid_altitude() {
        let tracker = FlightDataStore::new();

        // Send valid latitude and longitude
        // Send invalid altitude (-999.0 feet, presumed X-Plane invalid indicator)
        let values = vec![
            (0, 37.7749f32),   // lat
            (1, -122.4194f32), // lon
            (2, -999.0f32),    // alt_agl_m (invalid altitude in feet)
            (3, 270.0f32),     // heading
            (4, 50.0f32),      // ground_speed
        ];

        tracker.update_from_response(&values);
        let data = tracker.get_flight_data();

        // We should be connected (we got a packet)
        assert!(data.connected);

        // But data should be invalid due to bad altitude
        assert!(!data.data_valid);

        // Latitude and longitude should be stored correctly
        assert!((data.lat - 37.7749).abs() < 0.001);
        assert!((data.lon - (-122.4194)).abs() < 0.001);

        // Heading and ground speed should be stored correctly
        assert!((data.heading - 270.0).abs() < 0.1);
        assert!((data.ground_speed_mps - 50.0).abs() < 0.1);
    }
}
