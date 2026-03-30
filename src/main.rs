// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use autoortho_lib::config::AutoOrthoConfig;
use autoortho_lib::dynamic_zoom::DynamicZoom;
use autoortho_lib::fuse::filesystem::DdsFileSystem;
use autoortho_lib::pipeline::cache::DdsCache;
use autoortho_lib::stats::StatsStore;
use autoortho_lib::tiles::fetcher::TileFetcher;
use autoortho_lib::tiles::prefetch::{RoutePrefetchConfig, SpatialPrefetcher};
use autoortho_lib::tiles::provider::ProviderFactory;
use autoortho_lib::xplane::dataref::DatarefTracker;
use log::{info, warn};
use parking_lot::Mutex;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(not(windows))]
use tokio::sync::broadcast;

fn main() -> Result<(), Box<dyn Error>> {
    // Default to info level, RUST_LOG=debug for verbose output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--reset-window") {
        let mut config = autoortho_lib::config::AutoOrthoConfig::load();
        config.reset_window_position();
        config.save().map_err(|e| e.to_string())?;
        println!("Window position reset to centered. Launch with --gui to see the change.");
        return Ok(());
    }

    if args.iter().any(|a| a == "--gui") {
        info!("Launching desktop UI");
        let runtime = autoortho_lib::create_runtime();
        autoortho_lib::ui::run(runtime).map_err(|e| format!("GUI error: {}", e))?;
        return Ok(());
    }

    if let Some(pos) = args.iter().position(|a| a == "--test-tile") {
        let provider = args.get(pos + 1).map(|s| s.as_str()).unwrap_or("ARC");
        let rt = tokio::runtime::Runtime::new()?;
        return rt.block_on(test_tile_generation(provider));
    }

    if let Some(pos) = args.iter().position(|a| a == "--mount") {
        #[cfg(not(windows))]
        {
            let config = autoortho_lib::config::AutoOrthoConfig::load();
            let config_mount_dir = config.mount_dir().to_string_lossy().into_owned();
            let mountpoint = args
                .get(pos + 1)
                .map(|s| s.as_str())
                .unwrap_or(&config_mount_dir);
            let rt = tokio::runtime::Runtime::new()?;
            return rt.block_on(run_with_mount(mountpoint));
        }
        #[cfg(windows)]
        {
            eprintln!("FUSE mounting is not supported on Windows. Starting server without mount.");
        }
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_server())
}

