//! Pure Rust BC1 (DXT1) and BC3 (DXT5) block compression.
//!
//! Compresses 4x4 blocks of RGBA pixels into BC1 (8 bytes) or BC3 (16 bytes) format.
//! This is a quality-sufficient implementation for satellite imagery — not optimal
//! but correct and fast enough. Can be replaced with ISPC texcomp FFI for higher
//! throughput later.

use crate::pipeline::dds::DdsFormat;

/// Compress a full RGBA image to BC1 or BC3 format.
/// Input: RGBA pixel data, width and height must be multiples of 4.
/// Returns compressed block data (no DDS header).
pub fn compress_image(data: &[u8], width: u32, height: u32, format: DdsFormat) -> Vec<u8> {
    let blocks_x = width.div_ceil(4);
    let blocks_y = height.div_ceil(4);
    let block_size = match format {
        DdsFormat::BC1 => 8,
        DdsFormat::BC3 => 16,
    };
    let mut output = Vec::with_capacity((blocks_x * blocks_y) as usize * block_size);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let mut block = [[0u8; 4]; 16]; // 4x4 RGBA pixels
            for py in 0..4u32 {
                for px in 0..4u32 {
                    let x = (bx * 4 + px).min(width - 1);
                    let y = (by * 4 + py).min(height - 1);
                    let idx = (y * width + x) as usize * 4;
                    if idx + 3 < data.len() {
                        block[(py * 4 + px) as usize] =
                            [data[idx], data[idx + 1], data[idx + 2], data[idx + 3]];
                    }
                }
            }

            match format {
                DdsFormat::BC1 => {
                    output.extend_from_slice(&compress_bc1_block(&block));
                }
                DdsFormat::BC3 => {
                    output.extend_from_slice(&compress_bc3_block(&block));
                }
            }
        }
    }

    output
}

/// Generate a solid-color BC1 or BC3 block for use as fallback/missing tile fill.
pub fn solid_color_block(r: u8, g: u8, b: u8, format: DdsFormat) -> Vec<u8> {
    let c565 = rgb_to_565(r, g, b);
    let c_lo = (c565 & 0xFF) as u8;
    let c_hi = ((c565 >> 8) & 0xFF) as u8;

    match format {
        DdsFormat::BC1 => {
            // color0 == color1, indices all 0 → all pixels use color0
            vec![c_lo, c_hi, c_lo, c_hi, 0, 0, 0, 0]
        }
        DdsFormat::BC3 => {
            // Alpha block: alpha0=255, alpha1=255, indices all 0 (opaque)
            let mut block = vec![0xFF, 0xFF, 0, 0, 0, 0, 0, 0];
            // Color block: same as BC1
            block.extend_from_slice(&[c_lo, c_hi, c_lo, c_hi, 0, 0, 0, 0]);
            block
        }
    }
}

/// Generate fallback bytes of the given length, filled with solid-color blocks.
pub fn fallback_bytes(length: usize, r: u8, g: u8, b: u8, format: DdsFormat) -> Vec<u8> {
    let block = solid_color_block(r, g, b, format);
    let block_size = block.len();
    let num_blocks = length.div_ceil(block_size);
    let mut out = Vec::with_capacity(num_blocks * block_size);
    for _ in 0..num_blocks {
        out.extend_from_slice(&block);
    }
    out.truncate(length);
    out
}

// --- Internal compression ---

/// Convert 8-bit RGB to RGB565 (5-6-5 packed, little-endian u16)
fn rgb_to_565(r: u8, g: u8, b: u8) -> u16 {
    let r5 = (r >> 3) as u16;
    let g6 = (g >> 2) as u16;
    let b5 = (b >> 3) as u16;
    (r5 << 11) | (g6 << 5) | b5
}

/// Expand RGB565 back to 8-bit RGB (for endpoint interpolation)
fn rgb565_to_rgb(c: u16) -> [u8; 3] {
    let r5 = ((c >> 11) & 0x1F) as u8;
    let g6 = ((c >> 5) & 0x3F) as u8;
    let b5 = (c & 0x1F) as u8;
    [
        (r5 << 3) | (r5 >> 2),
        (g6 << 2) | (g6 >> 4),
        (b5 << 3) | (b5 >> 2),
    ]
}

