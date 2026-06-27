//! Route-based prefetch engine for SimBrief flight plans.
//!
//! Fetches flight plans and prefetches tiles along the route ahead of the aircraft.

use crate::config::ConfigSnapshot;
use crate::dynamic_zoom::DynamicZoom;
use crate::tiles::fetcher::TileFetcher;
use crate::tiles::prefetch::{RoutePrefetchConfig, SpatialPrefetcher};
use crate::xplane::FlightDataTracker;
use crate::xplane::simbrief::fetch_flight_plan;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

const PREFETCH_SPACING_NM: f64 = 10.0;
const PREFETCH_MAX_LOOKAHEAD_NM: f32 = 99999.0;
const PREFETCH_POLL_INTERVAL_SECS: u64 = 30;

/// Route-based prefetch engine.
///
/// Fetches SimBrief flight plan and prefetches tiles along the route.
pub struct RoutePrefetchEngine {
    simbrief_user_id: String,
    config: Arc<parking_lot::RwLock<crate::config::AutoOrthoConfig>>,
    fetcher: Arc<TileFetcher>,
    tracker: Arc<dyn FlightDataTracker>,
}

impl RoutePrefetchEngine {
    pub fn new(
        simbrief_user_id: String,
        config: Arc<parking_lot::RwLock<crate::config::AutoOrthoConfig>>,
        fetcher: Arc<TileFetcher>,
        tracker: Arc<dyn FlightDataTracker>,
    ) -> Self {
        Self {
            simbrief_user_id,
            config,
            fetcher,
            tracker,
        }
    }

    /// Run the prefetch loop until shutdown.
    pub async fn run(
        &self,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Fetching SimBrief flight plan for route prefetching...");
        let plan = fetch_flight_plan(&self.simbrief_user_id).await?;

        info!(
            "SimBrief route loaded: {} -> {}",
            plan.origin, plan.destination
        );

        let config_snapshot: ConfigSnapshot = (&*self.config.read()).into();

        let (
            prefetch_route_percent,
            route_prefetch_radius_nm,
            airport_radius_nm,
            prefetch_airports,
            max_zoom,
            zoom_rules,
            tile_provider,
            enable_dynamic_zoom,
            use_simbrief_altitude,
            route_consideration_radius_nm,
        ) = (
            config_snapshot.flight.prefetch_route_percent,
            config_snapshot.flight.route_prefetch_radius_nm,
            config_snapshot.flight.airport_radius_nm,
            config_snapshot.flight.prefetch_airports,
            config_snapshot.tile.max_zoom,
            config_snapshot.tile.zoom_rules.clone(),
            config_snapshot.tile.provider.clone(),
            config_snapshot.tile.enable_dynamic_zoom,
            config_snapshot.flight.use_simbrief_altitude,
            config_snapshot.flight.route_consideration_radius_nm as f64,
        );

        let mut prefetcher = SpatialPrefetcher::new();
        let route_config = RoutePrefetchConfig {
            percent_ahead: prefetch_route_percent,
            waypoint_radius_nm: route_prefetch_radius_nm as f64,
            airport_radius_nm: airport_radius_nm as f64,
            include_airports: prefetch_airports,
            zoom: max_zoom,
        };

        let dynamic_zoom_for_prefetch = DynamicZoom::new(zoom_rules, &tile_provider);

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Route prefetch shutting down");
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(PREFETCH_POLL_INTERVAL_SECS)) => {
                }
            };

            let flight_data = self.tracker.get_flight_data();

            if flight_data.data_valid {
                let lat = flight_data.lat;
                let lon = flight_data.lon;

                if plan.is_on_route(lat, lon, route_consideration_radius_nm) {
                    let points = plan.get_prefetch_points(
                        lat,
                        lon,
                        PREFETCH_SPACING_NM,
                        PREFETCH_MAX_LOOKAHEAD_NM,
                    );

                    if !points.is_empty() {
                        let route_distance_nm = points
                            .last()
                            .map(|p| p.distance_along_route_nm)
                            .unwrap_or(0.0);

                        prefetcher.prefetch_route(&points, route_distance_nm, route_config);

                        while let Some((row, col)) = prefetcher.next_tile() {
                            let zoom = if enable_dynamic_zoom {
                                if use_simbrief_altitude && !points.is_empty() {
                                    let mut closest_dist = f64::MAX;
                                    let mut best_alt_agl = 0.0f32;
                                    for point in &points {
                                        let dist = ((point.lat - lat).powi(2)
                                            + (point.lon - lon).powi(2))
                                        .sqrt();
                                        if dist < closest_dist {
                                            closest_dist = dist;
                                            best_alt_agl = point.altitude_agl_ft();
                                        }
                                    }
                                    dynamic_zoom_for_prefetch.zoom_for_altitude_agl(best_alt_agl)
                                } else {
                                    let alt_agl_ft = flight_data.alt_agl_m * 3.28084;
                                    dynamic_zoom_for_prefetch.zoom_for_altitude_agl(alt_agl_ft)
                                }
                            } else {
                                max_zoom
                            };

                            if let Err(e) = self
                                .fetcher
                                .get_chunk_data(row, col, &tile_provider, zoom)
                                .await
                            {
                                warn!(
                                    "Prefetch failed for tile ({}, {}) at zoom {}: {}",
                                    row, col, zoom, e
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_prefetch_engine_creation() {
        use crate::config::AutoOrthoConfig;
        use crate::test_utils::MockProvider;
        use crate::tiles::fetcher::TileFetcher;
        use crate::xplane::dataref::FlightDataStore;

        let config = Arc::new(parking_lot::RwLock::new(AutoOrthoConfig::default()));
        let provider = Arc::new(MockProvider);
        let fetcher = Arc::new(TileFetcher::new(provider, "ARC"));
        let tracker: Arc<dyn FlightDataTracker> = Arc::new(FlightDataStore::new());

        let engine = RoutePrefetchEngine::new("test_user".to_string(), config, fetcher, tracker);

        assert_eq!(engine.simbrief_user_id, "test_user");
    }
}
