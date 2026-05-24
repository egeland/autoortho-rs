//! Zoom-level coordinate math for tile/chunk mapping.
//!
//! Handles the bit-shifting math that maps between tile coordinates
//! at different zoom levels. This is critical for:
//! - Determining which chunks to fetch for a tile at a given zoom
//! - Fallback resolution (finding lower-zoom parent chunks)
//! - Mipmap-to-zoom-level mapping

use serde::{Deserialize, Serialize};

/// A zoom rule: at or above this AGL altitude, use this zoom level.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZoomRule {
    /// Minimum AGL altitude in feet for this rule
    pub min_altitude_ft: f32,
    /// Zoom level to use at this altitude
    pub zoom_level: u32,
}

impl Default for ZoomRule {
    fn default() -> Self {
        Self {
            min_altitude_ft: 0.0,
            zoom_level: 19,
        }
    }
}

/// Scale a coordinate value by a zoom level difference.
///
/// Positive `zoom_diff` means zooming in (higher ZL) → divide by 2^diff.
/// Negative `zoom_diff` means zooming out (lower ZL) → multiply by 2^|diff|.
///
/// Matches Python's `scale_by_zoom_diff()`.
pub fn scale_by_zoom_diff(value: u32, zoom_diff: i32) -> u32 {
    if zoom_diff >= 0 {
        let shift = (zoom_diff as u32).min(31);
        value >> shift
    } else {
        let shift = ((-zoom_diff) as u32).min(31);
        value.saturating_mul(1u32.checked_shl(shift).unwrap_or(u32::MAX))
    }
}

/// Calculate the chunk grid parameters for a tile.
///
/// Given a tile's position and zoom levels, returns the chunk-fetch
/// parameters: the starting col/row, width/height in chunks, and
/// the zoom level to fetch at.
///
/// - `tile_col`, `tile_row`: tile position from the .ter filename
/// - `tile_width`, `tile_height`: nominal chunks (e.g., 16)
/// - `tilename_zoom`: zoom level from the .ter filename
/// - `max_zoom`: actual zoom level to fetch chunks at
pub fn chunk_grid_params(
    tile_col: u32,
    tile_row: u32,
    tile_width: u32,
    tile_height: u32,
    tilename_zoom: u32,
    max_zoom: u32,
) -> ChunkGrid {
    let zoom_diff = tilename_zoom as i32 - max_zoom as i32;

    let col = scale_by_zoom_diff(tile_col, zoom_diff);
    let row = scale_by_zoom_diff(tile_row, zoom_diff);
    let width = scale_by_zoom_diff(tile_width, zoom_diff).max(1);
    let height = scale_by_zoom_diff(tile_height, zoom_diff).max(1);

    ChunkGrid {
        col,
        row,
        width,
        height,
        zoom: max_zoom,
        pixel_width: width * 256,
        pixel_height: height * 256,
    }
}

/// Parameters for fetching chunks in a grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkGrid {
    /// Starting column in chunk coordinates
    pub col: u32,
    /// Starting row in chunk coordinates
    pub row: u32,
    /// Number of chunks wide
    pub width: u32,
    /// Number of chunks tall
    pub height: u32,
    /// Zoom level to fetch at
    pub zoom: u32,
    /// Total pixel width (width * 256)
    pub pixel_width: u32,
    /// Total pixel height (height * 256)
    pub pixel_height: u32,
}

impl ChunkGrid {
    /// Total number of chunks in the grid
    pub fn total_chunks(&self) -> u32 {
        self.width * self.height
    }

    /// Get chunk coordinate for a given index (row-major order).
    pub fn chunk_at(&self, index: u32) -> (u32, u32) {
        let chunk_col = self.col + (index % self.width);
        let chunk_row = self.row + (index / self.width);
        (chunk_col, chunk_row)
    }

    /// Iterator over all chunk (col, row) pairs in row-major order.
    pub fn iter_chunks(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (0..self.total_chunks()).map(move |i| self.chunk_at(i))
    }
}

