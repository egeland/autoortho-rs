# Services Module

## Purpose

Service traits for dependency injection, enabling FUSE-less testing and swappable implementations.

## Ownership

- `services.rs` — Module root, re-exports traits and types
- `services/tile_service.rs` — `TileService` trait and implementations

## Local Contracts

- `TileService` trait defines `get_dds(coords, provider, night_exclusion)` and `tile_exists()`
- `TileCoord` is re-exported from `tiles::coords` for convenience
- Production: `TileServiceImpl` wraps `DdsFileSystem`
- Testing: `FakeTileService` provides deterministic responses

## Work Guidance

- Add new traits for other service boundaries (e.g., `StatsService`, `ConfigService`)
- Each trait should have a corresponding fake implementation in tests
- Trait methods should be async to match the runtime model

## Verification

- `cargo test --lib services::`
- Integration tests use real `DdsFileSystem` with mock providers

## Child DOX Index

None — flat module.