// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Error types and result tracking for the X-Plane simulator.

use autoortho_lib::tiles::coords::TileCoord;
use std::fmt;
use std::time::Duration;

/// Error type for tile read operations.
#[derive(Debug)]
pub enum TileError {
    /// I/O error reading the file from the mount
    IoError(String),
    /// The tile data is not a valid DDS file
    InvalidFormat(String),
    /// The tile was not found in the mount
    NotFound,
    /// A read timeout occurred
    Timeout,
    /// The mount path itself is not accessible
    MountUnavailable,
}

impl fmt::Display for TileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "I/O error: {}", e),
            Self::InvalidFormat(e) => write!(f, "Invalid DDS format: {}", e),
            Self::NotFound => write!(f, "Tile not found"),
            Self::Timeout => write!(f, "Read timeout"),
            Self::MountUnavailable => write!(f, "Mount path unavailable"),
        }
    }
}

impl std::error::Error for TileError {}

/// Result of attempting to read a single tile.
pub struct TileResult {
    pub coord: TileCoord,
    pub filename: String,
    pub elapsed: Duration,
    pub result: Result<Vec<u8>, String>,
}

impl TileResult {
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    pub fn is_err(&self) -> bool {
        self.result.is_err()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_error_display() {
        let err = TileError::IoError("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = TileError::NotFound;
        assert!(err.to_string().contains("not found"));

        let err = TileError::Timeout;
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_tile_result_is_ok() {
        let coord = TileCoord::new(100, 200, 16).unwrap();
        let result = TileResult {
            coord,
            filename: "100_200_BI16.dds".to_string(),
            elapsed: Duration::from_millis(10),
            result: Ok(vec![0u8; 100]),
        };
        assert!(result.is_ok());
        assert!(!result.is_err());
    }

    #[test]
    fn test_tile_result_is_err() {
        let coord = TileCoord::new(100, 200, 16).unwrap();
        let result = TileResult {
            coord,
            filename: "100_200_BI16.dds".to_string(),
            elapsed: Duration::from_millis(10),
            result: Err("io error".to_string()),
        };
        assert!(!result.is_ok());
        assert!(result.is_err());
    }
}
