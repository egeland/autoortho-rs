// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Integration tests for the xplane-simulator binary.
//!
//! These tests verify end-to-end behavior:
//! - DDS files produced by autoortho_lib are valid
//! - Validation logic matches what the lib produces
//! - Reference metadata format is correct

use std::path::PathBuf;

/// Create a temporary mount directory with a valid DDS file for testing.
fn create_test_mount() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let mount = tmp.path().to_path_buf();
    let textures = mount.join("textures");
    std::fs::create_dir_all(&textures).expect("create textures dir");
    (tmp, mount)
}

/// Test that autoortho_lib produces DDS files our validator accepts.
#[test]
fn test_lib_dds_validates_against_our_parser() {
    use autoortho_lib::pipeline::dds::{DdsFormat, build_fallback_dds};
    use xplane_simulator::validation::validate_dds;

    // Test BC1 (DXT1) - default format
    let bc1 = build_fallback_dds(4096, 4096, DdsFormat::BC1, [120, 150, 100]);
    let result = validate_dds(&bc1);
    assert!(
        result.is_ok(),
        "BC1 DDS should validate: {:?}",
        result.err()
    );

    // Test BC3 (DXT5) - default for autoortho
    let bc3 = build_fallback_dds(4096, 4096, DdsFormat::BC3, [120, 150, 100]);
    let result = validate_dds(&bc3);
    assert!(
        result.is_ok(),
        "BC3 DDS should validate: {:?}",
        result.err()
    );
}

/// Test that we can write a DDS file to a temp mount and read it back.
#[test]
fn test_dds_round_trip_through_filesystem() {
    use autoortho_lib::pipeline::dds::{DdsFormat, build_fallback_dds};
    use xplane_simulator::validation::validate_dds;

    let (_tmp, mount) = create_test_mount();
    let texture_path = mount.join("textures").join("3232_488_BI16.dds");

    // Write a valid DDS file
    let data = build_fallback_dds(4096, 4096, DdsFormat::BC3, [100, 150, 200]);
    std::fs::write(&texture_path, &data).expect("write DDS");

    // Read it back
    let read_back = std::fs::read(&texture_path).expect("read DDS");
    assert_eq!(read_back.len(), data.len());
    assert_eq!(&read_back[0..4], b"DDS ");

    // Validate
    let info = validate_dds(&read_back).expect("validate");
    assert_eq!(info.width, 4096);
    assert_eq!(info.height, 4096);
}

/// Test SHA256 computation is consistent.
#[test]
fn test_sha256_consistency() {
    use xplane_simulator::validation::sha256;

    let data = b"hello world";
    let h1 = sha256(data);
    let h2 = sha256(data);
    assert_eq!(h1, h2, "SHA256 should be deterministic");
    assert_eq!(h1.len(), 64, "SHA256 hex should be 64 chars");
}

/// Test SHA256 of empty and known data.
#[test]
fn test_sha256_known_values() {
    use xplane_simulator::validation::sha256;

    // Known SHA256 of empty string
    let h = sha256(b"");
    assert_eq!(
        h, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "Empty string SHA256 should match"
    );

    // Known SHA256 of "abc"
    let h = sha256(b"abc");
    assert_eq!(
        h, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "abc SHA256 should match"
    );
}

/// Test that a corrupted DDS file is rejected.
#[test]
fn test_corrupted_dds_rejected() {
    use xplane_simulator::validation::validate_dds;

    // Empty data
    let result = validate_dds(&[]);
    assert!(result.is_err());

    // Wrong magic
    let mut bad = vec![0u8; 128];
    bad[0] = b'X';
    let result = validate_dds(&bad);
    assert!(result.is_err());

    // Correct magic but wrong dimensions
    let mut bad = vec![0u8; 128];
    bad[0..4].copy_from_slice(b"DDS ");
    bad[4..8].copy_from_slice(&124u32.to_le_bytes());
    bad[12..16].copy_from_slice(&1024u32.to_le_bytes()); // wrong height
    bad[16..20].copy_from_slice(&1024u32.to_le_bytes()); // wrong width
    let result = validate_dds(&bad);
    assert!(result.is_err(), "Wrong dimensions should fail");
}

