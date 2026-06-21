// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use iced::Task;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::ui::AutoOrthoApp;
use crate::ui::Message;

pub fn handle_set_test_lat(app: &mut AutoOrthoApp, v: String) {
    if let Some((lat, rest)) = v.split_once('|') {
        if let Some((lon, zoom_str)) = rest.split_once('|') {
            app.state
                .dev_test
                .set_tile_coords(lat.to_string(), lon.to_string());
            if let Ok(z) = zoom_str.parse::<u32>() {
                app.state.dev_test.set_tile_zoom(z);
            }
        }
    } else {
        app.state
            .dev_test
            .set_tile_coords(v, app.state.dev_test.test_tile_lon.clone());
    }
}

pub fn handle_set_test_lon(app: &mut AutoOrthoApp, v: String) {
    app.state
        .dev_test
        .set_tile_coords(app.state.dev_test.test_tile_lat.clone(), v);
}

pub fn handle_set_test_zoom(app: &mut AutoOrthoApp, v: u32) {
    app.state.dev_test.set_tile_zoom(v);
}

pub fn handle_fetch_test_tile(app: &mut AutoOrthoApp) -> Task<Message> {
    app.state.dev_test.start_tile_test();

    let lat = app
        .state
        .dev_test
        .test_tile_lat
        .parse::<f64>()
        .unwrap_or(-33.86);
    let lon = app
        .state
        .dev_test
        .test_tile_lon
        .parse::<f64>()
        .unwrap_or(151.21);
    let zoom = app.state.dev_test.test_tile_zoom;
    let provider_name = app.state.config.tile.provider.clone();
    let rate_limit = app.state.config.network.rate_limit.requests_per_second;

    let (tx, rx) = oneshot::channel();
    let rt = app.runtime.clone();

    rt.spawn(async move {
        use crate::tiles::fetcher::TileFetcher;
        use crate::tiles::provider::ProviderFactory;

        let provider = match ProviderFactory::create(&provider_name) {
            Some(p) => p,
            None => {
                let _ = tx.send(Err(format!("Unknown provider: {}", provider_name)));
                return;
            }
        };

        let fetcher = Arc::new(TileFetcher::with_rate_limit(
            provider,
            &provider_name,
            rate_limit,
        ));

        let result = crate::ui::fetch_test_tile_impl(lat, lon, zoom, &provider_name, fetcher).await;
        let _ = tx.send(result);
    });

    Task::perform(
        async { rx.await.unwrap_or(Err("Channel closed".into())) },
        |result| match result {
            Ok((msg, img)) => Message::TestTileComplete(msg, img),
            Err(e) => Message::TestTileFailed(e),
        },
    )
}

pub fn handle_test_tile_complete(
    app: &mut AutoOrthoApp,
    msg: String,
    image_data: Option<(u32, u32, Vec<u8>)>,
) {
    app.state.dev_test.complete_tile_test(msg, image_data);
}

pub fn handle_test_tile_failed(app: &mut AutoOrthoApp, err: String) {
    app.state.dev_test.fail_tile_test(err);
}

pub fn handle_test_fallback_lookup(app: &mut AutoOrthoApp) -> Task<Message> {
    app.state.dev_test.start_fallback_test();
    let lat = app
        .state
        .dev_test
        .test_tile_lat
        .parse::<f64>()
        .unwrap_or(-33.86);
    let lon = app
        .state
        .dev_test
        .test_tile_lon
        .parse::<f64>()
        .unwrap_or(151.21);
    let zoom = app.state.dev_test.test_tile_zoom;
    let cache_dir = std::path::PathBuf::from(&app.state.config.cache_dir);
    let fallback_config = app.state.config.fallback.clone();

    let (tx, rx) = oneshot::channel();
    let rt = app.runtime.clone();

    rt.spawn(async move {
        let result =
            crate::ui::test_fallback_lookup(lat, lon, zoom, &cache_dir, fallback_config).await;
        let _ = tx.send(result);
    });

    Task::perform(
        async { rx.await.unwrap_or(None) },
        Message::FallbackTestComplete,
    )
}

pub fn handle_fallback_test_complete(
    app: &mut AutoOrthoApp,
    result: Option<crate::ui::dev_test_state::FallbackTestResult>,
) {
    if let Some(r) = result {
        app.state.dev_test.complete_fallback_test(r);
    } else {
        app.state
            .dev_test
            .fail_fallback_test("No result".to_string());
    }
}
