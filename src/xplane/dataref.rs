//! Dataref tracker: subscribes to X-Plane datarefs via UDP and maintains
//! a thread-safe snapshot of current flight data.

use crate::xplane::{FlightDataAverager, HeadingAverager, RrefCodec, XPlaneError};
use tracing::{debug, error, info, warn};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;

/// The datarefs we subscribe to from X-Plane, with their assigned indices.
pub mod datarefs {
    pub const LATITUDE: (i32, &str) = (0, "sim/flightmodel/position/latitude");
    pub const LONGITUDE: (i32, &str) = (1, "sim/flightmodel/position/longitude");
    pub const ALT_AGL: (i32, &str) = (2, "sim/flightmodel/position/y_agl");
    pub const HEADING: (i32, &str) = (3, "sim/flightmodel/position/mag_psi");
    pub const GROUND_SPEED: (i32, &str) = (4, "sim/flightmodel/position/groundspeed");
    pub const LOCAL_TIME: (i32, &str) = (5, "sim/time/local_time_sec");
    pub const PRESSURE_ALT: (i32, &str) = (6, "sim/flightmodel2/position/pressure_altitude");
    pub const SUN_PITCH: (i32, &str) = (7, "sim/graphics/scenery/sun_pitch_degrees");

    pub const ALL: &[(i32, &str)] = &[
        LATITUDE,
        LONGITUDE,
        ALT_AGL,
        HEADING,
        GROUND_SPEED,
        LOCAL_TIME,
        PRESSURE_ALT,
        SUN_PITCH,
    ];
}

const METERS_TO_FEET: f32 = 3.28084;
const DEFAULT_FREQ_HZ: i32 = 1;
const UDP_TIMEOUT: Duration = Duration::from_secs(5);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
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

/// Thread-safe dataref tracker that maintains current and averaged flight data.
#[derive(Debug)]
pub struct DatarefTracker {
    data: Arc<RwLock<FlightData>>,
    lat_avg: Arc<RwLock<FlightDataAverager>>,
    lon_avg: Arc<RwLock<FlightDataAverager>>,
    alt_avg: Arc<RwLock<FlightDataAverager>>,
    hdg_avg: Arc<RwLock<HeadingAverager>>,
    spd_avg: Arc<RwLock<FlightDataAverager>>,
}

impl DatarefTracker {
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

    /// Update flight data from a decoded RREF response.
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

impl Default for DatarefTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::traits::FlightDataTracker for DatarefTracker {
    fn update_from_response(&self, values: &[(i32, f32)]) {
        DatarefTracker::update_from_response(self, values)
    }

    fn get_flight_data(&self) -> FlightData {
        DatarefTracker::get_flight_data(self)
    }

    fn mark_disconnected(&self) {
        DatarefTracker::mark_disconnected(self)
    }

    fn clear_averages(&self) {
        DatarefTracker::clear_averages(self)
    }
}

/// Run the dataref tracker loop. Connects to X-Plane, subscribes to datarefs,
/// and continuously updates the tracker with received values.
///
/// This function runs until the `shutdown` token is cancelled.
pub async fn run_tracker(
    tracker: Arc<dyn super::FlightDataTracker>,
    xplane_addr: SocketAddr,
    shutdown: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        if *shutdown.borrow() {
            info!("Dataref tracker shutting down");
            return;
        }

        info!("Connecting to X-Plane at {}", xplane_addr);

        match connect_and_track(tracker.as_ref(), xplane_addr, &shutdown).await {
            Ok(()) => {
                info!("Dataref tracker disconnected cleanly");
                return;
            }
            Err(e) => {
                warn!(
                    "X-Plane connection lost: {}. Reconnecting in {:?}...",
                    e, RECONNECT_DELAY
                );
                tracker.mark_disconnected();
                tracker.clear_averages();
                tokio::time::sleep(RECONNECT_DELAY).await;
            }
        }
    }
}

async fn connect_and_track(
    tracker: &dyn super::FlightDataTracker,
    addr: SocketAddr,
    shutdown: &tokio::sync::watch::Receiver<bool>,
) -> Result<(), XPlaneError> {
    let local: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let socket = UdpSocket::bind(local).await.map_err(|e| {
        error!("Failed to bind UDP socket: {}", e);
        XPlaneError::PacketError
    })?;

    // Get the local address we bound to for debugging
    let local_addr = socket.local_addr().unwrap_or(local);
    info!(
        "UDP socket bound to {}, connecting to X-Plane at {}",
        local_addr, addr
    );
    socket.connect(addr).await.map_err(|e| {
        error!("Failed to connect UDP socket to {}: {}", addr, e);
        XPlaneError::PacketError
    })?;

    // Subscribe to all datarefs
    for &(index, dataref) in datarefs::ALL {
        let packet = RrefCodec::encode_request(DEFAULT_FREQ_HZ, index, dataref);
        socket.send(&packet).await.map_err(|e| {
            error!("Failed to send subscription for dataref [{}]: {}", index, e);
            XPlaneError::PacketError
        })?;
        debug!("Subscribed to dataref [{}] {}", index, dataref);
    }

    info!(
        "Subscribed to {} datarefs at {} Hz (X-Plane at {})",
        datarefs::ALL.len(),
        DEFAULT_FREQ_HZ,
        addr
    );

    let mut buf = [0u8; 4096];

    loop {
        if *shutdown.borrow() {
            // Unsubscribe (send freq=0)
            for &(index, dataref) in datarefs::ALL {
                let packet = RrefCodec::encode_request(0, index, dataref);
                let _ = socket.send(&packet).await;
            }
            return Ok(());
        }

        let recv_result = tokio::time::timeout(UDP_TIMEOUT, socket.recv(&mut buf)).await;

        match recv_result {
            Ok(Ok(n)) => match RrefCodec::decode_response(&buf[..n]) {
                Ok(values) => {
                    tracker.update_from_response(&values);
                }
                Err(e) => {
                    debug!("Failed to decode RREF response: {}", e);
                }
            },
            Ok(Err(e)) => {
                warn!("UDP socket error: {}", e);
                return Err(XPlaneError::PacketError);
            }
            Err(_) => {
                // Timeout — X-Plane may not be running or not sending data
                warn!(
                    "UDP timeout ({:?}) waiting for X-Plane response at {}. 
                    This usually means:
                    1. X-Plane is not running a flight (RREF only works during active flight)
                    2. Firewall is blocking UDP port {}
                    3. X-Plane is on a different machine (check xplane_host config)",
                    UDP_TIMEOUT,
                    addr,
                    addr.port()
                );
                tracker.mark_disconnected();
            }
        }
    }
}

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
    fn test_tracker_update_from_response() {
        let tracker = DatarefTracker::new();

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
    fn test_tracker_averages_require_min_samples() {
        let tracker = DatarefTracker::new();

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
        let tracker = DatarefTracker::new();

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
    fn test_datarefs_all_has_8_entries() {
        assert_eq!(datarefs::ALL.len(), 8);
    }

    #[test]
    fn test_tracker_handles_invalid_altitude() {
        let tracker = DatarefTracker::new();

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
