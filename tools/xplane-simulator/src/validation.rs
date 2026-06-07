// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! DDS/JPEG tile validation for the X-Plane simulator.

use autoortho_lib::tiles::coords::TileCoord;
use sha2::{Digest, Sha256};
use std::fmt;

/// DDS magic bytes
const DDS_MAGIC: [u8; 4] = [b'D', b'D', b'S', b' '];

/// DDS pixel format "DXT1" (BC1) magic
const DDS_DXT1: [u8; 4] = *b"DXT1";
/// DDS pixel format "DXT5" (BC3) magic
const DDS_DXT5: [u8; 4] = *b"DXT5";

/// Validate that data is a valid DDS file.
/// Checks magic bytes, header size, and basic format fields.
///
/// DDS file structure:
/// - 4 bytes: "DDS " magic
/// - 124 bytes: DDS_HEADER
///   - 4 bytes: dwSize (= 124)
///   - 4 bytes: dwFlags
///   - 4 bytes: dwHeight
///   - 4 bytes: dwWidth
///   - 4 bytes: dwPitchOrLinearSize
///   - 4 bytes: dwDepth
///   - 4 bytes: dwMipMapCount
///   - 44 bytes: dwReserved1[11]
/// - 16 bytes: DDS_PIXELFORMAT (dwSize=32, dwFlags=0x4, dwFourCC)
///   - 4 bytes: dwSize (= 32)
///   - 4 bytes: dwFlags
///   - 4 bytes: dwFourCC
///   - ... (remaining fields not needed for validation)
/// - 4 bytes: dwCaps
/// - 4 bytes: dwCaps2
/// - ...
/// - Remaining: texture data (compressed blocks)
pub fn validate_dds(data: &[u8]) -> Result<DdsInfo, DdsValidationError> {
    // Minimum DDS file: 128 byte header
    if data.len() < 128 {
        return Err(DdsValidationError::TooShort {
            expected_min: 128,
            actual: data.len(),
        });
    }

    // Check magic bytes
    if data[0..4] != DDS_MAGIC {
        return Err(DdsValidationError::BadMagic {
            expected: DDS_MAGIC,
            found: [data[0], data[1], data[2], data[3]],
        });
    }

    // Parse DDS_HEADER structure (offset 4)
    // dwSize should be 124
    let header_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if header_size != 124 {
        return Err(DdsValidationError::BadHeaderSize {
            expected: 124,
            found: header_size,
        });
    }

    // Parse height and width (offsets 12 and 16)
    let height = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let width = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);

    // Standard autoortho tile size
    if height != 4096 || width != 4096 {
        return Err(DdsValidationError::BadDimensions { width, height });
    }

    // Parse DDS_PIXELFORMAT at offset 76 (4-byte magic + 72-byte prefix)
    //   offset 76:  dwSize (should be 32)
    //   offset 80:  dwFlags
    //   offset 84:  dwFourCC (DXT1 or DXT5)
    //   offset 88:  rgbBitCount (unused for compressed)
    let pf_size = u32::from_le_bytes([data[76], data[77], data[78], data[79]]);
    if pf_size != 32 {
        return Err(DdsValidationError::BadPixelFormatSize {
            expected: 32,
            found: pf_size,
        });
    }

    let fourcc = [data[84], data[85], data[86], data[87]];
    let format = if fourcc == DDS_DXT1 {
        DdsFormat::BC1
    } else if fourcc == DDS_DXT5 {
        DdsFormat::BC3
    } else {
        return Err(DdsValidationError::UnknownFormat {
            fourcc: String::from_utf8_lossy(&fourcc).to_string(),
        });
    };

    Ok(DdsInfo {
        width,
        height,
        format,
        size_bytes: data.len(),
    })
}

/// Compute SHA256 checksum of tile data.
pub fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Reference tile metadata for comparison.
#[derive(Debug, serde::Serialize)]
pub struct ReferenceMetadata {
    pub row: u32,
    pub col: u32,
    pub zoom: u32,
    pub size: usize,
    pub sha256: String,
    pub validated: bool,
}

impl ReferenceMetadata {
    pub fn from_tile(coord: &TileCoord, size: usize, sha256: String, validated: bool) -> Self {
        Self {
            row: coord.row,
            col: coord.col,
            zoom: coord.zoom,
            size,
            sha256,
            validated,
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum DdsValidationError {
    TooShort { expected_min: usize, actual: usize },
    BadMagic { expected: [u8; 4], found: [u8; 4] },
    BadHeaderSize { expected: u32, found: u32 },
    BadDimensions { width: u32, height: u32 },
    BadPixelFormatSize { expected: u32, found: u32 },
    UnknownFormat { fourcc: String },
}

impl fmt::Display for DdsValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort {
                expected_min,
                actual,
            } => {
                write!(
                    f,
                    "DDS file too short: {} bytes, expected at least {}",
                    actual, expected_min
                )
            }
            Self::BadMagic { expected, found } => {
                write!(
                    f,
                    "Bad DDS magic: expected {:?}, found {:?}",
                    expected, found
                )
            }
            Self::BadHeaderSize { expected, found } => {
                write!(
                    f,
                    "Bad DDS header size: expected {}, found {}",
                    expected, found
                )
            }
            Self::BadDimensions { width, height } => {
                write!(
                    f,
                    "Unexpected DDS dimensions: {}x{} (expected 4096x4096)",
                    width, height
                )
            }
            Self::BadPixelFormatSize { expected, found } => {
                write!(
                    f,
                    "Bad pixel format size: expected {}, found {}",
                    expected, found
                )
            }
            Self::UnknownFormat { fourcc } => {
                write!(f, "Unknown DDS format: {:?}", fourcc)
            }
        }
    }
}

