use crate::pipeline::compress;
use crate::pipeline::image::Image;
use std::io::Write;
use thiserror::Error;

/// DDS header is always 128 bytes (4-byte magic + 124-byte header struct)
pub const DDS_HEADER_SIZE: usize = 128;

// DDS header flags (matching pydds.py / Microsoft DDS spec)
const DDSD_CAPS: u32 = 0x0000_0001;
const DDSD_HEIGHT: u32 = 0x0000_0002;
const DDSD_WIDTH: u32 = 0x0000_0004;
const DDSD_PIXELFORMAT: u32 = 0x0000_1000;
const DDSD_MIPMAPCOUNT: u32 = 0x0002_0000;
const DDSD_LINEARSIZE: u32 = 0x0008_0000;

// DDS caps flags
const DDSCAPS_TEXTURE: u32 = 0x0000_1000;
const DDSCAPS_MIPMAP: u32 = 0x0040_0000;
// DDSCAPS_COMPLEX (0x8) — not used; Python pydds.py omits it and X-Plane works without it

// Pixel format flags
const DDPF_FOURCC: u32 = 0x0000_0004;

#[derive(Debug, Error)]
pub enum DdsError {
    #[error("Invalid DDS format")]
    InvalidFormat,
    #[error("Invalid dimensions: width and height must be >= 4 and multiples of 4")]
    InvalidDimensions,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdsFormat {
    BC1, // DXT1 - 8 bytes per 4x4 block, RGB with 1-bit alpha
    BC3, // DXT5 - 16 bytes per 4x4 block, RGBA with interpolated alpha
}

/// Bytes per compressed block for each format
pub fn block_byte_size(format: DdsFormat) -> usize {
    match format {
        DdsFormat::BC1 => 8,
        DdsFormat::BC3 => 16,
    }
}

/// Calculate number of mipmap levels for given dimensions (including level 0).
pub fn mipmap_count(width: u32, height: u32) -> u32 {
    let max_dim = width.max(height);
    if max_dim == 0 {
        return 0;
    }
    // Integer log2: number of times we can halve before reaching 1, plus 1 for level 0
    32 - max_dim.leading_zeros()
}

/// Calculate compressed size for a single mipmap level.
pub fn mipmap_level_size(width: u32, height: u32, format: DdsFormat) -> usize {
    let bbs = block_byte_size(format);
    let w = width.max(1);
    let h = height.max(1);
    let blocks = w.div_ceil(4) * h.div_ceil(4);
    blocks as usize * bbs
}

/// Calculate total mipmap chain size in bytes for a given width, height, and format.
/// This is the canonical implementation — use this everywhere.
pub fn mipmap_chain_size(width: u32, height: u32, format: DdsFormat) -> usize {
    let levels = mipmap_count(width, height);
    let mut size = 0usize;
    let mut w = width;
    let mut h = height;

    for _ in 0..levels {
        size += mipmap_level_size(w, h, format);
        w /= 2;
        h /= 2;
    }

    size
}

/// Calculate total DDS file size (header + mipmap chain) for a 4096×4096 BC3 tile.
pub fn dds_file_size_4096_bc3() -> usize {
    DDS_HEADER_SIZE + mipmap_chain_size(4096, 4096, DdsFormat::BC3)
}

/// Mipmap layout: offset and size for each level within the DDS file.
#[derive(Debug, Clone)]
pub struct MipmapLayout {
    pub levels: Vec<MipmapLevel>,
    pub total_size: usize,
}

/// A single mipmap level's position and dimensions.
#[derive(Debug, Clone)]
pub struct MipmapLevel {
    pub index: u32,
    pub width: u32,
    pub height: u32,
    pub offset: usize,    // byte offset from start of file
    pub data_size: usize, // compressed data size for this level
}

impl MipmapLayout {
    /// Compute the mipmap layout for a DDS file.
    pub fn compute(width: u32, height: u32, format: DdsFormat) -> Self {
        let count = mipmap_count(width, height);
        let mut levels = Vec::with_capacity(count as usize);
        let mut offset = DDS_HEADER_SIZE;
        let mut w = width;
        let mut h = height;

        for i in 0..count {
            let size = mipmap_level_size(w, h, format);
            levels.push(MipmapLevel {
                index: i,
                width: w.max(1),
                height: h.max(1),
                offset,
                data_size: size,
            });
            offset += size;
            w /= 2;
            h /= 2;
        }

        MipmapLayout {
            levels,
            total_size: offset,
        }
    }
}

/// Build a complete fallback DDS file filled with a solid color.
///
/// Creates a valid DDS file (header + all mipmap levels) without needing
/// an RGBA image. Each mipmap level is filled with solid-color compressed
/// blocks using `compress::fallback_bytes()`. This is cheap to generate
/// and suitable for night exclusion or missing-tile placeholders.
pub fn build_fallback_dds(width: u32, height: u32, format: DdsFormat, color: [u8; 3]) -> Vec<u8> {
    let layout = MipmapLayout::compute(width, height, format);
    let mut output = Vec::with_capacity(layout.total_size);

    // Write header using DdsBuilder
    let builder = DdsBuilder::new(width, height, format);
    builder
        .write_header(&mut output)
        .expect("writing DDS header to Vec should never fail");

    // Fill all mipmap levels with solid-color blocks
    let body_size = layout.total_size - DDS_HEADER_SIZE;
    let body = compress::fallback_bytes(body_size, color[0], color[1], color[2], format);
    output.extend_from_slice(&body);

    output
}

/// DDS file builder. Generates a complete DDS file from an RGBA image,
/// including header and all mipmap levels with BC1/BC3 compression.
pub struct DdsBuilder {
    pub width: u32,
    pub height: u32,
    pub format: DdsFormat,
}

impl DdsBuilder {
    pub fn new(width: u32, height: u32, format: DdsFormat) -> Self {
        Self {
            width,
            height,
            format,
        }
    }

