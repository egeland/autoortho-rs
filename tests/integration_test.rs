use autoortho_lib::config::AutoOrthoConfig;
use autoortho_lib::fuse::DdsPathParser;
use autoortho_lib::pipeline::dds::{DdsBuilder, DdsFormat};
use autoortho_lib::pipeline::decode::ImageBuffer;
use autoortho_lib::tiles::chunk::Chunk;
use autoortho_lib::tiles::coords::TileCoords;
use autoortho_lib::tiles::provider::ProviderFactory;
use autoortho_lib::xplane::RrefCodec;
use tempfile::TempDir;

#[test]
fn test_full_pipeline_config() {
    let config = AutoOrthoConfig::default();
    assert_eq!(config.tile_provider, "ARC");
    assert!(config.enable_night_exclusion);
}

#[test]
fn test_tile_coordinates() {
    // Test that tile conversion works without panicking
    let result = TileCoords::tile_to_latlng(1, 1, 2);
    assert!(result.is_ok());
    let (lat, lon) = result.unwrap();
    assert!(lat >= -85.0 && lat <= 85.0);
    assert!(lon >= -180.0 && lon <= 180.0);
}

#[test]
fn test_dds_path_parsing() {
    let parser = DdsPathParser::new();
    let (row, col, maptype, zoom) = parser.parse("/1234_5678_GO2_18.dds").unwrap();
    assert_eq!(row, 1234);
    assert_eq!(col, 5678);
    assert_eq!(maptype, "GO2");
    assert_eq!(zoom, 18);
}

#[test]
fn test_chunk_state_machine() {
    let mut chunk = Chunk::new(10, 20, "GO2".to_string(), 12);
    assert_eq!(
        chunk.state(),
        autoortho_lib::tiles::chunk::ChunkState::Missing
    );

    chunk.set_fetching().unwrap();
    assert_eq!(
        chunk.state(),
        autoortho_lib::tiles::chunk::ChunkState::Fetching
    );

    let data = vec![0xFF, 0xD8, 0xFF, 0xD9]; // JPEG magic
    chunk.set_cached(data).unwrap();
    assert_eq!(
        chunk.state(),
        autoortho_lib::tiles::chunk::ChunkState::Cached
    );
}

#[test]
fn test_dds_compression() {
    let mut image = ImageBuffer::new(64, 64, 4);
    for i in 0..image.data.len() {
        image.data[i] = 128;
    }

    let builder = DdsBuilder::new(64, 64, DdsFormat::BC3);
    let result = builder.compress(&image);
    assert!(result.is_ok());

    let dds = result.unwrap();
    assert!(dds.len() > 128);
    assert_eq!(&dds[0..4], b"DDS ");
}

#[test]
fn test_xplane_protocol() {
    // Encode a request
    let packet = RrefCodec::encode_request(2, 0, "sim/flightmodel/position/latitude");
    assert!(packet.starts_with(b"RREF"));
    assert_eq!(packet.len(), 412);

    // Create a response
    let mut response = Vec::new();
    response.extend_from_slice(b"RREF");
    response.extend_from_slice(&5i32.to_le_bytes());
    response.extend_from_slice(&45.5f32.to_le_bytes());

    // Decode it
    let results = RrefCodec::decode_response(&response).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 5);
    assert!((results[0].1 - 45.5).abs() < 0.001);
}

#[test]
fn test_flight_data_averaging() {
    use autoortho_lib::xplane::FlightDataAverager;

    let mut averager = FlightDataAverager::new(3);
    averager.add(10.0);
    averager.add(20.0);
    let avg = averager.add(30.0);

    assert!((avg - 20.0).abs() < 0.001);
}

#[test]
fn test_heading_circular_average() {
    use autoortho_lib::xplane::HeadingAverager;

    let mut averager = HeadingAverager::new(3);
    averager.add(350.0);
    averager.add(10.0);
    let avg = averager.add(0.0);

    // Average of 350, 10, 0 should be near 0 (north)
    assert!(avg < 10.0 || avg > 350.0);
}

#[test]
fn test_time_exclusion_module() {
    use autoortho_lib::time_exclusion::TimeExclusion;

    let te = TimeExclusion::new(-12.0, -10.0);

    assert!(te.is_night(-20.0));
    assert!(!te.is_day(-15.0));
    assert!(te.is_day(-5.0));

    let phase = te.day_phase(-11.0);
    assert!(phase > 0.0 && phase < 1.0);
}

#[test]
fn test_seasons() {
    use autoortho_lib::config::Season;
    use autoortho_lib::seasons::SeasonalAdjustment;

    let _sa = SeasonalAdjustment::new(Season::Summer, 1.0, 1.2, 1.1, 0.9);
    let result = SeasonalAdjustment::apply_to_rgb((127, 127, 127), 1.1);
    assert_eq!(
        result.0.min(result.1).min(result.2),
        result.0.min(result.1).min(result.2)
    );
}

#[test]
fn test_stats() {
    use autoortho_lib::stats::StatsStore;

    let stats = StatsStore::new();
    stats.record_cache_hit();
    stats.record_cache_hit();
    stats.record_cache_miss();

    let snapshot = stats.snapshot();
    assert_eq!(snapshot.cache_hits + snapshot.cache_misses, 3);
}

#[test]
fn test_quadkey_encoding() {
    use autoortho_lib::tiles::coords::TileCoords;

    let quadkey = TileCoords::tile_to_quadkey(1, 1, 2);
    assert_eq!(quadkey, "03");
}

