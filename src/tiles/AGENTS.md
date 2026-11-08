# Tiles Module

## Purpose

Tile engine: coordinate math, chunk management, tile assembly, prefetching, provider abstraction, network fetching, fallback handling.

## Ownership

- `coords.rs` — Lat/lng ↔ tile coordinate conversion
- `chunk.rs` — Tile chunk types and chunk-level operations
- `assembler.rs` — Assemble DDS from fetched image chunks
- `tile.rs` — Tile types, tile-level operations
- `zoom.rs` — `ZoomRule` type, zoom-level calculations
- `provider.rs` — `ProviderFactory`, tile provider abstraction
- `fetcher.rs` — `TileFetcher`, HTTP tile fetching with retry
- `prefetch.rs` — `SpatialPrefetcher`, route-based prefetch logic
- `route_prefetch.rs` — `RoutePrefetchEngine`, SimBrief integration loop
- `fallback.rs` — `FallbackConfig`, `FallbackLevel`, fallback DDS generation
- `rate_limiter.rs` — Per-provider rate limiting
- `apple_token.rs` — Apple Maps token handling
- `tile_cache.rs` — In-memory tile cache
- `tile_generator.rs` — Tile generation pipeline
- `tile_resolution.rs` — Tile resolution orchestrator
- `healer.rs` — Background re-generation of incomplete disk-cached tiles
- `assembly_pipeline.rs` — `AssemblyPipeline` service trait wrapper

## Local Contracts

- `ZoomRule` is re-exported from `config.rs` for backward compat
- `FallbackConfig` / `FallbackLevel` re-exported from `config.rs`
- Prefetch uses `SpatialPrefetcher` with optional `RoutePrefetchConfig`
- `RoutePrefetchEngine` owns the SimBrief integration loop (was in main.rs)
- `TileFetcher` requires runtime handle (tokio)
- `AssemblyResult.failed_indices` lists chunks filled with fallback color;
  `DdsCacheMetadata.fallback_indices` persists them, `attempts` counts healing
  passes (serde-default for pre-existing .ddm files)
- Incomplete tiles are identified by `DdsCacheMetadata::needs_healing()`;
  `TileResolution::regenerate()` re-generates bypassing cache and bumps
  `attempts`; healer caps at `MAX_ATTEMPTS` (3)
- Disk cache keys come in two formats: `row_col_MAP_zoom` (TileResolution) and
  `col_row_MAP_zzoom` (fallback up-serving); `healer::parse_tile_key` accepts both

## Work Guidance

- `provider.rs` is the entry point for adding new tile sources
- `prefetch.rs` depends on `xplane::FlightDataStore` (via `FlightDataTracker` trait) for position
- `route_prefetch.rs` encapsulates SimBrief flight plan prefetch logic
- Rate limiter is per-provider, not global

## Verification

- `cargo test --lib tiles`
- `route_prefetch.rs` tests: engine creation, SimBrief integration
- `healer.rs` tests: key parsing (both formats), heal pass behavior (heal + attempt cap)

## Child DOX Index

None — flat module.
