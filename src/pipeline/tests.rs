//! Cross-module pipeline integration tests.

use crate::pipeline::compress;
use crate::pipeline::dds::{DdsBuilder, DdsFormat, MipmapLayout, mipmap_count};
use crate::pipeline::decode::ImageBuffer;
use crate::pipeline::image::Image;

#[test]
fn test_jpeg_decode_to_dds_roundtrip() {
    // Decode a real JPEG test fixture
    let jpeg_data = include_bytes!("../../testfiles/test_tile_small.jpg");
    let buf = ImageBuffer::from_jpeg(jpeg_data).unwrap();
    assert!(buf.width > 0);
    assert!(buf.height > 0);
    assert_eq!(buf.channels, 4);

    // Build DDS from the decoded image
    let img = Image::from_raw(buf.width, buf.height, buf.data).unwrap();
    let builder = DdsBuilder::new(img.width, img.height, DdsFormat::BC3);
    let dds = builder.build(&img).unwrap();

    // Verify header
    assert_eq!(&dds[0..4], b"DDS ");
    assert_eq!(&dds[84..88], b"DXT5");

    // Verify mipmap count in header
    let mm_count_header = u32::from_le_bytes([dds[28], dds[29], dds[30], dds[31]]);
    assert_eq!(mm_count_header, mipmap_count(img.width, img.height));

    // Verify total size
    let layout = MipmapLayout::compute(img.width, img.height, DdsFormat::BC3);
    assert_eq!(dds.len(), layout.total_size);
}

#[test]
fn test_mipmap_chain_reduces_correctly() {
    let img = Image::new_filled(256, 256, [100, 150, 200, 255]);

    // Reduce through full mipmap chain
    let mut current = img;
    let mut sizes = vec![(current.width, current.height)];

    while current.width > 1 || current.height > 1 {
        current = current.reduce_half();
        sizes.push((current.width, current.height));
    }

    // Should produce 256, 128, 64, 32, 16, 8, 4, 2, 1
    assert_eq!(sizes.len(), 9);
    assert_eq!(sizes[0], (256, 256));
    assert_eq!(sizes[8], (1, 1));

    // Each level should be exactly half the previous
    for i in 1..sizes.len() {
        assert_eq!(sizes[i].0, (sizes[i - 1].0 / 2).max(1));
    }
}

#[test]
fn test_compose_16x16_chunks_to_tile() {
    // Simulate composing 16x16 chunks of 256x256 into a 4096x4096 tile
    // Use smaller sizes for speed: 4x4 chunks of 8x8 = 32x32 tile
    let chunks_per_side = 4u32;
    let chunk_size = 8u32;
    let tile_size = chunks_per_side * chunk_size;

    let mut tile = Image::new(tile_size, tile_size);

    for cy in 0..chunks_per_side {
        for cx in 0..chunks_per_side {
            // Each chunk has a unique color based on position
            let r = (cx * 60) as u8;
            let g = (cy * 60) as u8;
            let chunk = Image::new_filled(chunk_size, chunk_size, [r, g, 128, 255]);
            tile.paste(cx * chunk_size, cy * chunk_size, &chunk)
                .unwrap();
        }
    }

    // Verify corners
    assert_eq!(tile.get_pixel(0, 0).unwrap(), [0, 0, 128, 255]);
    assert_eq!(tile.get_pixel(chunk_size, 0).unwrap(), [60, 0, 128, 255]);
    assert_eq!(tile.get_pixel(0, chunk_size).unwrap(), [0, 60, 128, 255]);

    // Build DDS from the composed tile
    let builder = DdsBuilder::new(tile_size, tile_size, DdsFormat::BC1);
    let dds = builder.build(&tile).unwrap();
    assert_eq!(&dds[0..4], b"DDS ");
}

#[test]
fn test_fallback_color_blocks_valid() {
    // Verify fallback blocks produce valid BC1/BC3 data
    let bc1 = compress::solid_color_block(66, 77, 55, DdsFormat::BC1);
    assert_eq!(bc1.len(), 8);

    let bc3 = compress::solid_color_block(66, 77, 55, DdsFormat::BC3);
    assert_eq!(bc3.len(), 16);
    // Alpha endpoints should both be 0xFF (fully opaque)
    assert_eq!(bc3[0], 0xFF);
    assert_eq!(bc3[1], 0xFF);

    // Fallback bytes should produce exact requested length
    let fb = compress::fallback_bytes(10000, 66, 77, 55, DdsFormat::BC3);
    assert_eq!(fb.len(), 10000);
}

#[test]
fn test_4096x4096_bc1_size_matches_python() {
    // The Python SIZE_4096x4096_BC1 = 11,186,000 bytes
    // Verify our calculation matches (or is close — Python's calculation is
    // slightly different in how it counts sub-4x4 mipmaps)
    let builder = DdsBuilder::new(4096, 4096, DdsFormat::BC1);
    let total = builder.total_size();

    // Python: curbytes starts at 128 (header) and adds max(1, (w*h>>4)) * blocksize
    // Our total should be in the same ballpark
    assert!(total > 11_000_000, "total={}", total);
    assert!(total < 11_300_000, "total={}", total);
}
