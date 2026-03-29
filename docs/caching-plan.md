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
- **DDS in-memory cache size from config** — Hardcoded to 256 entries, config value not used
- **DiskBudgetManager eviction** — Not wired up, cache grows without bound
- FallbackLevel::None (disabled)

---

## Summary

The DDS caching is fully functional with disk persistence. The main missing piece is the JPEG disk cache for raw tiles.

### Remaining Work (Priority Order)

1. **JPEG disk cache** - Highest priority missing feature
2. **Wire DDS in-memory cache size from config** - Currently hardcoded to 256
3. **Add DiskBudgetManager eviction** - Cache grows without bound
4. **Add FallbackLevel::None option**

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

## BCn Compression Upgrade

### Current State
`src/pipeline/compress.rs` uses hand-rolled pure Rust BC1/BC3 compressor.

### Plan: texpresso Integration

Replace with **texpresso** v2 — a pure Rust BCn texture compression suite that is significantly faster.

**Status: NOT STARTED** - This is an optional performance enhancement.

### Implementation

```toml
# Cargo.toml
texpresso = "2.0"
```

Use behind feature flag, benchmark before enabling by default.
