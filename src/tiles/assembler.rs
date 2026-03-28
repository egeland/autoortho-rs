//! Tile assembler: composes 16×16 JPEG chunks into a complete DDS tile.
//!
//! This is the core pipeline that sits between the tile fetcher (which provides
//! individual 256×256 JPEG chunks) and the FUSE filesystem (which serves DDS files).
//!
//! The pipeline:
//! 1. Collect 16×16 = 256 JPEG chunks for a tile at a given zoom level
//! 2. Decode all JPEGs to RGBA (in parallel with rayon)
//! 3. Compose decoded chunks into one 4096×4096 RGBA image
//! 4. Generate mipmap chain (reduce_half repeatedly)
//! 5. Compress each mipmap level to BC1/BC3
//! 6. Write DDS header + all compressed mipmaps

use crate::pipeline::dds::{DdsBuilder, DdsError, DdsFormat};
use crate::pipeline::decode::{DecodeError, ImageBuffer};
use crate::pipeline::image::Image;
use rayon::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssemblyError {
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),
    #[error("DDS error: {0}")]
    Dds(#[from] DdsError),
    #[error("Wrong chunk count: expected {expected}, got {actual}")]
    WrongChunkCount { expected: usize, actual: usize },
}

/// Configuration for tile assembly.
#[derive(Debug, Clone)]
pub struct AssemblyConfig {
    /// Chunks per side (typically 16 for a 4096×4096 tile)
    pub chunks_per_side: u32,
    /// Size of each chunk in pixels (typically 256)
    pub chunk_size: u32,
    /// Output DDS compression format
    pub format: DdsFormat,
    /// Fallback color for missing/failed chunks (R, G, B)
    pub missing_color: [u8; 3],
    /// Seasonal saturation adjustment (1.0 = no change)
    pub seasonal_saturation: f32,
}

impl Default for AssemblyConfig {
    fn default() -> Self {
        Self {
            chunks_per_side: 16,
            chunk_size: 256,
            format: DdsFormat::BC1,
            missing_color: [66, 77, 55], // Matches Python default
            seasonal_saturation: 1.0,
        }
    }
}

impl AssemblyConfig {
    /// Total tile width/height in pixels
    pub fn tile_size(&self) -> u32 {
        self.chunks_per_side * self.chunk_size
    }

    /// Expected total number of chunks
    pub fn total_chunks(&self) -> usize {
        (self.chunks_per_side * self.chunks_per_side) as usize
    }
}

/// Result of tile assembly, containing the complete DDS file data.
#[derive(Debug)]
pub struct AssemblyResult {
    /// Complete DDS file bytes (header + all mipmaps)
    pub dds_data: Vec<u8>,
    /// Number of chunks successfully decoded
    pub chunks_decoded: u32,
    /// Number of chunks that used fallback color
    pub chunks_failed: u32,
    /// Number of mipmap levels generated
    pub mipmap_count: u32,
}

/// Assemble a complete DDS tile from JPEG chunk data.
///
/// `jpeg_chunks` is a row-major array of optional JPEG data.
/// Index `i` maps to chunk at row `i / chunks_per_side`, col `i % chunks_per_side`.
/// `None` entries use the fallback missing color.
pub fn assemble_tile(
    jpeg_chunks: &[Option<Vec<u8>>],
    config: &AssemblyConfig,
) -> Result<AssemblyResult, AssemblyError> {
    let expected = config.total_chunks();
    if jpeg_chunks.len() != expected {
        return Err(AssemblyError::WrongChunkCount {
            expected,
            actual: jpeg_chunks.len(),
        });
    }

    let tile_size = config.tile_size();

    // Step 1: Decode all JPEGs in parallel using rayon
    let decoded: Vec<Option<Image>> = jpeg_chunks
        .par_iter()
        .map(|chunk_data| {
            chunk_data.as_ref().and_then(|data| {
                ImageBuffer::from_jpeg(data)
                    .ok()
                    .and_then(|buf| Image::from_raw(buf.width, buf.height, buf.data).ok())
            })
        })
        .collect();

    // Step 2: Compose chunks into full tile image
    let [mr, mg, mb] = config.missing_color;
    let fallback_chunk = Image::new_filled(config.chunk_size, config.chunk_size, [mr, mg, mb, 255]);

    let mut tile = Image::new(tile_size, tile_size);
    let mut chunks_decoded = 0u32;
    let mut chunks_failed = 0u32;

    for (i, maybe_img) in decoded.iter().enumerate() {
        let row = (i as u32) / config.chunks_per_side;
        let col = (i as u32) % config.chunks_per_side;
        let x = col * config.chunk_size;
        let y = row * config.chunk_size;

        match maybe_img {
            Some(img) if img.width == config.chunk_size && img.height == config.chunk_size => {
                tile.paste(x, y, img).ok();
                chunks_decoded += 1;
            }
            _ => {
                tile.paste(x, y, &fallback_chunk).ok();
                chunks_failed += 1;
            }
        }
    }

    // Step 2b: Apply seasonal saturation if enabled
    if config.seasonal_saturation != 1.0 {
        tile.apply_saturation(config.seasonal_saturation);
    }

    // Step 3: Build complete DDS with mipmap chain
    let builder = DdsBuilder::new(tile_size, tile_size, config.format);
    let dds_data = builder.build(&tile)?;
    let mipmap_count = crate::pipeline::dds::mipmap_count(tile_size, tile_size);

    Ok(AssemblyResult {
        dds_data,
        chunks_decoded,
        chunks_failed,
        mipmap_count,
    })
}

