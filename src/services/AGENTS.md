# Services Module

## Purpose

Service traits for dependency injection, enabling FUSE-less testing and swappable implementations.

## Ownership

- `mod.rs` — Module root, re-exports traits and types
- `stats_service.rs` — `StatsService` trait and `StatsServiceImpl` / `FakeStatsService`
- `fallback_service.rs` — `FallbackService` trait and `FallbackServiceImpl` / `FakeFallbackService`
- `cache_service.rs` — `CacheService` trait and `CacheServiceImpl` / `FakeCacheService`

## Local Contracts

- `StatsService` trait: `record_download()`, `record_cache_hit()`, `record_cache_miss()`, `snapshot()`, `hit_ratio()`, `clear()`
- `FallbackService` trait: `find_fallback()`, `solid_fallback()`, `needs_fallback()`
- `CacheService` trait: `get()`, `put()`, `has()`, `remove()`, `clear()`, `entry_count()`, `size_bytes()`, `max_size_bytes()`, `usage_fraction()`, `promote()`, `evict_non_route_tiles()`
- Each trait has a `Fake*` implementation in `#[cfg(test)]` module for testing
- Production impls wrap concrete types: `StatsStore`, `FallbackSystem`, `DdsCache`

## Work Guidance

- Traits enable `DdsFileSystem` to depend on abstractions, not concrete types
- Fake implementations are `pub(crate)` for use by integration tests
- Keep trait methods async to match the runtime model

## Verification

- `cargo test --lib services::`
- Each service module has unit tests for fake impl and production impl

## Child DOX Index

None — flat module.