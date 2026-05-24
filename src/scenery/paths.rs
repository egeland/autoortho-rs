// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! X-Plane scenery path derivations.
//!
//! Canonical home for all paths derived from `xplane_path` and `cache_dir`.
//! Moved from `config.rs` as part of architecture deepening (candidate #3).

use std::path::PathBuf;

/// X-Plane's Custom Scenery directory, derived from `xplane_path`.
pub fn custom_scenery_path(xplane_path: &str) -> PathBuf {
    PathBuf::from(xplane_path).join("Custom Scenery")
}

/// FUSE mount point, derived from `xplane_path`.
/// This is the scenery pack root directory that X-Plane accesses.
pub fn mount_dir(xplane_path: &str) -> PathBuf {
    custom_scenery_path(xplane_path).join("z_autoortho")
}

/// Scenery data directory for real files (DSF, metadata, etc.).
/// Lives outside the mount point to avoid filesystem recursion.
pub fn scenery_data_dir(cache_dir: &str) -> PathBuf {
    PathBuf::from(cache_dir).join("scenery").join("z_autoortho")
}

/// Scenery install directory (Custom Scenery), derived from `xplane_path`.
pub fn scenery_install_dir(xplane_path: &str) -> PathBuf {
    custom_scenery_path(xplane_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_scenery_path() {
        assert_eq!(
            custom_scenery_path("/Games/X-Plane 12"),
            PathBuf::from("/Games/X-Plane 12/Custom Scenery")
        );
    }

    #[test]
    fn test_mount_dir() {
        assert_eq!(
            mount_dir("/Games/X-Plane 12"),
            PathBuf::from("/Games/X-Plane 12/Custom Scenery/z_autoortho")
        );
    }

    #[test]
    fn test_scenery_data_dir() {
        assert_eq!(
            scenery_data_dir("/Users/frode/Library/Caches/autoortho"),
            PathBuf::from("/Users/frode/Library/Caches/autoortho/scenery/z_autoortho")
        );
    }

    #[test]
    fn test_scenery_install_dir() {
        assert_eq!(
            scenery_install_dir("/Games/X-Plane 12"),
            PathBuf::from("/Games/X-Plane 12/Custom Scenery")
        );
    }
}
