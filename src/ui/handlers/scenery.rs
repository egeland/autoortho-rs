// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use iced::Task;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::ui::AutoOrthoApp;
use crate::ui::Message;
use crate::ui::state;

pub fn handle_refresh_available_regions(app: &mut AutoOrthoApp) -> Task<Message> {
    app.state.scenery.refreshing = true;
    app.state.scenery.status = Some("Fetching available regions...".to_string());

    let data_dir = app.state.scenery.data_dir.clone();
    let download_dir = app.state.scenery.download_dir.clone();
    let (tx, rx) = oneshot::channel();
    let rt = app.runtime.clone();

    rt.spawn(async move {
        let result = crate::ui::fetch_regions_and_installed(&data_dir, &download_dir).await;
        let _ = tx.send(result);
    });

    Task::perform(
        async { rx.await.unwrap_or(Err("Channel closed".into())) },
        |result| match result {
            Ok((regions, _installed)) => Message::RegionsLoaded(regions),
            Err(e) => Message::RegionsLoadFailed(e),
        },
    )
}

pub fn handle_regions_loaded(app: &mut AutoOrthoApp, regions: Vec<state::SceneryRegionInfo>) {
    app.state.scenery.refreshing = false;
    app.state.scenery.available_regions = regions;
    app.state.scenery.status = Some(format!(
        "Found {} regions available for download",
        app.state.scenery.available_regions.len()
    ));
    let packs = crate::scenery::installer::list_installed_packs(std::path::Path::new(
        &app.state.scenery.data_dir,
    ));
    app.state.scenery.installed_packs = packs
        .into_iter()
        .map(|p| state::InstalledPackInfo {
            id: p.id,
            name: p.name,
            version: p.ver,
        })
        .collect();
}

pub fn handle_regions_load_failed(app: &mut AutoOrthoApp, err: String) {
    app.state.scenery.refreshing = false;
    app.state.scenery.status = Some(format!("Error: {}", err));
}

pub fn handle_download_region(app: &mut AutoOrthoApp, region_id: String) -> Task<Message> {
    let total_bytes = app
        .state
        .scenery
        .available_regions
        .iter()
        .find(|r| r.id == region_id)
        .map(|r| r.total_size_bytes)
        .unwrap_or(0);
    let files_total = app
        .state
        .scenery
        .available_regions
        .iter()
        .find(|r| r.id == region_id)
        .map(|r| r.package_count as u32)
        .unwrap_or(0);

    let cancel = tokio_util::sync::CancellationToken::new();
    let dl_state = state::DownloadState {
        cancel: cancel.clone(),
        bytes_downloaded: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        total_bytes,
        current_file: Arc::new(Mutex::new(String::new())),
        files_done: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        files_total,
        extract_files_done: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        extract_files_total: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        extracting: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        pack_current: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        pack_total: Arc::new(std::sync::atomic::AtomicU32::new(0)),
    };
    app.state
        .scenery
        .downloading
        .insert(region_id.clone(), dl_state.clone());
    app.state.scenery.status = Some(format!("Downloading {}...", region_id));

    let download_dir = app.state.scenery.download_dir.clone();
    let data_dir = app.state.scenery.data_dir.clone();
    let rid = region_id.clone();
    let progress_bytes = dl_state.bytes_downloaded.clone();
    let progress_file = dl_state.current_file.clone();
    let progress_files_done = dl_state.files_done.clone();
    let (tx, rx) = oneshot::channel();
    let rt = app.runtime.clone();

    rt.spawn(async move {
        let result = crate::ui::download_and_install_region(
            &rid,
            &download_dir,
            &data_dir,
            &cancel,
            &progress_bytes,
            &progress_file,
            &progress_files_done,
            &dl_state.extract_files_done,
            &dl_state.extract_files_total,
            &dl_state.extracting,
            &dl_state.pack_current,
            &dl_state.pack_total,
        )
        .await;
        let _ = tx.send((rid, result));
    });

    Task::perform(
        async {
            rx.await
                .unwrap_or(("unknown".into(), Err("Channel closed".into())))
        },
        |(rid, result)| match result {
            Ok(msg) => Message::DownloadComplete(rid, msg),
            Err(e) => Message::DownloadFailed(rid, e),
        },
    )
}

pub fn handle_cancel_download(app: &mut AutoOrthoApp, region_id: String) {
    if let Some(dl) = app.state.scenery.downloading.get(&region_id) {
        dl.cancel.cancel();
    }
    app.state.scenery.status = Some(format!(
        "Cancelling {}... (partial files kept for resume)",
        region_id
    ));
}

pub fn handle_clean_region_downloads(app: &mut AutoOrthoApp, region_id: String) {
    let download_dir = std::path::Path::new(&app.state.scenery.download_dir);
    match crate::scenery::installer::clean_downloads(download_dir, &region_id) {
        Ok(bytes) => {
            if let Some(r) = app
                .state
                .scenery
                .available_regions
                .iter_mut()
                .find(|r| r.id == region_id)
            {
                r.has_partial_download = false;
            }
            app.state.scenery.status = Some(format!(
                "Cleaned {:.1} MB of downloads for {}",
                bytes as f64 / 1_048_576.0,
                region_id
            ));
        }
        Err(e) => {
            app.state.scenery.status = Some(format!("Clean failed: {}", e));
        }
    }
}

pub fn handle_uninstall_region(app: &mut AutoOrthoApp, region_id: String) {
    let data_dir = std::path::Path::new(&app.state.scenery.data_dir);
    match crate::scenery::installer::uninstall_region(&region_id, data_dir) {
        Ok(()) => {
            app.state.scenery.status = Some(format!("Uninstalled {}", region_id));
            let packs = crate::scenery::installer::list_installed_packs(data_dir);
            app.state.scenery.installed_packs = packs
                .into_iter()
                .map(|p| state::InstalledPackInfo {
                    id: p.id,
                    name: p.name,
                    version: p.ver,
                })
                .collect();
        }
        Err(e) => {
            app.state.scenery.status = Some(format!("Uninstall failed: {}", e));
        }
    }
}

pub fn handle_download_complete(app: &mut AutoOrthoApp, region_id: String, msg: String) {
    app.state.scenery.downloading.remove(&region_id);
    app.state.scenery.status = Some(msg);
    let packs = crate::scenery::installer::list_installed_packs(std::path::Path::new(
        &app.state.scenery.data_dir,
    ));
    app.state.scenery.installed_packs = packs
        .into_iter()
        .map(|p| state::InstalledPackInfo {
            id: p.id,
            name: p.name,
            version: p.ver,
        })
        .collect();
}

pub fn handle_download_failed(app: &mut AutoOrthoApp, region_id: String, err: String) {
    app.state.scenery.downloading.remove(&region_id);
    if err.contains("Cancelled") {
        if let Some(r) = app
            .state
            .scenery
            .available_regions
            .iter_mut()
            .find(|r| r.id == region_id)
        {
            r.has_partial_download = true;
        }
        app.state.scenery.status = Some(format!(
            "{} cancelled. Click Resume to continue, or Clean for a fresh start.",
            region_id
        ));
    } else {
        app.state.scenery.status = Some(format!("Error downloading {}: {}", region_id, err));
    }
}