/// Map a mipmap level to the zoom level and chunk grid for that mipmap.
///
/// Mipmap 0 = highest detail (max_zoom, 16×16 chunks for standard tiles)
/// Mipmap 1 = max_zoom - 1 (8×8 chunks → 2048×2048 pixels)
/// Mipmap 2 = max_zoom - 2 (4×4 chunks → 1024×1024 pixels)
/// etc.
pub fn mipmap_to_zoom(
    mipmap_level: u32,
    tile_col: u32,
    tile_row: u32,
    tile_width: u32,
    tile_height: u32,
    tilename_zoom: u32,
    max_zoom: u32,
) -> Option<ChunkGrid> {
    if mipmap_level as i32 > max_zoom as i32 {
        return None;
    }

    let effective_zoom = max_zoom - mipmap_level;
    Some(chunk_grid_params(
        tile_col,
        tile_row,
        tile_width,
        tile_height,
        tilename_zoom,
        effective_zoom,
    ))
}

/// Calculate the parent chunk at a lower zoom that contains the given chunk.
///
/// Used for fallback resolution: when a chunk at (col, row, zoom) is missing,
/// find the parent chunk at (zoom - levels_up) that covers it.
///
/// Returns (parent_col, parent_row, parent_zoom, crop_x, crop_y, crop_size)
/// where crop_x/y are the pixel offsets within the parent to extract,
/// and crop_size is the size of the region to crop (in parent pixels).
pub fn parent_chunk(col: u32, row: u32, zoom: u32, levels_up: u32) -> Option<ParentChunk> {
    if levels_up > zoom {
        return None;
    }

    let parent_zoom = zoom - levels_up;
    let scale = 1u32 << levels_up;

    let parent_col = col / scale;
    let parent_row = row / scale;

    // Position within the parent chunk (0..scale-1 in each dimension)
    let sub_col = col % scale;
    let sub_row = row % scale;

    let crop_size = 256 / scale;
    let crop_x = sub_col * crop_size;
    let crop_y = sub_row * crop_size;

    Some(ParentChunk {
        col: parent_col,
        row: parent_row,
        zoom: parent_zoom,
        crop_x,
        crop_y,
        crop_size,
        upscale_factor: scale,
    })
}