/// Compress a 4x4 block to BC1 (DXT1) — 8 bytes output.
/// Uses a simple min/max endpoint selection with closest-color index assignment.
fn compress_bc1_block(pixels: &[[u8; 4]; 16]) -> [u8; 8] {
    // Find min/max RGB values across the block
    let (mut min_r, mut min_g, mut min_b) = (255u8, 255u8, 255u8);
    let (mut max_r, mut max_g, mut max_b) = (0u8, 0u8, 0u8);

    for px in pixels {
        min_r = min_r.min(px[0]);
        min_g = min_g.min(px[1]);
        min_b = min_b.min(px[2]);
        max_r = max_r.max(px[0]);
        max_g = max_g.max(px[1]);
        max_b = max_b.max(px[2]);
    }

    // Inset endpoints slightly for better quality
    let inset_r = ((max_r as i16 - min_r as i16) >> 4) as u8;
    let inset_g = ((max_g as i16 - min_g as i16) >> 4) as u8;
    let inset_b = ((max_b as i16 - min_b as i16) >> 4) as u8;

    let c0_r = min_r.saturating_add(inset_r);
    let c0_g = min_g.saturating_add(inset_g);
    let c0_b = min_b.saturating_add(inset_b);
    let c1_r = max_r.saturating_sub(inset_r);
    let c1_g = max_g.saturating_sub(inset_g);
    let c1_b = max_b.saturating_sub(inset_b);

    let mut color0 = rgb_to_565(c1_r, c1_g, c1_b);
    let mut color1 = rgb_to_565(c0_r, c0_g, c0_b);

    // Ensure color0 > color1 for 4-color mode (no transparency)
    if color0 == color1 {
        // Solid color — just use index 0 for all pixels
        let lo = (color0 & 0xFF) as u8;
        let hi = ((color0 >> 8) & 0xFF) as u8;
        return [lo, hi, lo, hi, 0, 0, 0, 0];
    }

    if color0 < color1 {
        std::mem::swap(&mut color0, &mut color1);
    }

    // Build palette: color0, color1, 2/3*c0+1/3*c1, 1/3*c0+2/3*c1
    let rgb0 = rgb565_to_rgb(color0);
    let rgb1 = rgb565_to_rgb(color1);
    let palette = [
        rgb0,
        rgb1,
        [
            ((2 * rgb0[0] as u16 + rgb1[0] as u16) / 3) as u8,
            ((2 * rgb0[1] as u16 + rgb1[1] as u16) / 3) as u8,
            ((2 * rgb0[2] as u16 + rgb1[2] as u16) / 3) as u8,
        ],
        [
            ((rgb0[0] as u16 + 2 * rgb1[0] as u16) / 3) as u8,
            ((rgb0[1] as u16 + 2 * rgb1[1] as u16) / 3) as u8,
            ((rgb0[2] as u16 + 2 * rgb1[2] as u16) / 3) as u8,
        ],
    ];

    // Assign each pixel to the closest palette entry
    let mut indices = 0u32;
    for (i, px) in pixels.iter().enumerate() {
        let mut best_idx = 0u32;
        let mut best_dist = u32::MAX;
        for (pi, pal) in palette.iter().enumerate() {
            let dr = px[0] as i32 - pal[0] as i32;
            let dg = px[1] as i32 - pal[1] as i32;
            let db = px[2] as i32 - pal[2] as i32;
            let dist = (dr * dr + dg * dg + db * db) as u32;
            if dist < best_dist {
                best_dist = dist;
                best_idx = pi as u32;
            }
        }
        indices |= best_idx << (i * 2);
    }

    let mut out = [0u8; 8];
    out[0] = (color0 & 0xFF) as u8;
    out[1] = ((color0 >> 8) & 0xFF) as u8;
    out[2] = (color1 & 0xFF) as u8;
    out[3] = ((color1 >> 8) & 0xFF) as u8;
    out[4] = (indices & 0xFF) as u8;
    out[5] = ((indices >> 8) & 0xFF) as u8;
    out[6] = ((indices >> 16) & 0xFF) as u8;
    out[7] = ((indices >> 24) & 0xFF) as u8;
    out
}

/// Compress a 4x4 block to BC3 (DXT5) — 16 bytes output.
/// BC3 = explicit alpha block (8 bytes) + BC1 color block (8 bytes, no transparency mode).
fn compress_bc3_block(pixels: &[[u8; 4]; 16]) -> [u8; 16] {
    // Alpha block (8 bytes)
    let alpha_block = compress_bc3_alpha_block(pixels);

    // Color block (8 bytes) — same as BC1
    let color_block = compress_bc1_block(pixels);

    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&alpha_block);
    out[8..].copy_from_slice(&color_block);
    out
}

