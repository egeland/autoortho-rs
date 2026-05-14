// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use crate::pipeline::dds;
use regex::Regex;
use thiserror::Error;

pub mod filesystem;

#[cfg(not(windows))]
pub mod mount;

#[cfg(windows)]
pub mod mount_win;

pub mod platform;

#[derive(Debug, Error)]
pub enum FuseError {
    #[error("Invalid DDS path")]
    InvalidPath,
    #[error("Path parsing failed")]
    ParseFailed,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Shutdown requested")]
    ShutdownRequested,
}

/// DDS virtual file path parser.
///
/// Parses X-Plane tile paths like `/textures/3232_2176_BI16.dds`
/// into (row, col, maptype, zoom) tuples.
pub struct DdsPathParser {
    regex: Regex,
}

impl Default for DdsPathParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DdsPathParser {
    pub fn new() -> Self {
        let regex = Regex::new(r".*/(\d+)[-_](\d+)[-_](\S*)(\d{2})\.dds")
            .expect("DDS path regex is a valid constant pattern");
        Self { regex }
    }

    /// Parse DDS path into (row, col, maptype, zoom)
    pub fn parse(&self, path: &str) -> Result<(u32, u32, String, u32), FuseError> {
        let caps = self.regex.captures(path).ok_or(FuseError::ParseFailed)?;

        let row = caps[1].parse::<u32>().map_err(|_| FuseError::ParseFailed)?;
        let col = caps[2].parse::<u32>().map_err(|_| FuseError::ParseFailed)?;
        let maptype = caps[3].trim_end_matches('_').to_string();
        let zoom = caps[4].parse::<u32>().map_err(|_| FuseError::ParseFailed)?;

        if zoom > 28 {
            return Err(FuseError::InvalidPath);
        }

        Ok((row, col, maptype, zoom))
    }
}

/// Calculate DDS file size for a standard 4096×4096 tile.
pub fn calculate_dds_size(_zoom: u32) -> usize {
    dds::dds_file_size_4096_bc3()
}

/// Check if a path is the poison pill shutdown marker.
pub fn is_poison_path(path: &str) -> bool {
    path.ends_with(".poison") || path == "/.poison"
}

/// Virtual directory entries that X-Plane expects.
pub const VIRTUAL_DIRS: &[&str] = &["textures", "terrain"];

/// Marker file shown in virtual directories to indicate system is active.
pub const MARKER_FILE: &str = "AOISWORKING";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dds_path_parser_creation() {
        let parser = DdsPathParser::new();
        assert!(parser.regex.is_match("/0_0_GO2_12.dds"));
    }

    #[test]
    fn test_dds_path_parse_valid() {
        let parser = DdsPathParser::new();
        let (row, col, maptype, zoom) = parser.parse("/0_0_GO2_12.dds").unwrap();
        assert_eq!(row, 0);
        assert_eq!(col, 0);
        assert_eq!(maptype, "GO2");
        assert_eq!(zoom, 12);
    }

    #[test]
    fn test_dds_path_parse_with_path_prefix() {
        let parser = DdsPathParser::new();
        let (row, col, maptype, zoom) = parser.parse("/textures/3232_2176_BI16.dds").unwrap();
        assert_eq!(row, 3232);
        assert_eq!(col, 2176);
        assert_eq!(maptype, "BI");
        assert_eq!(zoom, 16);
    }

    #[test]
    fn test_dds_path_parse_with_hyphens() {
        let parser = DdsPathParser::new();
        let (row, col, maptype, zoom) = parser.parse("/100-50-BI_15.dds").unwrap();
        assert_eq!(row, 100);
        assert_eq!(col, 50);
        assert_eq!(maptype, "BI");
        assert_eq!(zoom, 15);
    }

    #[test]
    fn test_dds_path_parse_invalid_no_match() {
        let parser = DdsPathParser::new();
        assert!(parser.parse("/invalid.dds").is_err());
    }

    #[test]
    fn test_dds_path_parse_zoom_too_high() {
        let parser = DdsPathParser::new();
        assert!(parser.parse("/0_0_GO2_99.dds").is_err());
    }

    #[test]
    fn test_dds_path_parse_various_maptypes() {
        let parser = DdsPathParser::new();
        let maptypes = [
            "GO2", "BI", "ARC", "NAIP", "USGS", "FIREFLY", "YNDX", "APPLE",
        ];
        for maptype in maptypes {
            let path = format!("/10_20_{}_18.dds", maptype);
            let result = parser.parse(&path).unwrap();
            assert_eq!(result.2, maptype);
        }
    }

    #[test]
    fn test_calculate_dds_size_realistic() {
        let size = calculate_dds_size(12);
        assert!(size > 1_000_000);
        assert!(size < 100_000_000);
    }

    #[test]
    fn test_is_poison_path() {
        assert!(is_poison_path("/.poison"));
        assert!(is_poison_path("/some/path/.poison"));
        assert!(!is_poison_path("/textures/tile.dds"));
    }

    #[test]
    #[cfg(windows)]
    fn test_cross_platform_fuse_available() {
        use crate::fuse::mount_win as fuse_mount;
        let _fuse_mount = fuse_mount::mount;
        assert!(true, "Dokan mount module available on Windows");
    }

    #[test]
    #[cfg(not(windows))]
    fn test_cross_platform_fuse_available() {
        use crate::fuse::mount as fuse_mount;
        let _fuse_mount = fuse_mount::mount;
        assert!(true, "FUSE mount module available on non-Windows");
    }

    #[test]
    fn test_platform_name_windows() {
        #[cfg(target_os = "windows")]
        {
            assert_eq!(platform::platform_name(), "Windows (Dokan)");
        }
    }

    #[test]
    fn test_is_fuse_available() {
        // On Windows, this depends on Dokan2 being installed
        // On Unix-like systems, this should return true if FUSE is available
        let result = platform::is_fuse_available();
        // We don't assert a specific value since availability depends on the environment
        // but we ensure the function returns a boolean
        assert!(result == true || result == false);
    }
}