    /// Write a DDS header to the writer.
    /// Matches the Python pydds.py DDS header format for X-Plane compatibility.
    pub fn write_header<W: Write>(&self, writer: &mut W) -> Result<(), DdsError> {
        let num_mipmaps = mipmap_count(self.width, self.height);
        let bbs = block_byte_size(self.format) as u32;

        // DDS magic
        writer.write_all(b"DDS ")?;

        // Header size (124 bytes)
        writer.write_all(&124u32.to_le_bytes())?;

        // Flags
        let flags = DDSD_CAPS
            | DDSD_HEIGHT
            | DDSD_WIDTH
            | DDSD_PIXELFORMAT
            | DDSD_MIPMAPCOUNT
            | DDSD_LINEARSIZE;
        writer.write_all(&flags.to_le_bytes())?;

        // Height, Width
        writer.write_all(&self.height.to_le_bytes())?;
        writer.write_all(&self.width.to_le_bytes())?;

        // pitchOrLinearSize: compressed size of top-level mipmap
        let linear_size = ((self.width * self.height) >> 4).max(1) * bbs;
        writer.write_all(&linear_size.to_le_bytes())?;

        // Depth (0 for 2D textures)
        writer.write_all(&0u32.to_le_bytes())?;

        // Mipmap count
        writer.write_all(&num_mipmaps.to_le_bytes())?;

        // Reserved1 (11 DWORDs)
        for _ in 0..11 {
            writer.write_all(&0u32.to_le_bytes())?;
        }

        // Pixel format struct (32 bytes)
        writer.write_all(&32u32.to_le_bytes())?; // pfSize
        writer.write_all(&DDPF_FOURCC.to_le_bytes())?; // pfFlags

        // FourCC
        match self.format {
            DdsFormat::BC1 => writer.write_all(b"DXT1")?,
            DdsFormat::BC3 => writer.write_all(b"DXT5")?,
        }

        // rgbBitCount, R/G/B/A masks (unused for compressed)
        for _ in 0..5 {
            writer.write_all(&0u32.to_le_bytes())?;
        }

        // Caps: TEXTURE | MIPMAP (matching Python pydds.py for X-Plane compatibility)
        let caps = DDSCAPS_TEXTURE | DDSCAPS_MIPMAP;
        writer.write_all(&caps.to_le_bytes())?;

        // Caps2, Caps3, Caps4, Reserved2
        for _ in 0..4 {
            writer.write_all(&0u32.to_le_bytes())?;
        }

        Ok(())
    }

    /// Calculate total file size including header and all mipmap levels.
    pub fn total_size(&self) -> usize {
        DDS_HEADER_SIZE + mipmap_chain_size(self.width, self.height, self.format)
    }

    /// Calculate mipmap chain size (data only, no header).
    pub fn mipmap_chain_size(&self) -> usize {
        mipmap_chain_size(self.width, self.height, self.format)
    }