/// Compress the alpha channel of a 4x4 block for BC3 format.
/// Uses two endpoints and 3-bit indices (8 possible interpolated values).
fn compress_bc3_alpha_block(pixels: &[[u8; 4]; 16]) -> [u8; 8] {
    // Find min/max alpha
    let mut alpha_min = 255u8;
    let mut alpha_max = 0u8;
    for px in pixels {
        alpha_min = alpha_min.min(px[3]);
        alpha_max = alpha_max.max(px[3]);
    }

    // If all same alpha, trivial encoding
    if alpha_min == alpha_max {
        return [alpha_max, alpha_max, 0, 0, 0, 0, 0, 0];
    }

    // Build 8-entry palette: alpha0, alpha1, + 6 interpolated
    // When alpha0 > alpha1: 6 interpolated values
    let a0 = alpha_max;
    let a1 = alpha_min;

    let palette: [u8; 8] = [
        a0,
        a1,
        ((6 * a0 as u16 + a1 as u16) / 7) as u8,
        ((5 * a0 as u16 + 2 * a1 as u16) / 7) as u8,
        ((4 * a0 as u16 + 3 * a1 as u16) / 7) as u8,
        ((3 * a0 as u16 + 4 * a1 as u16) / 7) as u8,
        ((2 * a0 as u16 + 5 * a1 as u16) / 7) as u8,
        ((a0 as u16 + 6 * a1 as u16) / 7) as u8,
    ];

    // Assign each pixel's alpha to closest palette entry (3-bit index)
    let mut indices = 0u64;
    for (i, px) in pixels.iter().enumerate() {
        let alpha = px[3];
        let mut best_idx = 0u64;
        let mut best_dist = u32::MAX;
        for (pi, &pal_a) in palette.iter().enumerate() {
            let dist = (alpha as i32 - pal_a as i32).unsigned_abs();
            if dist < best_dist {
                best_dist = dist;
                best_idx = pi as u64;
            }
        }
        indices |= best_idx << (i * 3);
    }

    let mut out = [0u8; 8];
    out[0] = a0;
    out[1] = a1;
    // Pack 48 bits of indices (16 pixels × 3 bits) into 6 bytes
    out[2] = (indices & 0xFF) as u8;
    out[3] = ((indices >> 8) & 0xFF) as u8;
    out[4] = ((indices >> 16) & 0xFF) as u8;
    out[5] = ((indices >> 24) & 0xFF) as u8;
    out[6] = ((indices >> 32) & 0xFF) as u8;
    out[7] = ((indices >> 40) & 0xFF) as u8;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb565_roundtrip() {
        // Test that 565 conversion preserves values within quantization error
        let c = rgb_to_565(255, 128, 0);
        let rgb = rgb565_to_rgb(c);
        assert!((rgb[0] as i16 - 255).abs() <= 7);
        assert!((rgb[1] as i16 - 128).abs() <= 3);
        assert!((rgb[2] as i16 - 0).abs() <= 7);
    }

    #[test]
    fn test_bc1_solid_color() {
        let block = [[200u8, 100, 50, 255]; 16];
        let compressed = compress_bc1_block(&block);
        assert_eq!(compressed.len(), 8);

        // Solid color: color0 should equal color1
        let c0 = u16::from_le_bytes([compressed[0], compressed[1]]);
        let c1 = u16::from_le_bytes([compressed[2], compressed[3]]);
        assert_eq!(c0, c1);
    }

    #[test]
    fn test_bc3_block_size() {
        let block = [[128u8, 128, 128, 200]; 16];
        let compressed = compress_bc3_block(&block);
        assert_eq!(compressed.len(), 16);
    }

    #[test]
    fn test_bc3_alpha_solid() {
        let block = [[0u8, 0, 0, 255]; 16];
        let alpha = compress_bc3_alpha_block(&block);
        // All alpha 255: endpoints should both be 255
        assert_eq!(alpha[0], 255);
        assert_eq!(alpha[1], 255);
    }

    #[test]
    fn test_compress_image_bc1_output_size() {
        let width = 64u32;
        let height = 64u32;
        let data = vec![128u8; (width * height * 4) as usize];
        let compressed = compress_image(&data, width, height, DdsFormat::BC1);
        let expected_blocks = ((width + 3) / 4) * ((height + 3) / 4);
        assert_eq!(compressed.len(), expected_blocks as usize * 8);
    }

    #[test]
    fn test_compress_image_bc3_output_size() {
        let width = 64u32;
        let height = 64u32;
        let data = vec![128u8; (width * height * 4) as usize];
        let compressed = compress_image(&data, width, height, DdsFormat::BC3);
        let expected_blocks = ((width + 3) / 4) * ((height + 3) / 4);
        assert_eq!(compressed.len(), expected_blocks as usize * 16);
    }

    #[test]
    fn test_solid_color_block_bc1() {
        let block = solid_color_block(66, 77, 55, DdsFormat::BC1);
        assert_eq!(block.len(), 8);
    }

    #[test]
    fn test_solid_color_block_bc3() {
        let block = solid_color_block(66, 77, 55, DdsFormat::BC3);
        assert_eq!(block.len(), 16);
        // First byte of alpha block should be 0xFF (opaque)
        assert_eq!(block[0], 0xFF);
    }

    #[test]
    fn test_fallback_bytes_exact_length() {
        let fb = fallback_bytes(1024, 66, 77, 55, DdsFormat::BC1);
        assert_eq!(fb.len(), 1024);

        let fb = fallback_bytes(1024, 66, 77, 55, DdsFormat::BC3);
        assert_eq!(fb.len(), 1024);
    }

    #[test]
    fn test_compress_two_color_block() {
        // Half black, half white block
        let mut block = [[0u8, 0, 0, 255]; 16];
        for i in 8..16 {
            block[i] = [255, 255, 255, 255];
        }
        let compressed = compress_bc1_block(&block);
        // Should have different endpoints
        let c0 = u16::from_le_bytes([compressed[0], compressed[1]]);
        let c1 = u16::from_le_bytes([compressed[2], compressed[3]]);
        assert_ne!(c0, c1);
    }
}