/// Assemble a DDS tile from pre-decoded RGBA images (skips JPEG decode step).
/// Useful when chunks are already decoded or come from fallback resolution.
pub fn assemble_from_images(
    images: &[Option<&Image>],
    config: &AssemblyConfig,
) -> Result<AssemblyResult, AssemblyError> {
    let expected = config.total_chunks();
    if images.len() != expected {
        return Err(AssemblyError::WrongChunkCount {
            expected,
            actual: images.len(),
        });
    }

    let tile_size = config.tile_size();
    let [mr, mg, mb] = config.missing_color;
    let fallback_chunk = Image::new_filled(config.chunk_size, config.chunk_size, [mr, mg, mb, 255]);

    let mut tile = Image::new(tile_size, tile_size);
    let mut chunks_decoded = 0u32;
    let mut chunks_failed = 0u32;

    for (i, maybe_img) in images.iter().enumerate() {
        let row = (i as u32) / config.chunks_per_side;
        let col = (i as u32) % config.chunks_per_side;
        let x = col * config.chunk_size;
        let y = row * config.chunk_size;

        match maybe_img {
            Some(img) if img.width == config.chunk_size && img.height == config.chunk_size => {
                tile.paste(x, y, img).ok();
                chunks_decoded += 1;
            }
            _ => {
                tile.paste(x, y, &fallback_chunk).ok();
                chunks_failed += 1;
            }
        }
    }

    let builder = DdsBuilder::new(tile_size, tile_size, config.format);
    let dds_data = builder.build(&tile)?;
    let mipmap_count = crate::pipeline::dds::mipmap_count(tile_size, tile_size);

    Ok(AssemblyResult {
        dds_data,
        chunks_decoded,
        chunks_failed,
        mipmap_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_jpeg() -> Vec<u8> {
        // Create a valid 8x8 JPEG by encoding a small image
        use image::{DynamicImage, RgbaImage};
        let img = RgbaImage::from_pixel(256, 256, image::Rgba([128, 64, 32, 255]));
        let dyn_img = DynamicImage::ImageRgba8(img);
        let mut buf = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buf);
        dyn_img
            .write_to(&mut cursor, image::ImageFormat::Jpeg)
            .unwrap();
        buf
    }

    #[test]
    fn test_assembly_config_defaults() {
        let config = AssemblyConfig::default();
        assert_eq!(config.tile_size(), 4096);
        assert_eq!(config.total_chunks(), 256);
    }

    #[test]
    fn test_assemble_all_missing() {
        // All chunks missing → full fallback color tile
        let config = AssemblyConfig {
            chunks_per_side: 4,
            chunk_size: 8,
            format: DdsFormat::BC1,
            missing_color: [66, 77, 55],
            seasonal_saturation: 1.0,
        };

        let chunks: Vec<Option<Vec<u8>>> = vec![None; 16];
        let result = assemble_tile(&chunks, &config).unwrap();

        assert_eq!(result.chunks_decoded, 0);
        assert_eq!(result.chunks_failed, 16);
        assert_eq!(&result.dds_data[0..4], b"DDS ");
    }

    #[test]
    fn test_assemble_wrong_chunk_count() {
        let config = AssemblyConfig::default();
        let chunks: Vec<Option<Vec<u8>>> = vec![None; 10]; // Wrong count
        let result = assemble_tile(&chunks, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_assemble_with_real_jpegs() {
        let jpeg = make_minimal_jpeg();

        let config = AssemblyConfig {
            chunks_per_side: 2,
            chunk_size: 256,
            format: DdsFormat::BC3,
            missing_color: [66, 77, 55],
            seasonal_saturation: 1.0,
        };

        // 2x2 grid: 3 valid + 1 missing
        let chunks: Vec<Option<Vec<u8>>> = vec![
            Some(jpeg.clone()),
            Some(jpeg.clone()),
            Some(jpeg.clone()),
            None,
        ];

        let result = assemble_tile(&chunks, &config).unwrap();
        assert_eq!(result.chunks_decoded, 3);
        assert_eq!(result.chunks_failed, 1);
        assert_eq!(&result.dds_data[0..4], b"DDS ");
        assert_eq!(&result.dds_data[84..88], b"DXT5");
        assert!(result.mipmap_count > 1);
    }

    #[test]
    fn test_assemble_from_images_basic() {
        let config = AssemblyConfig {
            chunks_per_side: 2,
            chunk_size: 8,
            format: DdsFormat::BC1,
            missing_color: [0, 0, 0],
            seasonal_saturation: 1.0,
        };

        let img1 = Image::new_filled(8, 8, [255, 0, 0, 255]);
        let img2 = Image::new_filled(8, 8, [0, 255, 0, 255]);

        let images: Vec<Option<&Image>> = vec![Some(&img1), Some(&img2), None, Some(&img1)];

        let result = assemble_from_images(&images, &config).unwrap();
        assert_eq!(result.chunks_decoded, 3);
        assert_eq!(result.chunks_failed, 1);
    }

    #[test]
    fn test_assemble_result_dds_size() {
        let config = AssemblyConfig {
            chunks_per_side: 4,
            chunk_size: 8,
            format: DdsFormat::BC1,
            missing_color: [0, 0, 0],
            seasonal_saturation: 1.0,
        };

        let chunks: Vec<Option<Vec<u8>>> = vec![None; 16];
        let result = assemble_tile(&chunks, &config).unwrap();

        // Verify DDS total size matches layout
        let layout = crate::pipeline::dds::MipmapLayout::compute(32, 32, DdsFormat::BC1);
        assert_eq!(result.dds_data.len(), layout.total_size);
    }
}
