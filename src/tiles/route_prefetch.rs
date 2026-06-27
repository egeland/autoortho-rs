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
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Fetching SimBrief flight plan for route prefetching...");
        let plan = fetch_flight_plan(&self.simbrief_user_id).await?;

        info!(
            "SimBrief route loaded: {} -> {}",
            plan.origin, plan.destination
        );

        self.run_with_plan(plan, shutdown_rx).await
    }

    /// Run the prefetch loop with a pre-fetched flight plan.
    pub(crate) async fn run_with_plan(
        &self,
        plan: crate::xplane::simbrief::FlightPlan,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config_snapshot: ConfigSnapshot = (&*self.config.read()).into();

        let route_config = RoutePrefetchConfig {
            percent_ahead: config_snapshot.flight.prefetch_route_percent,
            waypoint_radius_nm: config_snapshot.flight.route_prefetch_radius_nm as f64,
            airport_radius_nm: config_snapshot.flight.airport_radius_nm as f64,
            include_airports: config_snapshot.flight.prefetch_airports,
            zoom: config_snapshot.tile.max_zoom,
        };

        let dynamic_zoom_for_prefetch = DynamicZoom::new(
            config_snapshot.tile.zoom_rules.clone(),
            &config_snapshot.tile.provider,
        );
        let mut prefetcher = SpatialPrefetcher::new();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Route prefetch shutting down");
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(PREFETCH_POLL_INTERVAL_SECS)) => {
                }
            };

            self.process_prefetch_iteration(
                &plan,
                &mut prefetcher,
                &dynamic_zoom_for_prefetch,
                &route_config,
                config_snapshot.tile.enable_dynamic_zoom,
                config_snapshot.flight.use_simbrief_altitude,
                config_snapshot.tile.max_zoom,
                &config_snapshot.tile.provider,
            )
            .await;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn process_prefetch_iteration(
        &self,
        plan: &crate::xplane::simbrief::FlightPlan,
        prefetcher: &mut SpatialPrefetcher,
        dynamic_zoom: &DynamicZoom,
        route_config: &RoutePrefetchConfig,
        enable_dynamic_zoom: bool,
        use_simbrief_altitude: bool,
        max_zoom: u32,
        tile_provider: &str,
    ) {
        let flight_data = self.tracker.get_flight_data();

        if !flight_data.data_valid {
            return;
        }

        let lat = flight_data.lat;
        let lon = flight_data.lon;

        let route_consideration_radius_nm = route_config.waypoint_radius_nm;

        if !plan.is_on_route(lat, lon, route_consideration_radius_nm) {
            return;
        }

        let points =
            plan.get_prefetch_points(lat, lon, PREFETCH_SPACING_NM, PREFETCH_MAX_LOOKAHEAD_NM);

        if points.is_empty() {
            return;
        }

        let route_distance_nm = points
            .last()
            .map(|p| p.distance_along_route_nm)
            .unwrap_or(0.0);

        prefetcher.prefetch_route(&points, route_distance_nm, *route_config);

        while let Some((row, col)) = prefetcher.next_tile() {
            let zoom = if enable_dynamic_zoom {
                if use_simbrief_altitude && !points.is_empty() {
                    let mut closest_dist = f64::MAX;
                    let mut best_alt_agl = 0.0f32;
                    for point in &points {
                        let dist = ((point.lat - lat).powi(2) + (point.lon - lon).powi(2)).sqrt();
                        if dist < closest_dist {
                            closest_dist = dist;
                            best_alt_agl = point.altitude_agl_ft();
                        }
                    }
                    dynamic_zoom.zoom_for_altitude_agl(best_alt_agl)
                } else {
                    let alt_agl_ft = flight_data.alt_agl_m * 3.28084;
                    dynamic_zoom.zoom_for_altitude_agl(alt_agl_ft)
                }
            } else {
                max_zoom
            };

            if let Err(e) = self
                .fetcher
                .get_chunk_data(row, col, tile_provider, zoom)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AutoOrthoConfig;
    use crate::test_utils::MockProvider;
    use crate::tiles::fetcher::TileFetcher;
    use crate::xplane::dataref::{FlightData, FlightDataStore};
    use crate::xplane::simbrief::{FlightFix, FlightPlan};

    fn make_engine() -> (
        RoutePrefetchEngine,
        Arc<parking_lot::RwLock<AutoOrthoConfig>>,
        Arc<FlightDataStore>,
    ) {
        let config = Arc::new(parking_lot::RwLock::new(AutoOrthoConfig::default()));
        // Use FailingProvider: tiles fail fast, no infinite caching loops
        let provider = Arc::new(crate::test_utils::FailingProvider);
        let fetcher = Arc::new(TileFetcher::new(provider, "ARC"));
        let store = Arc::new(FlightDataStore::new());
        let tracker: Arc<dyn FlightDataTracker> = store.clone();
        let engine =
            RoutePrefetchEngine::new("test_user".to_string(), config.clone(), fetcher, tracker);
        (engine, config, store)
    }

    fn sample_plan() -> FlightPlan {
        FlightPlan {
            origin: "KLAX".into(),
            destination: "KLAS".into(),
            origin_elevation_ft: 126.0,
            destination_elevation_ft: 2181.0,
            cruise_altitude_ft: 35000.0,
            fixes: vec![
                FlightFix {
                    ident: "KLAX".into(),
                    name: "Los Angeles".into(),
                    fix_type: "apt".into(),
                    lat: 33.94,
                    lon: -118.41,
                    altitude_ft: 126.0,
                    ground_height_ft: 126.0,
                    time_total_sec: 0.0,
                    time_leg_sec: 0.0,
                    ground_speed_kt: 0.0,
                },
                FlightFix {
                    ident: "KLAS".into(),
                    name: "Las Vegas".into(),
                    fix_type: "apt".into(),
                    lat: 36.08,
                    lon: -115.15,
                    altitude_ft: 2181.0,
                    ground_height_ft: 2181.0,
                    time_total_sec: 2400.0,
                    time_leg_sec: 2400.0,
                    ground_speed_kt: 300.0,
                },
            ],
        }
    }

    #[test]
    fn test_route_prefetch_engine_creation() {
        let (engine, _, _) = make_engine();
        assert_eq!(engine.simbrief_user_id, "test_user");
    }

    #[tokio::test]
    async fn test_run_with_plan_empty_fixes_exits() {
        let (engine, _, _) = make_engine();
        let plan = FlightPlan {
            origin: "".into(),
            destination: "".into(),
            origin_elevation_ft: 0.0,
            destination_elevation_ft: 0.0,
            cruise_altitude_ft: 0.0,
            fixes: vec![],
        };
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let _ = shutdown_tx.send(());
        let result = engine.run_with_plan(plan, shutdown_rx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_plan_valid_data_on_route() {
        let (engine, config, store) = make_engine();
        {
            let mut c = config.write();
            c.tile.enable_dynamic_zoom = true;
        }
        let plan = sample_plan();
        store.update(33.94, -118.41, 100.0, 270.0, 150.0, 0.0, 126.0, -10.0);
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let _ = shutdown_tx.send(());
        let result = engine.run_with_plan(plan, shutdown_rx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_plan_valid_data_off_route() {
        let (engine, _, store) = make_engine();
        let plan = sample_plan();
        store.update(45.0, -90.0, 500.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let _ = shutdown_tx.send(());
        let result = engine.run_with_plan(plan, shutdown_rx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_plan_data_not_valid() {
        let (engine, _, _store) = make_engine();
        let plan = sample_plan();
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let _ = shutdown_tx.send(());
        let result = engine.run_with_plan(plan, shutdown_rx).await;
        assert!(result.is_ok());
    }

    fn low_zoom_config_snapshot() -> ConfigSnapshot {
        let mut cfg = crate::config::AutoOrthoConfig::default();
        cfg.tile.max_zoom = 2;
        (&cfg).into()
    }

    fn make_test_route_config(snapshot: &ConfigSnapshot) -> RoutePrefetchConfig {
        RoutePrefetchConfig {
            percent_ahead: snapshot.flight.prefetch_route_percent,
            waypoint_radius_nm: snapshot.flight.route_prefetch_radius_nm as f64,
            airport_radius_nm: snapshot.flight.airport_radius_nm as f64,
            include_airports: snapshot.flight.prefetch_airports,
            zoom: snapshot.tile.max_zoom,
        }
    }

    #[tokio::test]
    async fn test_process_iteration_on_route_dynamic_zoom() {
        let (engine, _, store) = make_engine();
        let plan = sample_plan();
        store.update(33.94, -118.41, 100.0, 270.0, 150.0, 0.0, 126.0, -10.0);

        let snapshot = low_zoom_config_snapshot();
        let route_config = make_test_route_config(&snapshot);
        let dynamic_zoom =
            DynamicZoom::new(snapshot.tile.zoom_rules.clone(), &snapshot.tile.provider);
        let mut prefetcher = SpatialPrefetcher::new();

        engine
            .process_prefetch_iteration(
                &plan,
                &mut prefetcher,
                &dynamic_zoom,
                &route_config,
                true,
                false,
                2,
                &snapshot.tile.provider,
            )
            .await;
    }

    #[tokio::test]
    async fn test_process_iteration_on_route_no_dynamic_zoom() {
        let (engine, _, store) = make_engine();
        let plan = sample_plan();
        store.update(33.94, -118.41, 100.0, 270.0, 150.0, 0.0, 126.0, -10.0);

        let snapshot = low_zoom_config_snapshot();
        let route_config = make_test_route_config(&snapshot);
        let dynamic_zoom =
            DynamicZoom::new(snapshot.tile.zoom_rules.clone(), &snapshot.tile.provider);
        let mut prefetcher = SpatialPrefetcher::new();

        engine
            .process_prefetch_iteration(
                &plan,
                &mut prefetcher,
                &dynamic_zoom,
                &route_config,
                false,
                false,
                2,
                &snapshot.tile.provider,
            )
            .await;
    }

    #[tokio::test]
    async fn test_process_iteration_on_route_use_simbrief_alt() {
        let (engine, _, store) = make_engine();
        let plan = sample_plan();
        store.update(33.94, -118.41, 100.0, 270.0, 150.0, 0.0, 126.0, -10.0);

        let snapshot = low_zoom_config_snapshot();
        let mut cfg = crate::config::AutoOrthoConfig::default();
        cfg.tile.max_zoom = 2;
        cfg.flight.use_simbrief_altitude = true;
        let route_config = RoutePrefetchConfig {
            percent_ahead: cfg.flight.prefetch_route_percent,
            waypoint_radius_nm: cfg.flight.route_prefetch_radius_nm as f64,
            airport_radius_nm: cfg.flight.airport_radius_nm as f64,
            include_airports: cfg.flight.prefetch_airports,
            zoom: 2,
        };
        let dynamic_zoom = DynamicZoom::new(vec![], &"ARC");
        let mut prefetcher = SpatialPrefetcher::new();

        engine
            .process_prefetch_iteration(
                &plan,
                &mut prefetcher,
                &dynamic_zoom,
                &route_config,
                true,
                true,
                2,
                "ARC",
            )
            .await;
    }

    #[tokio::test]
    async fn test_process_iteration_off_route() {
        let (engine, _, store) = make_engine();
        let plan = sample_plan();
        store.update(45.0, -90.0, 500.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        let snapshot = low_zoom_config_snapshot();
        let route_config = make_test_route_config(&snapshot);
        let dynamic_zoom =
            DynamicZoom::new(snapshot.tile.zoom_rules.clone(), &snapshot.tile.provider);
        let mut prefetcher = SpatialPrefetcher::new();

        engine
            .process_prefetch_iteration(
                &plan,
                &mut prefetcher,
                &dynamic_zoom,
                &route_config,
                true,
                false,
                2,
                &snapshot.tile.provider,
            )
            .await;
    }

    #[tokio::test]
    async fn test_process_iteration_data_not_valid() {
        let (engine, _, _store) = make_engine();
        let plan = sample_plan();

        let snapshot = low_zoom_config_snapshot();
        let route_config = make_test_route_config(&snapshot);
        let dynamic_zoom =
            DynamicZoom::new(snapshot.tile.zoom_rules.clone(), &snapshot.tile.provider);
        let mut prefetcher = SpatialPrefetcher::new();

        engine
            .process_prefetch_iteration(
                &plan,
                &mut prefetcher,
                &dynamic_zoom,
                &route_config,
                true,
                false,
                2,
                &snapshot.tile.provider,
            )
            .await;
    }
}
