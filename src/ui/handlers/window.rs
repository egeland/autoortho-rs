// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use iced::Task;

use crate::ui::AutoOrthoApp;
use crate::ui::Message;
use crate::ui::SAVED_WINDOW_GEOM;

pub fn handle_window_opened(app: &mut AutoOrthoApp, id: iced::window::Id) -> Task<Message> {
    if let Some(geom) = SAVED_WINDOW_GEOM.lock().take() {
        let has_pos = geom.0.is_some() && geom.1.is_some();
        let has_size = geom.2.is_some() && geom.3.is_some();

        let mut tasks: Vec<Task<Message>> = Vec::new();

        if has_pos {
            let (x, y) = (geom.0.unwrap(), geom.1.unwrap());
            log::debug!("Restoring window position: ({}, {})", x, y);
            tasks.push(iced::window::move_to::<Message>(id, iced::Point::new(x, y)));
        }

        if has_size {
            let (w, h) = (geom.2.unwrap(), geom.3.unwrap());
            tasks.push(Task::perform(
                async {
                    futures_lite::future::yield_now().await;
                    futures_lite::future::yield_now().await;
                    futures_lite::future::yield_now().await;
                },
                move |_| Message::WindowRestoreSize(id, w, h),
            ));
        }

        if !tasks.is_empty() {
            return Task::batch(tasks);
        }
    }
    app.window_events_locked_until = None;
    Task::none()
}

pub fn handle_window_restore_size(
    _app: &mut AutoOrthoApp,
    id: iced::window::Id,
    w: f32,
    h: f32,
) -> Task<Message> {
    log::debug!("Restoring window size: ({}, {})", w, h);
    iced::window::resize::<Message>(id, iced::Size::new(w, h))
}

pub fn handle_window_moved(app: &mut AutoOrthoApp, pos: iced::Point) -> Task<Message> {
    if app
        .window_events_locked_until
        .is_some_and(|t| std::time::Instant::now() < t)
    {
        return Task::none();
    }
    app.window_events_locked_until = None;
    app.state.config.ui.window_x = Some(pos.x);
    app.state.config.ui.window_y = Some(pos.y);
    let _ = app.state.config.save();
    Task::none()
}

pub fn handle_window_resized(app: &mut AutoOrthoApp, size: iced::Size) -> Task<Message> {
    if app
        .window_events_locked_until
        .is_some_and(|t| std::time::Instant::now() < t)
    {
        return Task::none();
    }
    app.window_events_locked_until = None;
    let scale = app.state.config.ui.ui_scale as f32;
    app.state.config.ui.window_width = Some(size.width / scale);
    app.state.config.ui.window_height = Some(size.height / scale);
    let _ = app.state.config.save();
    Task::none()
}

pub fn handle_window_close_requested(app: &mut AutoOrthoApp) {
    log::info!(
        "Window close: saving config with x={:?} y={:?} w={:?} h={:?}",
        app.state.config.ui.window_x,
        app.state.config.ui.window_y,
        app.state.config.ui.window_width,
        app.state.config.ui.window_height
    );
    let _ = app.state.config.save();
    if let Some(tx) = app.shutdown_tx.take() {
        let _ = tx.send(true);
    }
}
