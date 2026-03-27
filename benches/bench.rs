use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_dds_compression_bc1(c: &mut Criterion) {
    use autoortho_lib::pipeline::compress::compress_image;
    use autoortho_lib::pipeline::dds::DdsFormat;

    let width = 256u32;
    let height = 256u32;
    let data: Vec<u8> = (0..width * height * 4).map(|i| (i % 256) as u8).collect();

    c.bench_function("compress_image BC1 256x256", |b| {
        b.iter(|| {
            compress_image(
                black_box(&data),
                black_box(width),
                black_box(height),
                DdsFormat::BC1,
            )
        });
    });
}

fn bench_dds_compression_bc3(c: &mut Criterion) {
    use autoortho_lib::pipeline::compress::compress_image;
    use autoortho_lib::pipeline::dds::DdsFormat;

    let width = 256u32;
    let height = 256u32;
    let data: Vec<u8> = (0..width * height * 4).map(|i| (i % 256) as u8).collect();

    c.bench_function("compress_image BC3 256x256", |b| {
        b.iter(|| {
            compress_image(
                black_box(&data),
                black_box(width),
                black_box(height),
                DdsFormat::BC3,
            )
        });
    });
}

fn bench_jpeg_decode(c: &mut Criterion) {
    use autoortho_lib::pipeline::decode::ImageBuffer;
    use std::fs;

    // Read a real JPEG from test assets if available, otherwise use a placeholder
    let jpeg_data = fs::read("tests/assets/arc_14_5453_3406.jpg")
        .or_else(|_| fs::read("tests/assets/google_14_5453_3406.jpg"))
        .unwrap_or_else(|_| {
            // Fallback: create minimal valid JPEG
            vec![
                0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
            ]
        });

    c.bench_function("jpeg_decode 256x256", |b| {
        b.iter(|| ImageBuffer::from_jpeg(black_box(&jpeg_data)));
    });
}

fn bench_coord_conversion(c: &mut Criterion) {
    use autoortho_lib::tiles::coords::TileCoords;

    let lat = 33.9425;
    let lon = -118.4081;
    let zoom = 14u32;

    c.bench_function("latlng_to_tile zoom 14", |b| {
        b.iter(|| TileCoords::latlng_to_tile(black_box(lat), black_box(lon), black_box(zoom)));
    });
}

fn bench_haversine(c: &mut Criterion) {
    use autoortho_lib::xplane::simbrief::haversine_nm;

    let lat1 = 33.9425;
    let lon1 = -118.4081;
    let lat2 = 40.6413;
    let lon2 = -73.7781;

    c.bench_function("haversine distance LAX-JFK", |b| {
        b.iter(|| {
            haversine_nm(
                black_box(lat1),
                black_box(lon1),
                black_box(lat2),
                black_box(lon2),
            )
        });
    });
}

fn bench_route_distance(c: &mut Criterion) {
    use autoortho_lib::xplane::simbrief::{FlightFix, FlightPlan};

    let plan = FlightPlan {
        origin: "KLAX".to_string(),
        destination: "KJFK".to_string(),
        cruise_altitude_ft: 35000.0,
        origin_elevation_ft: 126.0,
        destination_elevation_ft: 13.0,
        fixes: vec![
            FlightFix {
                ident: "KLAX".to_string(),
                name: "Los Angeles Intl".to_string(),
                fix_type: "apt".to_string(),
                lat: 33.9425,
                lon: -118.4081,
                altitude_ft: 0.0,
                ground_height_ft: 126.0,
                time_total_sec: 0.0,
                time_leg_sec: 0.0,
                ground_speed_kt: 0.0,
            },
            FlightFix {
                ident: "PEEGG".to_string(),
                name: "Peary".to_string(),
                fix_type: "wpt".to_string(),
                lat: 34.5,
                lon: -117.0,
                altitude_ft: 15000.0,
                ground_height_ft: 2500.0,
                time_total_sec: 600.0,
                time_leg_sec: 600.0,
                ground_speed_kt: 450.0,
            },
            FlightFix {
                ident: "KJFK".to_string(),
                name: "John F Kennedy Intl".to_string(),
                fix_type: "apt".to_string(),
                lat: 40.6413,
                lon: -73.7781,
                altitude_ft: 0.0,
                ground_height_ft: 13.0,
                time_total_sec: 14400.0,
                time_leg_sec: 13800.0,
                ground_speed_kt: 450.0,
            },
        ],
    };

    c.bench_function("get_prefetch_points full route", |b| {
        b.iter(|| {
            plan.get_prefetch_points(
                black_box(33.9425),
                black_box(-118.4081),
                black_box(10.0),
                black_box(99999.0),
            )
        });
    });
}

criterion_group!(
    benches,
    bench_dds_compression_bc1,
    bench_dds_compression_bc3,
    bench_jpeg_decode,
    bench_coord_conversion,
    bench_haversine,
    bench_route_distance
);
criterion_main!(benches);
