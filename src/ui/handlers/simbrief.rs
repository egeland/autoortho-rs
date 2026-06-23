// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use iced::Task;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::tiles::provider;
use crate::ui::AutoOrthoApp;
use crate::ui::Message;
use crate::webui::custommap::CustomMapStore;
use crate::xplane::simbrief::{FlightFix, FlightPlan};

pub fn handle_fetch_simbrief(app: &mut AutoOrthoApp) -> Task<Message> {
    app.state.simbrief.fetching = true;
    app.state.simbrief.error = None;
    let user_id = app.state.config.flight.simbrief_user_id.clone();
    Task::perform(
        async move { crate::xplane::simbrief::fetch_flight_plan(&user_id).await },
        |result| match result {
            Ok(plan) => {
                let alt = plan.cruise_altitude_ft;
                let summary = format!(
                    "{} \u{2192} {} (FL{:.0})",
                    plan.origin,
                    plan.destination,
                    alt / 100.0
                );
                let fixes: Vec<(String, String, f32)> = plan
                    .fixes
                    .iter()
                    .map(|f| {
                        let alt = if f.ident == plan.origin {
                            plan.origin_elevation_ft
                        } else if f.ident == plan.destination {
                            plan.destination_elevation_ft
                        } else {
                            f.altitude_ft
                        };
                        (f.ident.clone(), f.fix_type.clone(), alt)
                    })
                    .collect();
                Message::SimbriefLoaded(summary, fixes, Arc::new(Mutex::new(Some(plan))))
            }
            Err(e) => Message::SimbriefFailed(e.to_string()),
        },
    )
}

pub fn handle_simbrief_loaded(
    app: &mut AutoOrthoApp,
    summary: String,
    fixes: Vec<(String, String, f32)>,
    plan: Arc<Mutex<Option<FlightPlan>>>,
) -> Task<Message> {
    app.state.simbrief.fetching = false;
    app.state.simbrief.route_summary = Some(summary.clone());
    app.state.simbrief.fixes = fixes;
    app.state.simbrief.show_details = false;
    app.state.simbrief.error = None;

    let flight_plan_opt = {
        let guard = plan.lock();
        guard.clone()
    };

    let (origin_lat, origin_lon, dest_lat, dest_lon, origin_code, dest_code): (
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        String,
        String,
    ) = if let Some(ref fp) = flight_plan_opt {
        let ofix: Option<&FlightFix> = FlightPlan::origin_fix(fp);
        let dfix: Option<&FlightFix> = FlightPlan::destination_fix(fp);
        (
            ofix.map(|f| f.lat),
            ofix.map(|f| f.lon),
            dfix.map(|f| f.lat),
            dfix.map(|f| f.lon),
            fp.origin.clone(),
            fp.destination.clone(),
        )
    } else {
        (None, None, None, None, String::new(), String::new())
    };

    app.state.simbrief.flight_plan = flight_plan_opt;

    let provider = app.state.config.tile.provider.clone();
    let zoom = app.state.config.flight.near_airport_zoom;

    let custom_map_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("autoortho")
        .join("custom_map.json");
    let custom_map = CustomMapStore::load(custom_map_path);
    let custom_cells = custom_map.get_cells();

    if let (Some(olat), Some(olon), Some(dlat), Some(dlon)) =
        (origin_lat, origin_lon, dest_lat, dest_lon)
    {
        return Task::perform(
            async move {
                let mut warnings = Vec::new();

                let origin_cell = format!("{},{}", olat.floor() as i32, olon.floor() as i32);
                let dest_cell = format!("{},{}", dlat.floor() as i32, dlon.floor() as i32);

                let origin_has_custom = custom_cells.contains_key(&origin_cell);
                if !origin_has_custom
                    && provider::test_provider_coverage(&provider, olat, olon, zoom)
                        .await
                        .is_err()
                {
                    warnings.push(format!("origin ({})", origin_code));
                }

                let dest_has_custom = custom_cells.contains_key(&dest_cell);
                if !dest_has_custom
                    && provider::test_provider_coverage(&provider, dlat, dlon, zoom)
                        .await
                        .is_err()
                {
                    warnings.push(format!("destination ({})", dest_code));
                }

                if warnings.is_empty() {
                    None
                } else {
                    Some(format!(
                        "Provider {} may not have coverage for your route: {}. Consider using ArcGIS for global coverage.",
                        provider,
                        warnings.join(" and ")
                    ))
                }
            },
            Message::SimbriefCoverageChecked,
        );
    }

    Task::none()
}

pub fn handle_simbrief_coverage_checked(app: &mut AutoOrthoApp, warning: Option<String>) {
    app.state.simbrief.coverage_warning = warning;
}

pub fn handle_simbrief_failed(app: &mut AutoOrthoApp, err: String) {
    app.state.simbrief.fetching = false;
    app.state.simbrief.error = Some(err);
}

pub fn handle_toggle_simbrief_details(app: &mut AutoOrthoApp) {
    app.state.simbrief.show_details = !app.state.simbrief.show_details;
}

/// Check provider coverage for a tile provider change (when flight plan is active).
pub fn handle_set_tile_provider(app: &mut AutoOrthoApp, provider_name: String) -> Task<Message> {
    // Extract flight plan data before mutating state
    let (olat, olon, dlat, dlon, origin_code, dest_code): (
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        String,
        String,
    ) = if let Some(ref fp) = app.state.simbrief.flight_plan {
        let ofix = FlightPlan::origin_fix(fp);
        let dfix = FlightPlan::destination_fix(fp);
        (
            ofix.map(|f| f.lat),
            ofix.map(|f| f.lon),
            dfix.map(|f| f.lat),
            dfix.map(|f| f.lon),
            fp.origin.clone(),
            fp.destination.clone(),
        )
    } else {
        (None, None, None, None, String::new(), String::new())
    };

    let zoom = app.state.config.flight.near_airport_zoom;

    let custom_map_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("autoortho")
        .join("custom_map.json");
    let custom_map = CustomMapStore::load(custom_map_path);
    let custom_cells = custom_map.get_cells();

    // Now mutate state (set_tile_provider also clears simbrief_coverage_warning)
    crate::ui::handlers::set_tile_provider(&mut app.state, provider_name.clone());

    if let (Some(olat), Some(olon), Some(dlat), Some(dlon)) = (olat, olon, dlat, dlon) {
        return Task::perform(
            async move {
                let mut warnings = Vec::new();

                let origin_cell = format!("{},{}", olat.floor() as i32, olon.floor() as i32);
                let dest_cell = format!("{},{}", dlat.floor() as i32, dlon.floor() as i32);

                let origin_has_custom = custom_cells.contains_key(&origin_cell);
                if !origin_has_custom
                    && provider::test_provider_coverage(&provider_name, olat, olon, zoom)
                        .await
                        .is_err()
                {
                    warnings.push(format!("origin ({})", origin_code));
                }

                let dest_has_custom = custom_cells.contains_key(&dest_cell);
                if !dest_has_custom
                    && provider::test_provider_coverage(&provider_name, dlat, dlon, zoom)
                        .await
                        .is_err()
                {
                    warnings.push(format!("destination ({})", dest_code));
                }

                if warnings.is_empty() {
                    None
                } else {
                    Some(format!(
                        "Provider {} may not have coverage for your route: {}. Consider using ArcGIS for global coverage.",
                        provider_name,
                        warnings.join(" and ")
                    ))
                }
            },
            Message::SimbriefCoverageChecked,
        );
    }

    Task::none()
}
