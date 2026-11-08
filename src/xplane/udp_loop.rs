//! UDP communication loop for X-Plane dataref tracking.
//!
//! Handles subscribing to X-Plane datarefs via UDP, receiving updates,
//! and reconnection on connection loss.

use crate::xplane::traits::FlightDataTracker;
use crate::xplane::{RrefCodec, XPlaneError};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{debug, error, info, warn};

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

const DEFAULT_FREQ_HZ: i32 = 1;
const UDP_TIMEOUT: Duration = Duration::from_secs(5);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

/// Run the dataref tracker loop. Connects to X-Plane, subscribes to datarefs,
/// and continuously updates the tracker with received values.
///
/// This function runs until the `shutdown` token is cancelled.
pub async fn run_tracker(
    tracker: Arc<dyn FlightDataTracker>,
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

/// Spawn the tracker loop wired to a broadcast shutdown channel.
///
/// Headless mount mode uses `broadcast` for shutdown (web server, healer,
/// Ctrl+C), while `run_tracker` expects a `watch` channel; this bridges the
/// two so the mount path gets live X-Plane telemetry too.
pub fn spawn_tracker(
    tracker: Arc<dyn FlightDataTracker>,
    xplane_addr: SocketAddr,
    shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    let (watch_tx, watch_rx) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        let mut rx = shutdown_rx;
        loop {
            match rx.recv().await {
                Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    let _ = watch_tx.send(true);
                    break;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });
    tokio::spawn(run_tracker(tracker, xplane_addr, watch_rx))
}

async fn connect_and_track(
    tracker: &dyn FlightDataTracker,
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
    use crate::xplane::dataref::FlightData;
    use std::sync::Mutex;

    #[derive(Default)]
    struct StubTracker {
        updates: Mutex<Vec<(i32, f32)>>,
    }

    impl FlightDataTracker for StubTracker {
        fn update_from_response(&self, values: &[(i32, f32)]) {
            self.updates.lock().unwrap().extend_from_slice(values);
        }
        fn get_flight_data(&self) -> FlightData {
            FlightData::builder().build()
        }
        fn mark_disconnected(&self) {}
        fn clear_averages(&self) {}
    }

    /// Headless mount mode only has a broadcast shutdown channel, while
    /// run_tracker wants a watch channel. spawn_tracker must bridge the two
    /// and feed received RREF data into the tracker — otherwise the mount
    /// path has no live flight data (night exclusion, position-aware
    /// prefetch and dynamic zoom stay dead).
    #[tokio::test]
    async fn test_spawn_tracker_feeds_updates_and_shuts_down_on_broadcast() {
        // Fake X-Plane: stream a valid RREF response (index 0 = 42.5) so the
        // tracker's recv loop wakes promptly and shutdown is observed fast.
        let server = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server.local_addr().unwrap();
        let server_task = tokio::spawn(async move {
            let mut buf = [0u8; 512];
            let (_, peer) = server.recv_from(&mut buf).await.unwrap();
            let mut response = b"RREF".to_vec();
            response.extend_from_slice(&0i32.to_le_bytes());
            response.extend_from_slice(&42.5f32.to_le_bytes());
            loop {
                server.send_to(&response, peer).await.unwrap();
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        });

        let store = Arc::new(StubTracker::default());
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

        let handle = spawn_tracker(store.clone(), server_addr, shutdown_rx);

        // Tracker must receive the streamed RREF data
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let got = !store.updates.lock().unwrap().is_empty();
            if got || tokio::time::Instant::now() > deadline {
                assert!(got, "tracker never received RREF data");
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(*store.updates.lock().unwrap(), vec![(0, 42.5)]);

        // Broadcast shutdown must terminate the loop
        let _ = shutdown_tx.send(());
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("tracker task did not shut down on broadcast")
            .unwrap();
        server_task.abort();
    }

    #[test]
    fn test_datarefs_all_has_8_entries() {
        assert_eq!(datarefs::ALL.len(), 8);
    }
}
