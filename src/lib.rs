// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

pub mod altitude_predictor;
pub mod app_context;
pub mod config;
pub mod dynamic_zoom;

#[cfg(test)]
pub mod test_utils;

#[cfg(feature = "fuse")]
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

/// Library version, kept in sync with Cargo.toml.
/// Used by webui and CLI for version display.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
