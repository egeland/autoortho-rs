// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use iced::Task;
use tokio::sync::{oneshot, watch};

use crate::ui::AutoOrthoApp;
use crate::ui::Message;
use crate::ui::state::ServiceStatus;

pub fn handle_start_services(app: &mut AutoOrthoApp) -> Task<Message> {
    app.state.services.web_server = ServiceStatus::Starting;
    app.state.services.xplane_tracker = ServiceStatus::Starting;
    app.state.clear_error();

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    app.shutdown_tx = Some(shutdown_tx);

    let xplane_host = app.state.config.network.xplane_host.clone();
    let xplane_port = app.state.config.network.xplane_port;
    let config = app.state.config.clone();

    let (result_tx, result_rx) = oneshot::channel();
    let rt = app.runtime.clone();

    rt.spawn(async move {
        let result = crate::ui::start_all_services(
            crate::webui::WEB_UI_PORT,
            &xplane_host,
            xplane_port,
            config,
            shutdown_rx,
        )
        .await;
        let _ = result_tx.send(result);
    });

    Task::perform(
        async {
            result_rx
                .await
                .unwrap_or(Err("Runtime channel closed".into()))
        },
        |result| match result {
            Ok((url, tracker, tile_progress, dds_cache)) => {
                Message::ServicesStarted(url, Some(tracker), tile_progress, dds_cache)
            }
            Err(e) => Message::ServicesFailed(e),
        },
    )
}

pub fn handle_stop_services(app: &mut AutoOrthoApp) {
    if let Some(tx) = app.shutdown_tx.take() {
        let _ = tx.send(true);
    }
    app.state.services.web_server = ServiceStatus::Stopped;
    app.state.services.web_server_url = None;
    app.state.services.xplane_tracker = ServiceStatus::Stopped;
}

pub fn handle_services_started(
    app: &mut AutoOrthoApp,
    url: String,
    tracker: Option<std::sync::Arc<crate::xplane::dataref::DatarefTracker>>,
    tile_progress: std::sync::Arc<crate::ui::state::TileProgress>,
    dds_cache: Option<std::sync::Arc<parking_lot::Mutex<crate::pipeline::cache::DdsCache>>>,
) {
    app.state.services.web_server = ServiceStatus::Running;
    app.state.services.web_server_url = Some(url);
    app.state.services.xplane_tracker = ServiceStatus::Running;
    app.state.tracker = tracker;
    app.state.tile_progress = tile_progress;
    app.state.dds_cache = dds_cache;
}

pub fn handle_services_failed(app: &mut AutoOrthoApp, err: String) {
    app.state.services.web_server = ServiceStatus::Error;
    app.state.services.xplane_tracker = ServiceStatus::Error;
    app.state.set_error(format!("Failed to start: {}", err));
}