    /// Build a complete DDS file from an RGBA image.
    /// Generates all mipmap levels using 2x2 box filter reduction
    /// and compresses each level to BC1/BC3.
    pub fn build(&self, image: &Image) -> Result<Vec<u8>, DdsError> {
        if image.width != self.width || image.height != self.height {
            return Err(DdsError::InvalidFormat);
        }

        let layout = MipmapLayout::compute(self.width, self.height, self.format);
        let mut output = Vec::with_capacity(layout.total_size);

        // Write header
        self.write_header(&mut output)?;

        // Compress mipmap level 0 (full size)
        let compressed =
            compress::compress_image(&image.data, image.width, image.height, self.format);
        output.extend_from_slice(&compressed);

        // Generate and compress remaining mipmap levels
        let mut current = image.reduce_half();
        for _level in layout.levels.iter().skip(1) {
            let compressed = compress::compress_image(
                &current.data,
                current.width.max(1),
                current.height.max(1),
                self.format,
            );
            output.extend_from_slice(&compressed);

            if current.width <= 1 && current.height <= 1 {
                // Fill remaining levels with the same 1x1 block
                break;
            }
            current = current.reduce_half();
        }

        // Pad to expected total size if we stopped early (sub-4x4 mipmaps)
        if output.len() < layout.total_size {
            let remaining = layout.total_size - output.len();
            let fill = compress::fallback_bytes(remaining, 0, 0, 0, self.format);
            output.extend_from_slice(&fill);
        }

        Ok(output)
    }

    /// Compress a single mipmap level from an RGBA ImageBuffer (legacy API).
    /// Returns complete DDS file with header + single mip (no mipmap chain).
    pub fn compress(
        &self,
        image: &crate::pipeline::decode::ImageBuffer,
    ) -> Result<Vec<u8>, DdsError> {
        if image.width != self.width || image.height != self.height {
            return Err(DdsError::InvalidFormat);
        }
        if image.channels != 4 {
            return Err(DdsError::InvalidFormat);
        }

        let mut output = Vec::new();
        self.write_header(&mut output)?;

        let compressed =
            compress::compress_image(&image.data, image.width, image.height, self.format);
        output.extend_from_slice(&compressed);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dds_header_size() {
        let builder = DdsBuilder::new(256, 256, DdsFormat::BC1);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        assert_eq!(buf.len(), 128);
    }

    #[test]
    fn test_dds_header_magic() {
        let builder = DdsBuilder::new(256, 256, DdsFormat::BC1);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        assert_eq!(&buf[0..4], b"DDS ");
    }

    #[test]
    fn test_dds_bc1_fourcc() {
        let builder = DdsBuilder::new(256, 256, DdsFormat::BC1);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        assert_eq!(&buf[84..88], b"DXT1");
    }

    #[test]
    fn test_dds_bc3_fourcc() {
        let builder = DdsBuilder::new(256, 256, DdsFormat::BC3);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        assert_eq!(&buf[84..88], b"DXT5");
    }

    #[test]
    fn test_header_mipmap_count() {
        let builder = DdsBuilder::new(256, 256, DdsFormat::BC1);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        // Mipmap count at offset 28
        let mm_count = u32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]);
        assert_eq!(mm_count, 9); // 256→128→64→32→16→8→4→2→1 = 9 levels
    }

    #[test]
    fn test_header_flags_include_mipmaps() {
        let builder = DdsBuilder::new(256, 256, DdsFormat::BC1);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        let flags = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        assert_ne!(flags & DDSD_MIPMAPCOUNT, 0);
        assert_ne!(flags & DDSD_LINEARSIZE, 0);
    }

    #[test]
    fn test_header_caps_match_python() {
        let builder = DdsBuilder::new(256, 256, DdsFormat::BC1);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        // Caps at offset 108 — must match Python pydds.py: TEXTURE | MIPMAP
        let caps = u32::from_le_bytes([buf[108], buf[109], buf[110], buf[111]]);
        assert_ne!(caps & DDSCAPS_TEXTURE, 0);
        assert_ne!(caps & DDSCAPS_MIPMAP, 0);
        assert_eq!(caps, 0x00401000); // Exact match with Python
    }

    #[test]
    fn test_header_linear_size_bc1() {
        let builder = DdsBuilder::new(256, 256, DdsFormat::BC1);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        let linear_size = u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
        // BC1: (256*256/16) * 8 = 32768
        assert_eq!(linear_size, 32768);
    }

    #[test]
    fn test_header_linear_size_bc3() {
        let builder = DdsBuilder::new(256, 256, DdsFormat::BC3);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        let linear_size = u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
        // BC3: (256*256/16) * 16 = 65536
        assert_eq!(linear_size, 65536);
    }

    #[test]
    fn test_dds_dimensions_in_header() {
        let builder = DdsBuilder::new(512, 256, DdsFormat::BC1);
        let mut buf = Vec::new();
        builder.write_header(&mut buf).unwrap();
        let height = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
        let width = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
        assert_eq!(height, 256);
        assert_eq!(width, 512);
    }

    #[test]
    fn test_mipmap_count_256() {
        assert_eq!(mipmap_count(256, 256), 9); // 256..1
    }

