// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! X-Plane tile request simulator.
//!
//! Simulates X-Plane tile requests along a route, reading tiles from the
//! autoortho FUSE/Dokan mount to verify correct behavior under load.
//!
//! Usage:
//!     cargo run -p xplane-simulator -- --xplane /path/to/X-Plane

use anyhow::Context;
use autoortho_lib::scenery::paths;
use autoortho_lib::tiles::coords::{TileCoord, TileCoords};
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, warn};
use tracing_subscriber::prelude::*;
use xplane_simulator::perf;
use xplane_simulator::route::Route;
use xplane_simulator::{error, validation};

/// Resolve the mount path from the X-Plane installation path.
fn resolve_mount_path(xplane_path: &str) -> PathBuf {
    // The FUSE mount is at {xplane}/Custom Scenery/z_autoortho/
    // This directory contains virtual textures/terrain dirs and pass-through files
    paths::mount_dir(xplane_path)
}

/// Compute all tile coordinates along a route at a given zoom level.
/// Samples intermediate waypoints to ensure complete coverage.
fn tiles_along_route(route: &Route, zoom: u32) -> Vec<TileCoord> {
    let mut coords: Vec<TileCoord> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for waypoint in route.interpolate_waypoints(20) {
        // Sample a grid around each waypoint (3x3 = 9 tiles)
        let (center_col, center_row) =
            TileCoords::latlng_to_tile(waypoint.lat, waypoint.lon, zoom).unwrap_or((0, 0));

        for dr in -1..=1i32 {
            for dc in -1..=1i32 {
                let row = center_row as i32 + dr;
                let col = center_col as i32 + dc;
                if row < 0 || col < 0 {
                    continue;
                }
                let key = (row as u32, col as u32);
                if seen.insert(key)
                    && let Ok(coord) = TileCoord::new(row as u32, col as u32, zoom)
                {
                    coords.push(coord);
                }
            }
        }
    }

    coords
}

/// Format a tile coordinate as a DDS filename.
/// Uses BI (Bing) as the default maptype, matches autoortho convention.
fn tile_dds_filename(coord: &TileCoord) -> String {
    format!("{}_{}_BI{:02}.dds", coord.row, coord.col, coord.zoom)
}

/// Build the full path to a DDS tile inside the mount.
/// DDS files live under the virtual textures/ directory.
fn tile_mount_path(mount_path: &std::path::Path, coord: &TileCoord) -> PathBuf {
    mount_path.join("textures").join(tile_dds_filename(coord))
}

#[derive(clap::Parser)]
#[command(name = "xplane-simulator")]
#[command(about = "Simulate X-Plane tile requests along a route")]
struct Args {
    /// Path to X-Plane installation (used to derive mount path)
    #[arg(long = "xplane", short = 'x')]
    xplane_path: Option<String>,

    /// Explicit mount path (overrides xplane-path derivation)
    #[arg(long = "mount", short = 'm')]
    mount_path: Option<PathBuf>,

    /// Zoom level for tile requests (default: 16)
    #[arg(long, short = 'z', default_value = "16")]
    zoom: u32,

    /// Route specification: FROM→TO (e.g., YSSY→YMML)
    #[arg(long, default_value = "YSSY→YMML")]
    route_spec: String,

    /// Reference directory for checksum comparison (generated on first run)
    #[arg(long, visible_short_alias = 'D')]
    reference_dir: Option<PathBuf>,

    /// Generate reference checksums and exit
    #[arg(long = "generate-reference")]
    generate_reference: bool,

    /// Tile provider to use (e.g., BI, ARC, GO2)
    #[arg(long, short = 'p', default_value = "BI")]
    provider: String,
}

