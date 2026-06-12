// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Web UI server using Axum.
//!
//! Provides a browser-based dashboard for monitoring AutoOrtho:
//! - `/` — Landing page
//! - `/map` — Live flight tracking map (Leaflet.js)
//! - `/stats` — Performance metrics
//! - `/metrics` — JSON stats API
//! - `/api/position` — Current aircraft position JSON
//! - `/ws` — WebSocket for live position updates
//! - `/api/custommap/*` — Custom map cell editor API

pub mod custommap;
pub mod routes;

pub const WEB_UI_PORT: u16 = 5847;

use crate::config::AutoOrthoConfig;
use crate::stats::StatsStore;
use crate::xplane::dataref::DatarefTracker;
use custommap::CustomMapStore;
use log::{error, info};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Shared application state accessible by all route handlers.
pub struct WebState {
    pub stats: Arc<StatsStore>,
    pub tracker: Arc<DatarefTracker>,
    pub custom_map: Arc<CustomMapStore>,
    pub config: Arc<parking_lot::RwLock<AutoOrthoConfig>>,
    pub position_tx: broadcast::Sender<PositionUpdate>,
}

impl WebState {
    pub fn new(
        stats: Arc<StatsStore>,
        tracker: Arc<DatarefTracker>,
        custom_map: Arc<CustomMapStore>,
        config: Arc<parking_lot::RwLock<AutoOrthoConfig>>,
    ) -> Self {
        let (position_tx, _) = broadcast::channel(32);
        Self {
            stats,
            tracker,
            custom_map,
            config,
            position_tx,
        }
    }
}

/// Position update sent via WebSocket
#[derive(Clone, Serialize)]
pub struct PositionUpdate {
    pub lat: f64,
    pub lon: f64,
    pub alt_agl_ft: f32,
    pub heading: f32,
    pub ground_speed_mps: f32,
    pub connected: bool,
}

/// Start the web server on the given port.
///
/// Returns the actual bound address (useful when port=0 for OS-assigned).
/// The server shuts down gracefully when `shutdown_rx` receives `true`.
pub async fn start_server_with_shutdown(
    port: u16,
    stats: Arc<StatsStore>,
    tracker: Arc<DatarefTracker>,
    config: Arc<parking_lot::RwLock<AutoOrthoConfig>>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
    let custom_map_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("autoortho")
        .join("custom_map.json");
    let custom_map = CustomMapStore::load(custom_map_path);

    let state = Arc::new(WebState::new(stats, tracker, custom_map, config));
    let app = routes::create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;

    info!("Web UI server listening on http://{}", bound_addr);

    let shutdown_signal = async move {
        let _ = shutdown_rx.wait_for(|v| *v).await;
        info!("Web server received shutdown signal");
    };

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
        {
            error!("Web server error: {}", e);
        }
        info!("Web server shut down");
    });

    Ok(bound_addr)
}

/// Start the web server on the given port (no shutdown support).
///
/// Returns the actual bound address (useful when port=0 for OS-assigned).
pub async fn start_server(
    port: u16,
    stats: Arc<StatsStore>,
    tracker: Arc<DatarefTracker>,
    config: Arc<parking_lot::RwLock<AutoOrthoConfig>>,
) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
    let (_tx, rx) = tokio::sync::watch::channel(false);
    start_server_with_shutdown(port, stats, tracker, config, rx).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_starts_on_random_port() {
        let stats = Arc::new(StatsStore::new());
        let tracker = Arc::new(DatarefTracker::new());
        let config = Arc::new(parking_lot::RwLock::new(
            crate::config::AutoOrthoConfig::default(),
        ));
        // Uses default custom map store (temp path won't persist)

        let addr = start_server(0, stats, tracker, config).await.unwrap();
        assert_ne!(addr.port(), 0); // Should have been assigned a real port
    }

    #[tokio::test]
    async fn test_server_restart_after_shutdown() {
        use tokio::sync::watch;

        let stats = Arc::new(StatsStore::new());
        let tracker = Arc::new(DatarefTracker::new());
        let config = Arc::new(parking_lot::RwLock::new(
            crate::config::AutoOrthoConfig::default(),
        ));

        // Start server on a random port
        let (tx1, rx1) = watch::channel(false);
        let addr =
            start_server_with_shutdown(0, stats.clone(), tracker.clone(), config.clone(), rx1)
                .await
                .unwrap();
        let port = addr.port();

        // Shut down the server
        let _ = tx1.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Start a new server on the same port — should succeed
        let (_tx2, rx2) = watch::channel(false);
        let addr2 = start_server_with_shutdown(port, stats, tracker, config, rx2)
            .await
            .unwrap();
        assert_eq!(addr2.port(), port);
    }

    #[test]
    fn test_web_ui_port_constant() {
        assert_eq!(WEB_UI_PORT, 5847);
    }
}
