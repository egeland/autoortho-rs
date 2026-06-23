// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Tile generation pipeline — re-exported from `tiles::tile_generator`.
//!
//! The implementation lives in `tiles::tile_generator`. This module
//! re-exports it for backward compatibility with code that imports
//! `crate::fuse::tile_generator`.

pub use crate::tiles::tile_generator::*;