/// Fetch a real satellite tile and write it as a DDS file for inspection.
async fn test_tile_generation(provider_name: &str) -> Result<(), Box<dyn Error>> {
    use autoortho_lib::pipeline::dds::DdsFormat;
    use autoortho_lib::pipeline::decode::ImageBuffer;
    use autoortho_lib::pipeline::image::Image;
    use autoortho_lib::tiles::assembler::{AssemblyConfig, assemble_tile};

    println!("=== AutoOrtho Tile Generation Test ===");
    println!();

    let provider = ProviderFactory::create(provider_name).expect("Unknown tile provider");
    println!("Provider: {} ({})", provider.name(), provider_name);

    let fetcher = Arc::new(TileFetcher::new(provider, provider_name));

    // Sydney Opera House area
    let zoom = 14u32;
    let lat = -33.86_f64;
    let lon = 151.21_f64;

    // Convert to fractional tile coordinates, center the 2x2 grid
    let n = 2_f64.powi(zoom as i32);
    let tile_x = (lon + 180.0) / 360.0 * n;
    let lat_rad = lat.to_radians();
    let tile_y =
        (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n;
    let base_col = (tile_x - 1.0).floor().max(0.0) as u32;
    let base_row = (tile_y - 1.0).floor().max(0.0) as u32;

    println!("Target: Sydney, Australia (lat {}, lon {})", lat, lon);
    println!(
        "Tile coords: col={}, row={}, zoom={}",
        base_col, base_row, zoom
    );
    println!();

    // Fetch a small grid (2x2 = 4 chunks) centered on target
    let chunks_per_side = 2u32;
    let total = (chunks_per_side * chunks_per_side) as usize;

    println!(
        "Fetching {}x{} = {} chunks...",
        chunks_per_side, chunks_per_side, total
    );

    let mut jpeg_chunks: Vec<Option<Vec<u8>>> = Vec::with_capacity(total);
    let mut fetched = 0u32;
    let mut failed = 0u32;

    for dy in 0..chunks_per_side {
        for dx in 0..chunks_per_side {
            let c = base_col + dx;
            let r = base_row + dy;
            print!("  Chunk ({}, {})...", c, r);

            match fetcher.get_chunk_data(r, c, provider_name, zoom).await {
                Ok(Some(data)) => {
                    println!(" {} bytes", data.len());
                    fetched += 1;
                    jpeg_chunks.push(Some(data));
                }
                Ok(None) => {
                    println!(" no data");
                    failed += 1;
                    jpeg_chunks.push(None);
                }
                Err(e) => {
                    println!(" error: {}", e);
                    failed += 1;
                    jpeg_chunks.push(None);
                }
            }
        }
    }

    println!();
    println!("Fetched: {}/{}, Failed: {}", fetched, total, failed);

    // Save individual JPEG chunks for inspection
    for (i, chunk) in jpeg_chunks.iter().enumerate() {
        if let Some(data) = chunk {
            let path = format!("test_chunk_{}.jpg", i);
            std::fs::write(&path, data)?;
            println!("  Saved {}", path);
        }
    }

    // Compose chunks into a single RGBA image and save as PNG
    println!();
    println!("Composing preview PNG...");
    let tile_px = chunks_per_side * 256;
    let mut tile_image = Image::new(tile_px, tile_px);
    let fallback = Image::new_filled(256, 256, [66, 77, 55, 255]);

    for (i, chunk) in jpeg_chunks.iter().enumerate() {
        let cx = (i as u32 % chunks_per_side) * 256;
        let cy = (i as u32 / chunks_per_side) * 256;

        match chunk {
            Some(data) => {
                if let Ok(buf) = ImageBuffer::from_jpeg(data)
                    && let Ok(img) = Image::from_raw(buf.width, buf.height, buf.data)
                {
                    tile_image.paste(cx, cy, &img).ok();
                    continue;
                }
                tile_image.paste(cx, cy, &fallback).ok();
            }
            None => {
                tile_image.paste(cx, cy, &fallback).ok();
            }
        }
    }

    // Convert our Image to the `image` crate format and save as PNG
    let png_img =
        image::RgbaImage::from_raw(tile_image.width, tile_image.height, tile_image.data.clone())
            .expect("image dimensions match data length");
    png_img.save("test_output.png")?;
    println!("  Saved test_output.png ({}x{} pixels)", tile_px, tile_px);

    // Assemble into DDS
    println!();
    println!("Assembling DDS tile...");
    let assembly_config = AssemblyConfig {
        chunks_per_side,
        chunk_size: 256,
        format: DdsFormat::BC3,
        missing_color: [66, 77, 55],
        seasonal_saturation: 1.0,
    };

    let start = std::time::Instant::now();
    let result = assemble_tile(&jpeg_chunks, &assembly_config)?;
    let elapsed = start.elapsed();

    println!(
        "  DDS size: {} bytes ({:.1} MB)",
        result.dds_data.len(),
        result.dds_data.len() as f64 / 1_048_576.0
    );
    println!("  Chunks decoded: {}/{}", result.chunks_decoded, total);
    println!("  Chunks fallback: {}", result.chunks_failed);
    println!("  Mipmap levels: {}", result.mipmap_count);
    println!("  Assembly time: {:?}", elapsed);

    // Verify DDS header
    assert_eq!(&result.dds_data[0..4], b"DDS ", "Invalid DDS magic");
    assert_eq!(&result.dds_data[84..88], b"DXT5", "Expected BC3/DXT5");
    println!("  Header: valid DDS/DXT5");

    // Write to file for inspection
    let output_path = "test_output.dds";
    std::fs::write(output_path, &result.dds_data)?;
    println!();
    println!("Written to: {}", output_path);
    println!("You can inspect this with a DDS viewer or load it in X-Plane.");
    println!();
    println!("=== Test complete ===");

    Ok(())
}

/// Run with FUSE mount — serves DDS tiles at the mount point.
/// Only available on Linux and macOS.
#[cfg(not(windows))]
async fn run_with_mount(mountpoint: &str) -> Result<(), Box<dyn Error>> {
    use autoortho_lib::webui::custommap::CustomMapStore;
    use std::path::Path;

    info!(
        "AutoOrtho Rust v{} starting with FUSE mount",
        env!("CARGO_PKG_VERSION")
    );

    let config = AutoOrthoConfig::default();
    let provider = ProviderFactory::create(&config.tile_provider).expect("Unknown tile provider");
    info!("Provider: {} ({})", provider.name(), config.tile_provider);

    let dynamic_zoom = DynamicZoom::new(config.zoom_rules.clone(), &config.tile_provider);
    if config.enable_dynamic_zoom {
        info!(
            "Dynamic zoom enabled with {} rules",
            dynamic_zoom.zoom_rules().len()
        );
    }

    let custom_map_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("autoortho")
        .join("custom_map.json");
    let custom_map = CustomMapStore::load(custom_map_path);
    info!("Custom map: {} cells defined", custom_map.get_cells().len());

    let chunk_cache_entries = config.chunk_memory_cache_entries();
    let dds_cache_entries = config.dds_memory_cache_entries();
    info!(
        "Memory cache: {} chunk entries, {} DDS tile entries",
        chunk_cache_entries, dds_cache_entries
    );

    let fetcher = Arc::new(TileFetcher::with_cache_size(
        chunk_cache_entries,
        &config.tile_provider,
    ));

    let dds_cache = if config.enable_dds_cache {
        let cache_dir = PathBuf::from(&config.cache_dir).join("dds");
        match DdsCache::open(cache_dir, config.dds_cache_size_mb * 1024 * 1024) {
            Ok(cache) => {
                info!(
                    "DDS cache: {} entries, {} MB",
                    cache.entry_count(),
                    cache.size_bytes() / (1024 * 1024)
                );
                Some(Arc::new(Mutex::new(cache)))
            }
            Err(e) => {
                warn!(
                    "Failed to open DDS cache: {}. Running without disk cache.",
                    e
                );
                None
            }
        }
    } else {
        info!("DDS disk cache disabled");
        None
    };

    let fs = if let Some(dc) = dds_cache {
        Arc::new(DdsFileSystem::with_disk_cache_and_custom_map(
            fetcher.clone(),
            dc,
            custom_map,
            &config.tile_provider,
            dds_cache_entries,
        ))
    } else {
        Arc::new(DdsFileSystem::new_with_custom_map(
            fetcher.clone(),
            custom_map,
            &config.tile_provider,
            dds_cache_entries,
        ))
    };

    // Start web server
    let stats = Arc::new(StatsStore::new());
    let tracker = Arc::new(DatarefTracker::new());
    let web_config = Arc::new(parking_lot::RwLock::new(config.clone()));

    // Shutdown channel for graceful termination of background tasks
    let (shutdown_tx, _) = broadcast::channel(1);

    let addr = autoortho_lib::webui::start_server(
        5847,
        stats.clone(),
        tracker.clone(),
        web_config.clone(),
    )
    .await
    .map_err(|e| format!("Web server error: {}", e))?;
    info!("Web UI at http://{}", addr);

    // Night exclusion: poll sun_pitch from the dataref tracker and update the
    // filesystem's night exclusion flag accordingly.
    if config.enable_night_exclusion {
        let night_flag = fs.night_exclusion_flag();
        let tracker_for_night = tracker.clone();
        let night_threshold = config.night_threshold;
        let day_threshold = config.day_threshold;
        info!(
            "Night exclusion enabled (night <= {}°, day >= {}°)",
            night_threshold, day_threshold
        );
        tokio::spawn(async move {
            use autoortho_lib::time_exclusion::TimeExclusion;
            let te = TimeExclusion::new(night_threshold, day_threshold);
            loop {
                let data = tracker_for_night.get_flight_data();
                if data.data_valid {
                    let is_night = te.is_night(data.sun_pitch);
                    night_flag.store(is_night, std::sync::atomic::Ordering::Relaxed);
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    } else {
        info!("Night exclusion disabled");
    }

    // SimBrief route prefetch: if user_id is configured, fetch flight plan and start prefetch
    const PREFETCH_SPACING_NM: f64 = 10.0;
    const PREFETCH_MAX_LOOKAHEAD_NM: f32 = 99999.0;
    const PREFETCH_POLL_INTERVAL_SECS: u64 = 30;

    if !config.simbrief_user_id.is_empty() {
        info!("Fetching SimBrief flight plan for route prefetching...");
        let simbrief_user_id = config.simbrief_user_id.clone();
        match tokio::runtime::Handle::current().block_on(
            autoortho_lib::xplane::simbrief::fetch_flight_plan(&simbrief_user_id),
        ) {
            Ok(plan) => {
                info!(
                    "SimBrief route loaded: {} -> {}",
                    plan.origin, plan.destination
                );
                let fetcher_for_prefetch = fetcher.clone();
                let tracker_for_prefetch = tracker.clone();
                let config_for_prefetch = config.clone();
                let mut shutdown_rx = shutdown_tx.subscribe();

                tokio::spawn(async move {
                    let mut prefetcher = SpatialPrefetcher::new();
                    let route_config = RoutePrefetchConfig {
                        percent_ahead: config_for_prefetch.prefetch_route_percent,
                        waypoint_radius_nm: config_for_prefetch.route_prefetch_radius_nm as f64,
                        airport_radius_nm: config_for_prefetch.airport_radius_nm as f64,
                        include_airports: config_for_prefetch.prefetch_airports,
                        zoom: config_for_prefetch.max_zoom,
                    };

                    let dynamic_zoom_for_prefetch = DynamicZoom::new(
                        config_for_prefetch.zoom_rules.clone(),
                        &config_for_prefetch.tile_provider,
                    );

                    loop {
                        tokio::select! {
                            _ = shutdown_rx.recv() => {
                                info!("Route prefetch shutting down");
                                break;
                            }
                            _ = tokio::time::sleep(std::time::Duration::from_secs(PREFETCH_POLL_INTERVAL_SECS)) => {
                                // Continue with prefetch logic
                            }
                        };

                        let flight_data = tracker_for_prefetch.get_flight_data();

                        if flight_data.data_valid {
                            let lat = flight_data.lat;
                            let lon = flight_data.lon;

                            // Check if on route
                            if plan.is_on_route(
                                lat,
                                lon,
                                config_for_prefetch.route_consideration_radius_nm as f64,
                            ) {
                                // Get prefetch points along route
                                let points = plan.get_prefetch_points(
                                    lat,
                                    lon,
                                    PREFETCH_SPACING_NM,
                                    PREFETCH_MAX_LOOKAHEAD_NM,
                                );

                                if !points.is_empty() {
                                    // Calculate total route distance
                                    let route_distance_nm = points
                                        .last()
                                        .map(|p| p.distance_along_route_nm)
                                        .unwrap_or(0.0);

                                    // Update prefetch queue
                                    prefetcher.prefetch_route(
                                        &points,
                                        route_distance_nm,
                                        route_config,
                                    );

                                    // Trigger fetches for queued tiles
                                    while let Some((row, col)) = prefetcher.next_tile() {
                                        let zoom = if config_for_prefetch.enable_dynamic_zoom {
                                            if config_for_prefetch.use_simbrief_altitude
                                                && !points.is_empty()
                                            {
                                                let mut closest_dist = f64::MAX;
                                                let mut best_alt_agl = 0.0f32;
                                                for point in &points {
                                                    let dist = ((point.lat - lat).powi(2)
                                                        + (point.lon - lon).powi(2))
                                                    .sqrt();
                                                    if dist < closest_dist {
                                                        closest_dist = dist;
                                                        best_alt_agl = point.altitude_agl_ft();
                                                    }
                                                }
                                                dynamic_zoom_for_prefetch
                                                    .zoom_for_altitude_agl(best_alt_agl)
                                            } else {
                                                let alt_agl_ft = flight_data.alt_agl_m * 3.28084;
                                                dynamic_zoom_for_prefetch
                                                    .zoom_for_altitude_agl(alt_agl_ft)
                                            }
                                        } else {
                                            config_for_prefetch.max_zoom
                                        };

                                        if let Err(e) = fetcher_for_prefetch
                                            .get_chunk_data(
                                                row,
                                                col,
                                                &config_for_prefetch.tile_provider,
                                                zoom,
                                            )
                                            .await
                                        {
                                            log::debug!(
                                                "Prefetch failed for tile ({}, {}) at zoom {}: {}",
                                                row,
                                                col,
                                                zoom,
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
            Err(e) => {
                warn!("Failed to fetch SimBrief flight plan: {}", e);
            }
        }
    }

    // Create mount point if it doesn't exist
    let mount_path = Path::new(mountpoint);
    std::fs::create_dir_all(mount_path)?;

    info!("Mounting FUSE at: {}", mountpoint);
    info!("Press Ctrl+C to unmount and shut down.");
    info!(
        "Or stat {}/.poison to trigger graceful shutdown.",
        mountpoint
    );

    // Listen for Ctrl+C to trigger graceful shutdown
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Ctrl+C received, initiating shutdown...");
                let _ = shutdown_tx_clone.send(());
            }
            Err(e) => {
                warn!("Failed to listen for Ctrl+C: {}", e);
            }
        }
    });

    // Mount blocks until unmounted
    let runtime_handle = tokio::runtime::Handle::current();
    let fs_clone = fs.clone();
    let mount_path = mount_path.to_path_buf();

    // Run FUSE mount in a blocking thread (it blocks until unmounted)
    tokio::task::spawn_blocking::<_, Result<(), String>>(move || {
        #[cfg(not(windows))]
        {
            autoortho_lib::fuse::mount::mount(fs_clone, &mount_path, runtime_handle)
                .map_err(|e| e.to_string())
        }
        #[cfg(windows)]
        {
            autoortho_lib::fuse::mount_win::mount(fs_clone, &mount_path, runtime_handle)
                .map_err(|e| e.to_string())
        }
    })
    .await?
    .map_err(|e| format!("FUSE mount error: {}", e))?;

    info!("FUSE unmounted. Shutting down.");
    Ok(())
}

async fn run_server() -> Result<(), Box<dyn Error>> {
    use autoortho_lib::webui::custommap::CustomMapStore;

    info!("AutoOrtho Rust v{} starting", env!("CARGO_PKG_VERSION"));

    let config = AutoOrthoConfig::default();
    info!("Using tile provider: {}", config.tile_provider);
    info!("Zoom levels: {} - {}", config.min_zoom, config.max_zoom);

    let provider = ProviderFactory::create(&config.tile_provider).expect("Unknown tile provider");
    info!("Initialized {} provider", provider.name());

    let custom_map_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("autoortho")
        .join("custom_map.json");
    let custom_map = CustomMapStore::load(custom_map_path);
    info!("Custom map: {} cells defined", custom_map.get_cells().len());

    let chunk_cache_entries = config.chunk_memory_cache_entries();
    let dds_cache_entries = config.dds_memory_cache_entries();
    info!(
        "Memory cache: {} chunk entries, {} DDS tile entries",
        chunk_cache_entries, dds_cache_entries
    );

    let fetcher = Arc::new(TileFetcher::with_cache_size(
        chunk_cache_entries,
        &config.tile_provider,
    ));

    let dds_cache = if config.enable_dds_cache {
        let cache_dir = PathBuf::from(&config.cache_dir).join("dds");
        match DdsCache::open(cache_dir, config.dds_cache_size_mb * 1024 * 1024) {
            Ok(cache) => {
                info!(
                    "DDS cache: {} entries, {} MB",
                    cache.entry_count(),
                    cache.size_bytes() / (1024 * 1024)
                );
                Some(Arc::new(Mutex::new(cache)))
            }
            Err(e) => {
                warn!(
                    "Failed to open DDS cache: {}. Running without disk cache.",
                    e
                );
                None
            }
        }
    } else {
        info!("DDS disk cache disabled");
        None
    };

    let _fs = if let Some(dc) = dds_cache {
        DdsFileSystem::with_disk_cache_and_custom_map(
            fetcher,
            dc,
            custom_map,
            &config.tile_provider,
            dds_cache_entries,
        )
    } else {
        DdsFileSystem::new_with_custom_map(
            fetcher,
            custom_map,
            &config.tile_provider,
            dds_cache_entries,
        )
    };

    let stats = Arc::new(StatsStore::new());
    let tracker = Arc::new(DatarefTracker::new());
    let web_config = Arc::new(parking_lot::RwLock::new(config.clone()));
    let addr = autoortho_lib::webui::start_server(5847, stats, tracker, web_config)
        .await
        .map_err(|e| format!("Web server error: {}", e))?;
    info!("Web UI at http://{}", addr);

    info!("AutoOrtho ready. Mount: {}", config.mount_dir().display());
    info!("Press Ctrl+C to shut down.");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
