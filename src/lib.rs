// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

pub mod altitude_predictor;
pub mod app_context;
pub mod config;
pub mod downloader;
pub mod dynamic_zoom;

pub mod fuse;

pub mod pipeline;
pub mod scenery;
pub mod seasons;
pub mod stats;
pub mod tiles;
pub mod time_exclusion;
pub mod ui;
pub mod webui;
pub mod xplane;

use tokio::runtime::Builder;

/// Create a multi-threaded Tokio runtime for async operations.
/// This should be called once at application startup and shared across components.
pub fn create_runtime() -> tokio::runtime::Runtime {
    Builder::new_multi_thread()
        .enable_all()
        .thread_name("autoortho-worker")
        .build()
        .expect("Failed to create Tokio runtime")
}