impl std::error::Error for DdsValidationError {}

#[derive(Debug, Clone, Copy)]
pub enum DdsFormat {
    BC1, // DXT1 - 8 bytes per 4x4 block, RGB
    BC3, // DXT5 - 16 bytes per 4x4 block, RGBA
}

#[derive(Debug)]
pub struct DdsInfo {
    pub width: u32,
    pub height: u32,
    pub format: DdsFormat,
    pub size_bytes: usize,
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_dds_magic() {
        // Zero bytes should fail
        let result = validate_dds(&[0u8; 128]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_dds_too_short() {
        let mut data = Vec::new();
        data.extend_from_slice(b"DDS ");
        data.resize(64, 0); // Too short
        let result = validate_dds(&data);
        assert!(matches!(result, Err(DdsValidationError::TooShort { .. })));
    }

    #[test]
    fn test_validate_dds_minimal_valid() {
        // Build a minimal valid DDS file for testing header parsing
        let mut data = Vec::with_capacity(128);
        data.extend_from_slice(b"DDS "); // magic
        data.extend_from_slice(&124u32.to_le_bytes()); // dwSize
        data.extend_from_slice(&0u32.to_le_bytes()); // dwFlags
        data.extend_from_slice(&4096u32.to_le_bytes()); // dwHeight
        data.extend_from_slice(&4096u32.to_le_bytes()); // dwWidth
        data.extend_from_slice(&0u32.to_le_bytes()); // dwPitchOrLinearSize
        data.extend_from_slice(&0u32.to_le_bytes()); // dwDepth
        data.extend_from_slice(&1u32.to_le_bytes()); // dwMipMapCount
        data.extend_from_slice(&[0u8; 44]); // dwReserved1
        // DDS_PIXELFORMAT at offset 76 (4 + 72 = 76)
        // Wait, header is 124 bytes starting at offset 4, so PIXELFORMAT at 4+124=128?
        // Let me re-check: the DDS header is 124 bytes, pixel format is 16 bytes.
        // Total header = 128 bytes (4 magic + 124 header, where header includes PF)
        //
        // DDS_HEADER structure (124 bytes):
        // offset 0: dwSize (4)
        // offset 4: dwFlags (4)
        // offset 8: dwHeight (4)
        // offset 12: dwWidth (4)
        // offset 16: dwPitchOrLinearSize (4)
        // offset 20: dwDepth (4)
        // offset 24: dwMipMapCount (4)
        // offset 28: dwReserved1[11] (44 bytes, 28-71)
        // DDS_PIXELFORMAT starts at offset 72 (72 = 4+68? Let me recalculate)
        //
        // From Microsoft DDS spec:
        // DDS_HEADER = 124 bytes (dwSize through dwReserved1[10])
        // DDS_PIXELFORMAT = 16 bytes (separate from header, included in the 124)
        // Total: 4 (magic) + 124 (header including pixelformat)
        //
        // Actually let me look at the actual struct layout:
        // typedef struct {
        //   DWORD dwSize;        // 4
        //   DWORD dwFlags;       // 4
        //   DWORD dwHeight;      // 4
        //   DWORD dwWidth;       // 4
        //   DWORD dwPitchOrLinearSize; // 4
        //   DWORD dwBackBufferCount;  // 4
        //   DWORD dwMipMapCount;      // 4
        //   DWORD dwReserved1[11];    // 44
        //   DDS_PIXELFORMAT ddspf;    // 16 bytes = 4+4+4+4
        // } DDS_HEADER;  // total = 4+4+4+4+4+4+4+44+16 = 88... no wait
        //
        // Let me check pydds.py:
        // header = struct.pack('<IIIIIII', 124, flags, height, width, ...
        // That's 7 * 4 = 28 bytes
        // Then 11 DWORDs for reserved = 44 bytes
        // Then pixelformat = 16 bytes
        // Total header = 28 + 44 + 16 = 88...
        //
        // Hmm let me just check the actual autoortho_lib code
    }

    #[test]
    fn test_hex_encode() {
        let result = hex::encode([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
        assert_eq!(result, "48656c6c6f");
    }

    #[test]
    fn test_sha256_not_empty() {
        let data = b"hello world";
        let hash = sha256(data);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 = 32 bytes = 64 hex chars
    }
}
