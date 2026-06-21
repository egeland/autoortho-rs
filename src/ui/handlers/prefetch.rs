// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use iced::Task;
use tokio::sync::oneshot;

use crate::ui::AutoOrthoApp;
use crate::ui::Message;

pub fn handle_prefetch_route(app: &mut AutoOrthoApp) -> Task<Message> {
    let Some(flight_plan) = app.state.simbrief_flight_plan.clone() else {
        app.state.prefetch.status = Some("No flight plan loaded".to_string());
        return Task::none();
    };

    let num_fixes = app.state.simbrief_fixes.len();
    app.state.prefetch.waypoint_progress.init(num_fixes);
    app.state.prefetch.waypoint_status =
        vec![crate::ui::state::WaypointPrefetchStatus::NotStarted; num_fixes];

    app.state.prefetch.running = true;
    app.state.prefetch.status = None;
    app.state.prefetch.completed = 0;
    app.state.prefetch.total = 0;

    let cancel = tokio_util::sync::CancellationToken::new();
    app.state.prefetch.cancel = Some(cancel.clone());

    let config = app.state.config.clone();
    let waypoint_progress = app.state.prefetch.waypoint_progress.clone();
    let (tx, rx) = oneshot::channel();
    let rt = app.runtime.clone();

    rt.spawn(async move {
        let result =
            crate::ui::prefetch_route_impl(&flight_plan, &config, &cancel, waypoint_progress).await;
        let _ = tx.send(result);
    });

    Task::perform(
        async { rx.await.unwrap_or(Err("Channel closed".into())) },
        |result| match result {
            Ok(msg) if msg.contains("cache 90") => Message::PrefetchCompleteCacheFull(msg),
            Ok(msg) => Message::PrefetchComplete(msg),
            Err(e) => Message::PrefetchFailed(e),
        },
    )
}

pub fn handle_stop_prefetch(app: &mut AutoOrthoApp) {
    if let Some(cancel) = app.state.prefetch.cancel.take() {
        cancel.cancel();
    }
    app.state.prefetch.running = false;
    app.state.prefetch.status = Some("Prefetch cancelled".to_string());
}

pub fn handle_prefetch_progress(app: &mut AutoOrthoApp, completed: u32, total: u32) {
    app.state.prefetch.completed = completed;
    app.state.prefetch.total = total;
}

pub fn handle_prefetch_complete(app: &mut AutoOrthoApp, msg: String) {
    app.state.prefetch.running = false;
    app.state.prefetch.cancel = None;
    app.state.prefetch.status = Some(msg);
}

pub fn handle_prefetch_complete_cache_full(app: &mut AutoOrthoApp, msg: String) {
    app.state.prefetch.running = false;
    app.state.prefetch.cancel = None;
    app.state.prefetch.status = Some(msg);
}

pub fn handle_prefetch_failed(app: &mut AutoOrthoApp, err: String) {
    app.state.prefetch.running = false;
    app.state.prefetch.cancel = None;
    app.state.prefetch.status = Some(format!("Error: {}", err));
}