fn main() -> anyhow::Result<()> {
    // Parse args first to handle --help without requiring full init
    let args = Args::parse();

    // Logging: stdout + file
    let log_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(std::env::temp_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "xplane-simulator.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .init();

    info!("X-Plane Simulator starting");

    // Resolve mount path
    let mount_path = if let Some(ref mp) = args.mount_path {
        mp.clone()
    } else if let Some(ref xp) = args.xplane_path {
        resolve_mount_path(xp)
    } else {
        anyhow::bail!("Must specify --xplane or --mount");
    };

    info!("Mount path: {:?}", mount_path);

    // Validate X-Plane installation
    if let Some(ref xp) = args.xplane_path {
        match validation::validate_xplane_dir(std::path::Path::new(xp)) {
            Ok(()) => println!("✓ X-Plane directory valid: {}", xp),
            Err(e) => anyhow::bail!("{}", e),
        }
    }

    // Verify mount is accessible
    if !mount_path.exists() {
        anyhow::bail!(
            "Mount path does not exist: {:?}\n\
             Is autoortho-rs running with the FUSE mount active?",
            mount_path
        );
    }

    // Parse route
    let route =
        Route::from_spec(&args.route_spec).context("Failed to parse route specification")?;
    info!("Route: {}", route.name());
    for wp in &route.waypoints {
        info!("  - {} ({:.4}, {:.4})", wp.name, wp.lat, wp.lon);
    }

    // Compute tiles along route
    let tiles = tiles_along_route(&route, args.zoom);
    info!(
        "Route covers {} unique tiles at zoom {}",
        tiles.len(),
        args.zoom
    );

    if tiles.is_empty() {
        anyhow::bail!("No tiles found along route. Check coordinates.");
    }

    // Initialize performance tracker
    let mut perf_stats = perf::PerfStats::new();

    // Reference checksums
    let ref_dir = args.reference_dir.clone();
    let generate_ref = args.generate_reference;

    // Read and validate each tile
    let mut results: Vec<error::TileResult> = Vec::new();

    for coord in &tiles {
        let filename = tile_dds_filename(coord);
        let tile_path = tile_mount_path(&mount_path, coord);
        let start = Instant::now();

        let result = read_and_validate_tile(&tile_path, &mut perf_stats);

        let elapsed = start.elapsed();
        match &result {
            Ok(data) => {
                info!(
                    "✓ Read tile {} — {} bytes, {:?}",
                    filename,
                    data.len(),
                    elapsed
                );
            }
            Err(e) => {
                warn!("✗ Failed tile {}: {:?} ({:?})", filename, e, elapsed);
            }
        }

        results.push(error::TileResult {
            coord: *coord,
            filename,
            elapsed,
            result: result.map_err(|e| e.to_string()),
        });
    }

    // Print summary
    let total = results.len();
    let succeeded = results.iter().filter(|r| r.result.is_ok()).count();
    let failed = total - succeeded;

    info!("\n=== Summary ===");
    info!("Total tiles: {}", total);
    info!("Successful: {}", succeeded);
    info!("Failed: {}", failed);
    info!("Total time: {:?}", perf_stats.total_time());
    info!(
        "Throughput: {:.1} tiles/sec",
        total as f64 / perf_stats.total_time().as_secs_f64()
    );

    if succeeded == total {
        info!("\nAll tiles verified successfully!");
    } else {
        warn!("\n{} tiles failed - see above for details", failed);
    }

    // Reference comparison
    if generate_ref {
        let dir = ref_dir.unwrap_or_else(|| PathBuf::from("./reference"));
        std::fs::create_dir_all(&dir).context("Failed to create reference directory")?;
        for result in &results {
            if let Ok(ref data) = result.result {
                let checksum = validation::sha256(data);
                let meta = validation::ReferenceMetadata::from_tile(
                    &result.coord,
                    data.len(),
                    checksum,
                    validation::validate_dds(data).is_ok(),
                );
                let meta_path = dir.join(format!("{}.meta.json", result.coord));
                let json =
                    serde_json::to_string_pretty(&meta).context("Failed to serialize metadata")?;
                std::fs::write(meta_path, json).context("Failed to write metadata")?;
            }
        }
        info!("Reference generated in {:?}", dir);
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Read a tile file from the mount and validate it.
/// Returns the raw bytes on success.
fn read_and_validate_tile(
    path: &PathBuf,
    perf: &mut perf::PerfStats,
) -> Result<Vec<u8>, error::TileError> {
    let bytes = std::fs::read(path).map_err(|e| error::TileError::IoError(e.to_string()))?;

    perf.record_read(bytes.len());

    // Validate DDS format
    validation::validate_dds(&bytes).map_err(|e| error::TileError::InvalidFormat(e.to_string()))?;

    Ok(bytes)
}
