# Pipeline Module

## Purpose

Image processing pipeline: JPEG decode → DDS generation, memory budget management, compression, caching.

## Ownership

- `decode.rs` — JPEG/image decoding (image crate)
- `dds.rs` — DDS header generation, BC3 compression, file assembly
- `cache.rs` — Tile image LRU cache (memory + disk)
- `budget.rs` — Memory budget enforcement for tile cache
- `compress.rs` — Image compression utilities
- `image.rs` — Shared image types and conversions

## Local Contracts

- All DDS output is 4096×4096 BC3 format
- `calculate_dds_size()` in `fuse/mod.rs` must match `dds_file_size_4096_bc3()`
- Cache entries keyed by `(row, col, maptype, zoom)`
- Budget limits: `dds_memory_cache_mb`, `chunk_memory_cache_mb`

## Work Guidance

- Keep `tests.rs` for integration-style pipeline tests
- Unit tests live in each module file
- When modifying DDS format, verify against `DdsPathParser` in `fuse/mod.rs`

## Verification

- `cargo test --lib pipeline` runs all pipeline unit tests
- DDS output validated in `fuse/mod.rs` tests

## Child DOX Index

None — flat module.
