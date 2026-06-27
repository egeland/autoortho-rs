// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Scenery pack discovery, download, and management.
//!
//! AutoOrtho scenery packs are distributed via GitHub releases from
//! the autoortho-scenery repository. Each pack contains pre-built DSF
//! files that define tile coverage — AutoOrtho generates the actual
//! DDS textures on-the-fly via FUSE.

pub mod discovery;
pub mod download;
pub mod extract;
pub mod installer;
pub mod orchestrator;
pub mod packs_ini;
pub mod paths;
pub mod simheaven;