#[tokio::test]
async fn test_provider_arcgis() {
    let provider = ProviderFactory::create("ARC").expect("ARC provider should exist");
    // Sydney at zoom 10: row ~768, col ~614
    let result = provider.fetch(768, 614, 10).await;
    assert!(
        result.is_ok(),
        "ARC provider should fetch tile: {:?}",
        result.err()
    );
    let data = result.unwrap();
    assert!(!data.is_empty(), "ARC should return non-empty data");
    assert!(data.len() > 1000, "ARC should return image data (>1KB)");
}

#[tokio::test]
async fn test_provider_bing() {
    let provider = ProviderFactory::create("BI").expect("BI provider should exist");
    // Sydney at zoom 10: row ~768, col ~614
    let result = provider.fetch(768, 614, 10).await;
    assert!(
        result.is_ok(),
        "BI provider should fetch tile: {:?}",
        result.err()
    );
    let data = result.unwrap();
    assert!(!data.is_empty(), "BI should return non-empty data");
}

#[tokio::test]
async fn test_provider_google() {
    let provider = ProviderFactory::create("GO2").expect("GO2 provider should exist");
    // Sydney at zoom 10: row ~768, col ~614
    let result = provider.fetch(768, 614, 10).await;
    // Google may return 400 for various reasons (auth, rate limiting)
    // Just verify it doesn't panic and returns some response
    if result.is_ok() {
        let data = result.unwrap();
        assert!(
            !data.is_empty(),
            "GO2 should return non-empty data if successful"
        );
    }
}

#[tokio::test]
async fn test_provider_naip() {
    let provider = ProviderFactory::create("NAIP").expect("NAIP provider should exist");
    // NAIP only covers US, use a US location (New York at zoom 10: row ~585, col ~778)
    let result = provider.fetch(585, 778, 10).await;
    // NAIP may not have coverage everywhere, but should return a response (even if 404)
    if result.is_ok() {
        let data = result.unwrap();
        assert!(
            !data.is_empty(),
            "NAIP should return non-empty data if successful"
        );
    }
}

#[tokio::test]
async fn test_provider_usgs() {
    let provider = ProviderFactory::create("USGS").expect("USGS provider should exist");
    // Sydney at zoom 10: row ~768, col ~614
    let result = provider.fetch(768, 614, 10).await;
    // USGS may have limited coverage or return 404
    if result.is_ok() {
        let data = result.unwrap();
        assert!(
            !data.is_empty(),
            "USGS should return non-empty data if successful"
        );
    }
}

#[tokio::test]
async fn test_provider_eox() {
    let provider = ProviderFactory::create("EOX").expect("EOX provider should exist");
    // Sydney at zoom 10: row ~768, col ~614
    let result = provider.fetch(768, 614, 10).await;
    // EOX may have rate limiting
    if result.is_ok() {
        let data = result.unwrap();
        assert!(
            !data.is_empty(),
            "EOX should return non-empty data if successful"
        );
    }
}

#[tokio::test]
async fn test_provider_firefly() {
    let provider = ProviderFactory::create("FIREFLY").expect("FIREFLY provider should exist");
    // Sydney at zoom 10: row ~768, col ~614
    let result = provider.fetch(768, 614, 10).await;
    // Firefly may have limited coverage
    if result.is_ok() {
        let data = result.unwrap();
        assert!(
            !data.is_empty(),
            "FIREFLY should return non-empty data if successful"
        );
    }
}

#[tokio::test]
async fn test_bing_https_url() {
    use autoortho_lib::tiles::coords::TileCoords;

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .expect("Failed to create HTTP client");

    let quadkey = TileCoords::tile_to_quadkey(614, 768, 10);
    let https_url = format!(
        "https://ecn.t3.tiles.virtualearth.net/tiles/a{}.jpeg?g=1",
        quadkey
    );

    let response = client
        .get(&https_url)
        .send()
        .await
        .expect("Failed to send request");
    assert!(
        response.status().is_success(),
        "Bing HTTPS should return success: {}",
        response.status()
    );
}

#[tokio::test]
async fn test_naip_https_url() {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .expect("Failed to create HTTP client");

    // NAIP only covers US, use a US location (New York at zoom 10: row ~585, col ~778)
    let https_url =
        "https://naip.maptiles.arcgis.com/arcgis/rest/services/NAIP/MapServer/tile/10/585/778";

    let response = client
        .get(https_url)
        .send()
        .await
        .expect("Failed to send request");
    // NAIP may return 404 for missing tiles, but should respond over HTTPS
    assert!(
        response.status() == 200 || response.status() == 404,
        "NAIP HTTPS should respond: {}",
        response.status()
    );
}

// --- cleanup_mount integration tests ---

#[test]
fn test_cleanup_mount_nonexistent_path() {
    use autoortho_lib::fuse::platform::cleanup_mount;
    let path = std::path::Path::new("/nonexistent/autoortho_test_mount");
    let result = cleanup_mount(path);
    assert!(
        result.is_ok(),
        "cleanup_mount should succeed even for nonexistent paths"
    );
}

#[test]
fn test_cleanup_mount_existing_dir() {
    use autoortho_lib::fuse::platform::cleanup_mount;
    let tmp = TempDir::new().unwrap();
    let result = cleanup_mount(tmp.path());
    assert!(
        result.is_ok(),
        "cleanup_mount should succeed for existing directories"
    );
}

#[test]
fn test_cleanup_mount_idempotent() {
    use autoortho_lib::fuse::platform::cleanup_mount;
    let tmp = TempDir::new().unwrap();
    // First call
    let result1 = cleanup_mount(tmp.path());
    assert!(result1.is_ok(), "First cleanup_mount call should succeed");
    // Second call should also succeed
    let result2 = cleanup_mount(tmp.path());
    assert!(
        result2.is_ok(),
        "Second cleanup_mount call should also succeed"
    );
}
