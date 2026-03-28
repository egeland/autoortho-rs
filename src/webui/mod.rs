// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

//! Web UI server using Axum.
//!
//! Provides a browser-based dashboard for monitoring AutoOrtho:
//! - `/` — Landing page
//! - `/map` — Live flight tracking map (Leaflet.js)
//! - `/stats` — Performance metrics
//! - `/metrics` — JSON stats API
//! - `/api/position` — Current aircraft position JSON
//! - `/api/custommap/*` — Custom map cell editor API

pub mod custommap;
pub mod routes;

use crate::config::AutoOrthoConfig;
use crate::stats::StatsStore;
use crate::xplane::dataref::DatarefTracker;
use custommap::CustomMapStore;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;

/// Shared application state accessible by all route handlers.
pub struct WebState {
    pub stats: Arc<StatsStore>,
    pub tracker: Arc<DatarefTracker>,
    pub custom_map: Arc<CustomMapStore>,
    pub config: Arc<parking_lot::RwLock<AutoOrthoConfig>>,
}

/// Start the web server on the given port.
///
/// Returns the actual bound address (useful when port=0 for OS-assigned).
pub async fn start_server(
    port: u16,
    stats: Arc<StatsStore>,
    tracker: Arc<DatarefTracker>,
    config: Arc<parking_lot::RwLock<AutoOrthoConfig>>,
) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
    let custom_map_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("autoortho")
        .join("custom_map.json");
    let custom_map = CustomMapStore::load(custom_map_path);

    let state = Arc::new(WebState {
        stats,
        tracker,
        custom_map,
        config,
    });
    let app = routes::create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;

    info!("Web UI server listening on http://{}", bound_addr);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("Web server error: {}", e);
        }
        info!("Web server shut down");
    });

    Ok(bound_addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_starts_on_random_port() {
        let stats = Arc::new(StatsStore::new());
        let tracker = Arc::new(DatarefTracker::new());
        let config = Arc::new(parking_lot::RwLock::new(crate::config::AutoOrthoConfig::default()));
        // Uses default custom map store (temp path won't persist)

        let addr = start_server(0, stats, tracker, config).await.unwrap();
        assert_ne!(addr.port(), 0); // Should have been assigned a real port
    }
}
