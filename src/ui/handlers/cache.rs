// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use crate::ui::AutoOrthoApp;

pub fn handle_clear_dds_cache(app: &mut AutoOrthoApp) {
    let cache_dir = std::path::PathBuf::from(&app.state.config.cache_dir).join("dds");
    if cache_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&cache_dir) {
            log::warn!("Failed to clear DDS cache: {}", e);
            app.state.error_message = Some(format!("Failed to clear cache: {}", e));
        } else {
            log::info!("DDS cache cleared");
            app.state.dds_cache_size_bytes = 0;
            if let Some(ref cache) = app.state.dds_cache {
                let mut cache = cache.lock();
                let _ = cache.clear();
            }
        }
    }
}

pub fn handle_save_configuration(app: &mut AutoOrthoApp) {
    app.state.save_config();

    let xplane_dir = std::path::Path::new(&app.state.config.xplane_path);
    let active_regions: Vec<String> = app
        .state
        .scenery
        .installed_packs
        .iter()
        .filter_map(|p| {
            if p.id.starts_with("z_") || !p.id.starts_with("y_") {
                Some(p.id.clone())
            } else {
                None
            }
        })
        .collect();

    if !active_regions.is_empty() {
        match crate::scenery::simheaven::apply_simheaven_compat(
            xplane_dir,
            app.state.config.simheaven_compat,
            &active_regions,
        ) {
            Ok(_) => {}
            Err(e) => {
                log::warn!("SimHeaven compatibility apply failed: {}", e);
            }
        }
    }
}
