# Caching Strategy — Implementation Plan

## Overview

Implement persistent disk caching for both raw JPEG tiles and pre-built DDS textures, with configurable size limits and LRU eviction.

## Current Status: MOSTLY COMPLETE ✅

### What's Implemented ✅
- `src/pipeline/cache.rs` — `DdsCache` with zstd compression, atomic writes, DDM metadata
- Config: `dds_cache_size_mb`, `enable_dds_cache`, `dds_memory_cache_mb`, `chunk_memory_cache_mb`
- DDS disk cache wired into DdsFileSystem
- In-memory LRU for chunks (via TileFetcher LruCache)
- In-memory LRU for DDS tiles (via DdsFileSystem LruCache)
- Upserving: higher-zoom DDS tiles used when lower-zoom requested
- Fallback system: `FallbackLevel::Cache`, `Downserve`, `Network`, `Solid`
- Cache management UI: sliders, clear buttons, enable toggle

### What's NOT Implemented ❌
- **JPEG disk cache** — Raw downloaded tiles never persisted to disk (only in-memory)
- **DiskBudgetManager eviction** — Not wired up, cache grows without bound
- FallbackLevel::None (disabled)

### Recently Fixed
- ~~DDS in-memory cache size from config~~ — ✅ Fixed: `dds_memory_cache_mb` now wired through convenience constructors to DdsFileSystem

---

## Summary

The DDS caching is fully functional with disk persistence. The main missing piece is the JPEG disk cache for raw tiles.

### Remaining Work (Priority Order)

1. **JPEG disk cache** - Highest priority missing feature
2. **Add DiskBudgetManager eviction** - Cache grows without bound
3. **Add FallbackLevel::None option**

---

## Files

| File | Status |
|------|--------|
| `src/config.rs` | ✅ Complete |
| `src/tiles/jpeg_cache.rs` | ❌ Not created |
| `src/pipeline/cache.rs` | ✅ Complete |
| `src/fuse/filesystem.rs` | ✅ Complete |
| `src/ui/screens/settings.rs` | ✅ Complete |

---

## BCn Compression ✅

### Current State
Using **texpresso** v2.0.2 with rayon for parallel BC1/BC3 compression.

`texpresso = "2.0.2"` is in Cargo.toml and actively used in the pipeline. The original hand-rolled compressor in `src/pipeline/compress.rs` has been replaced. Criterion benchmarks exist in `benches/bench.rs` for BC1 and BC3 compression performance.
