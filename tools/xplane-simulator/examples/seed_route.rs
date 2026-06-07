//! Seed a full route's worth of DDS files for testing.
//! Run with: cargo run --example seed_route -p xplane-simulator -- /tmp/test-mount

use autoortho_lib::pipeline::dds::{DdsFormat, build_fallback_dds};
use autoortho_lib::tiles::coords::TileCoords;
use std::path::Path;
use xplane_simulator::route::Route;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mount_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/test-mount".to_string());
    let mount_path = Path::new(&mount_path);
    let zoom: u32 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(12);

    std::fs::create_dir_all(mount_path.join("textures"))?;

    let route = Route::from_spec("YSSY→YMML").unwrap();
    let waypoints = route.interpolate_waypoints(20);
    let mut seen = std::collections::HashSet::new();
    let mut count = 0;

    for wp in &waypoints {
        let (col, row) = TileCoords::latlng_to_tile(wp.lat, wp.lon, zoom)?;
        for dr in -1..=1i32 {
            for dc in -1..=1i32 {
                let r = row as i32 + dr;
                let c = col as i32 + dc;
                if r < 0 || c < 0 {
                    continue;
                }
                let key = (r as u32, c as u32);
                if seen.insert(key) {
                    let filename = format!("{}_{}_BI{:02}.dds", r, c, zoom);
                    let path = mount_path.join("textures").join(&filename);
                    let data = build_fallback_dds(4096, 4096, DdsFormat::BC3, [100, 150, 200]);
                    std::fs::write(&path, &data)?;
                    count += 1;
                }
            }
        }
    }

    println!("Wrote {} DDS files for route YSSY→YMML at z{}", count, zoom);
    Ok(())
}