/// Test that route generation produces valid tile coordinates.
#[test]
fn test_route_tiles_yssy_to_ymml() {
    use autoortho_lib::tiles::coords::TileCoord;
    use xplane_simulator::route::Route;

    let route = Route::from_spec("YSSY→YMML").expect("parse route");
    assert_eq!(route.waypoints.len(), 2);
    assert_eq!(route.waypoints[0].name, "YSSY");
    assert_eq!(route.waypoints[1].name, "YMML");

    // Generate tile coordinates at zoom 12
    let waypoints = route.interpolate_waypoints(20);
    let mut coords: Vec<TileCoord> = Vec::new();
    for wp in &waypoints {
        if let Ok((col, row)) =
            autoortho_lib::tiles::coords::TileCoords::latlng_to_tile(wp.lat, wp.lon, 12)
        {
            coords.push(TileCoord::new(row, col, 12).expect("create coord"));
        }
    }

    assert!(!coords.is_empty(), "Should have tile coords");
    // All coords should be in the Australia region at zoom 12
    // YSSY at z12: row=2458, col=3768
    // YMML at z12: row=2511, col=3695
    for coord in &coords {
        assert!(coord.zoom == 12);
        // Allow a wide range to cover the entire flight path
        assert!(
            coord.row > 2400 && coord.row < 2550,
            "Row out of range: {}",
            coord.row
        );
        assert!(
            coord.col > 3650 && coord.col < 3800,
            "Col out of range: {}",
            coord.col
        );
    }
}

/// Test reference metadata format.
#[test]
fn test_reference_metadata_format() {
    use autoortho_lib::tiles::coords::TileCoord;
    use xplane_simulator::validation::{ReferenceMetadata, sha256};

    let coord = TileCoord::new(3232, 488, 16).expect("create coord");
    let meta = ReferenceMetadata::from_tile(&coord, 12345, sha256(b"test"), true);

    // Serialize to JSON
    let json = serde_json::to_string(&meta).expect("serialize");
    assert!(json.contains("\"row\":3232"));
    assert!(json.contains("\"col\":488"));
    assert!(json.contains("\"zoom\":16"));
    assert!(json.contains("\"size\":12345"));
    assert!(json.contains("\"validated\":true"));
}

/// Test error type displays useful information.
#[test]
fn test_error_messages_are_informative() {
    use xplane_simulator::error::TileError;

    let err = TileError::IoError("permission denied".to_string());
    let msg = err.to_string();
    assert!(
        msg.contains("permission denied"),
        "Error should include underlying message"
    );

    let err = TileError::InvalidFormat("bad magic".to_string());
    let msg = err.to_string();
    assert!(
        msg.contains("bad magic"),
        "Error should include format detail"
    );

    let err = TileError::MountUnavailable;
    assert!(
        err.to_string().to_lowercase().contains("mount"),
        "Error should mention mount, got: {}",
        err
    );
}

/// Test that performance tracking is correct.
#[test]
fn test_perf_stats_aggregation() {
    use std::time::Duration;
    use xplane_simulator::perf::PerfStats;

    let mut stats = PerfStats::new();
    stats.record_read(1024);
    stats.record_read(2048);
    stats.record_read(512);

    assert_eq!(stats.tiles_read(), 3);
    assert_eq!(stats.total_bytes(), 3584);
    assert_eq!(stats.total_bytes(), 1024 + 2048 + 512);

    stats.record_latency(Duration::from_millis(10));
    stats.record_latency(Duration::from_millis(20));
    assert_eq!(stats.average_latency(), Duration::from_millis(15));
}