/// Information about a parent chunk for fallback resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentChunk {
    pub col: u32,
    pub row: u32,
    pub zoom: u32,
    /// X pixel offset within the parent to crop
    pub crop_x: u32,
    /// Y pixel offset within the parent to crop
    pub crop_y: u32,
    /// Size of the crop region in parent pixels
    pub crop_size: u32,
    /// Factor to upscale the crop back to 256×256
    pub upscale_factor: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_by_zoom_diff_positive() {
        // Zooming in: divide
        assert_eq!(scale_by_zoom_diff(1024, 1), 512);
        assert_eq!(scale_by_zoom_diff(1024, 2), 256);
        assert_eq!(scale_by_zoom_diff(16, 1), 8);
    }

    #[test]
    fn test_scale_by_zoom_diff_negative() {
        // Zooming out: multiply
        assert_eq!(scale_by_zoom_diff(512, -1), 1024);
        assert_eq!(scale_by_zoom_diff(256, -2), 1024);
    }

    #[test]
    fn test_scale_by_zoom_diff_zero() {
        assert_eq!(scale_by_zoom_diff(42, 0), 42);
    }

    #[test]
    fn test_chunk_grid_same_zoom() {
        // tilename_zoom == max_zoom → no scaling
        let grid = chunk_grid_params(100, 200, 16, 16, 16, 16);
        assert_eq!(grid.col, 100);
        assert_eq!(grid.row, 200);
        assert_eq!(grid.width, 16);
        assert_eq!(grid.height, 16);
        assert_eq!(grid.zoom, 16);
        assert_eq!(grid.pixel_width, 4096);
    }

    #[test]
    fn test_chunk_grid_zoom_down() {
        // tilename_zoom=18, max_zoom=17 → zoom_diff=1, divide by 2
        let grid = chunk_grid_params(200, 400, 16, 16, 18, 17);
        assert_eq!(grid.col, 100);
        assert_eq!(grid.row, 200);
        assert_eq!(grid.width, 8);
        assert_eq!(grid.height, 8);
        assert_eq!(grid.pixel_width, 2048);
    }

    #[test]
    fn test_chunk_grid_zoom_up() {
        // tilename_zoom=15, max_zoom=16 → zoom_diff=-1, multiply by 2
        let grid = chunk_grid_params(50, 100, 16, 16, 15, 16);
        assert_eq!(grid.col, 100);
        assert_eq!(grid.row, 200);
        assert_eq!(grid.width, 32);
        assert_eq!(grid.height, 32);
    }

    #[test]
    fn test_chunk_at_row_major() {
        let grid = chunk_grid_params(10, 20, 4, 4, 16, 16);
        // Index 0 → (10, 20), Index 1 → (11, 20), ...
        assert_eq!(grid.chunk_at(0), (10, 20));
        assert_eq!(grid.chunk_at(1), (11, 20));
        assert_eq!(grid.chunk_at(4), (10, 21)); // Next row
        assert_eq!(grid.chunk_at(15), (13, 23)); // Last chunk
    }

    #[test]
    fn test_iter_chunks_count() {
        let grid = chunk_grid_params(0, 0, 16, 16, 16, 16);
        assert_eq!(grid.iter_chunks().count(), 256);
    }

    #[test]
    fn test_mipmap_to_zoom_level0() {
        let grid = mipmap_to_zoom(0, 100, 200, 16, 16, 16, 16).unwrap();
        assert_eq!(grid.zoom, 16);
        assert_eq!(grid.width, 16);
    }

    #[test]
    fn test_mipmap_to_zoom_level1() {
        let grid = mipmap_to_zoom(1, 100, 200, 16, 16, 16, 16).unwrap();
        assert_eq!(grid.zoom, 15);
        assert_eq!(grid.width, 8);
        assert_eq!(grid.pixel_width, 2048);
    }

    #[test]
    fn test_mipmap_to_zoom_level4() {
        let grid = mipmap_to_zoom(4, 100, 200, 16, 16, 16, 16).unwrap();
        assert_eq!(grid.zoom, 12);
        assert_eq!(grid.width, 1);
        assert_eq!(grid.pixel_width, 256);
    }

    #[test]
    fn test_parent_chunk_one_level() {
        let parent = parent_chunk(101, 201, 16, 1).unwrap();
        assert_eq!(parent.col, 50);
        assert_eq!(parent.row, 100);
        assert_eq!(parent.zoom, 15);
        assert_eq!(parent.upscale_factor, 2);
        assert_eq!(parent.crop_size, 128);
        // col=101 → sub_col=101%2=1 → crop_x=128
        assert_eq!(parent.crop_x, 128);
        // row=201 → sub_row=201%2=1 → crop_y=128
        assert_eq!(parent.crop_y, 128);
    }

    #[test]
    fn test_parent_chunk_two_levels() {
        let parent = parent_chunk(100, 200, 16, 2).unwrap();
        assert_eq!(parent.col, 25);
        assert_eq!(parent.row, 50);
        assert_eq!(parent.zoom, 14);
        assert_eq!(parent.upscale_factor, 4);
        assert_eq!(parent.crop_size, 64);
    }

    #[test]
    fn test_parent_chunk_origin() {
        let parent = parent_chunk(0, 0, 16, 1).unwrap();
        assert_eq!(parent.col, 0);
        assert_eq!(parent.row, 0);
        assert_eq!(parent.crop_x, 0);
        assert_eq!(parent.crop_y, 0);
    }

    #[test]
    fn test_parent_chunk_too_many_levels() {
        assert!(parent_chunk(10, 20, 5, 6).is_none());
    }
}