    #[test]
    fn test_mipmap_count_4096() {
        assert_eq!(mipmap_count(4096, 4096), 13); // 4096..1
    }

    #[test]
    fn test_mipmap_count_1() {
        assert_eq!(mipmap_count(1, 1), 1);
    }

    #[test]
    fn test_mipmap_layout_offsets() {
        let layout = MipmapLayout::compute(256, 256, DdsFormat::BC1);
        assert_eq!(layout.levels[0].offset, 128); // Right after header
        assert_eq!(layout.levels[0].width, 256);
        assert_eq!(layout.levels[1].width, 128);
        assert_eq!(layout.levels.len(), 9);
        // Each level's offset should equal prev offset + prev data_size
        for i in 1..layout.levels.len() {
            assert_eq!(
                layout.levels[i].offset,
                layout.levels[i - 1].offset + layout.levels[i - 1].data_size
            );
        }
    }

    #[test]
    fn test_mipmap_chain_size_bc1_256() {
        let size = mipmap_chain_size(256, 256, DdsFormat::BC1);
        // Sum of (w/4 * h/4 * 8) for w=256,128,...,1
        // = 32768 + 8192 + 2048 + 512 + 128 + 32 + 8 + 8 + 8 = 43704
        assert!(size > 32768); // At least the top mip
        assert!(size < 50000);
    }

    #[test]
    fn test_build_complete_dds() {
        let img = Image::new_filled(64, 64, [128, 64, 32, 255]);
        let builder = DdsBuilder::new(64, 64, DdsFormat::BC3);
        let dds = builder.build(&img).unwrap();

        // Check header
        assert_eq!(&dds[0..4], b"DDS ");
        assert_eq!(&dds[84..88], b"DXT5");

        // Check total size matches layout
        let layout = MipmapLayout::compute(64, 64, DdsFormat::BC3);
        assert_eq!(dds.len(), layout.total_size);
    }

    #[test]
    fn test_build_dds_4096_bc1_size() {
        // This is the critical size that FUSE getattr returns
        let builder = DdsBuilder::new(4096, 4096, DdsFormat::BC1);
        let total = builder.total_size();
        assert!(total > 10_000_000); // ~11MB
        assert!(total < 12_000_000);
    }

    #[test]
    fn test_build_dds_4096_bc3_size() {
        let builder = DdsBuilder::new(4096, 4096, DdsFormat::BC3);
        let total = builder.total_size();
        assert!(total > 22_000_000); // ~22MB
        assert!(total < 24_000_000);
    }

    #[test]
    fn test_compress_legacy_api() {
        let image = crate::pipeline::decode::ImageBuffer::new(64, 64, 4);
        let builder = DdsBuilder::new(64, 64, DdsFormat::BC1);
        let result = builder.compress(&image);
        assert!(result.is_ok());
        let dds = result.unwrap();
        assert_eq!(&dds[0..4], b"DDS ");
    }

    #[test]
    fn test_compress_dimension_mismatch() {
        let image = crate::pipeline::decode::ImageBuffer::new(64, 64, 4);
        let builder = DdsBuilder::new(128, 128, DdsFormat::BC1);
        assert!(builder.compress(&image).is_err());
    }

    #[test]
    fn test_compress_channel_mismatch() {
        let image = crate::pipeline::decode::ImageBuffer::new(64, 64, 3);
        let builder = DdsBuilder::new(64, 64, DdsFormat::BC1);
        assert!(builder.compress(&image).is_err());
    }

    #[test]
    fn test_build_fallback_dds_bc3() {
        let dds = build_fallback_dds(4096, 4096, DdsFormat::BC3, [20, 25, 15]);
        assert_eq!(&dds[0..4], b"DDS ");
        assert_eq!(&dds[84..88], b"DXT5");
        let layout = MipmapLayout::compute(4096, 4096, DdsFormat::BC3);
        assert_eq!(dds.len(), layout.total_size);
    }

    #[test]
    fn test_build_fallback_dds_bc1() {
        let dds = build_fallback_dds(4096, 4096, DdsFormat::BC1, [20, 25, 15]);
        assert_eq!(&dds[0..4], b"DDS ");
        assert_eq!(&dds[84..88], b"DXT1");
        let layout = MipmapLayout::compute(4096, 4096, DdsFormat::BC1);
        assert_eq!(dds.len(), layout.total_size);
    }

    #[test]
    fn test_build_fallback_dds_small() {
        let dds = build_fallback_dds(64, 64, DdsFormat::BC3, [66, 77, 55]);
        let layout = MipmapLayout::compute(64, 64, DdsFormat::BC3);
        assert_eq!(dds.len(), layout.total_size);
    }
}
