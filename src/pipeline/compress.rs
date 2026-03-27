//! BC1 (DXT1) and BC3 (DXT5) block compression using texpresso.
//!
//! Compresses RGBA images into BC1 (8 bytes/block) or BC3 (16 bytes/block) format.
//! Uses texpresso with rayon for parallel block compression.
//! Solid-color and fallback helpers use trivial hand-rolled encoding.

use crate::pipeline::dds::DdsFormat;

/// Compress a full RGBA image to BC1 or BC3 format.
/// Input: RGBA pixel data, width and height must be multiples of 4.
/// Returns compressed block data (no DDS header).
pub fn compress_image(data: &[u8], width: u32, height: u32, format: DdsFormat) -> Vec<u8> {
    let tex_format = match format {
        DdsFormat::BC1 => texpresso::Format::Bc1,
        DdsFormat::BC3 => texpresso::Format::Bc3,
    };
    let w = width as usize;
    let h = height as usize;
    let mut output = vec![0u8; tex_format.compressed_size(w, h)];
    tex_format.compress(data, w, h, texpresso::Params::default(), &mut output);
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

/// Convert 8-bit RGB to RGB565 (5-6-5 packed, little-endian u16)
fn rgb_to_565(r: u8, g: u8, b: u8) -> u16 {
    let r5 = (r >> 3) as u16;
    let g6 = (g >> 2) as u16;
    let b5 = (b >> 3) as u16;
    (r5 << 11) | (g6 << 5) | b5
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_compress_image_bc1_nonzero_output() {
        // Two-tone image should produce non-trivial compressed data
        let width = 8u32;
        let height = 8u32;
        let mut data = vec![0u8; (width * height * 4) as usize];
        // Top half black, bottom half white
        for y in 4..8 {
            for x in 0..8 {
                let idx = (y * width + x) as usize * 4;
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            }
        }
        let compressed = compress_image(&data, width, height, DdsFormat::BC1);
        assert!(!compressed.iter().all(|&b| b == 0));
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
}
