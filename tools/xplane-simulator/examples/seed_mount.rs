//! Example: create a fake mount directory with valid DDS files for testing.
//! Run with: cargo run --example seed_mount -p xplane-simulator

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mount_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/test-mount".to_string());
    let mount_path = Path::new(&mount_path);

    // Create the mount directory
    std::fs::create_dir_all(mount_path.join("textures"))?;

    // Generate a few valid DDS files
    use autoortho_lib::pipeline::dds::{DdsFormat, build_fallback_dds};

    let test_tiles = [
        (3232, 488, 16, "BI"),
        (2458, 3768, 12, "BI"),
        (2511, 3695, 12, "BI"),
    ];

    for (row, col, zoom, maptype) in test_tiles {
        let filename = format!("{}_{}_{}{:02}.dds", row, col, maptype, zoom);
        let path = mount_path.join("textures").join(&filename);
        let data = build_fallback_dds(4096, 4096, DdsFormat::BC3, [100, 150, 200]);
        std::fs::write(&path, &data)?;
        println!("Wrote {} ({} bytes)", path.display(), data.len());
    }

    println!("Done. Mount at: {}", mount_path.display());
    Ok(())
}
