// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use iced::Task;

use crate::ui::AutoOrthoApp;
use crate::ui::Message;

pub fn handle_browse_xplane_path(app: &mut AutoOrthoApp) -> Task<Message> {
    crate::ui::browse_folder("xplane_path", &app.state.config.xplane_path)
}

pub fn handle_browse_cache_dir(app: &mut AutoOrthoApp) -> Task<Message> {
    crate::ui::browse_folder("cache_dir", &app.state.config.cache_dir)
}

pub fn handle_browse_scenery_download_dir(app: &mut AutoOrthoApp) -> Task<Message> {
    crate::ui::browse_folder("scenery_download_dir", &app.state.scenery.download_dir)
}

pub fn handle_folder_picked(app: &mut AutoOrthoApp, field: String, path: String) {
    match field.as_str() {
        "xplane_path" => {
            crate::ui::handlers::handle_set_xplane_path(&mut app.state, path);
        }
        "cache_dir" => {
            crate::ui::handlers::handle_set_cache_dir(&mut app.state, path);
        }
        "scenery_download_dir" => app.state.scenery.download_dir = path,
        _ => {}
    }
}
