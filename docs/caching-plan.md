# Caching Strategy — Implementation Plan

## Overview

Implement persistent disk caching for both raw JPEG tiles and pre-built DDS textures, with configurable size limits and LRU eviction.

Reference: [autoortho4xplane docs/performance.md](https://github.com/ProgrammingDinosaur/autoortho4xplane/blob/develop/docs/performance.md) (Ephemeral DDS Cache, JPEG Cache sections)

---

## Current State ✅ MOSTLY DONE

### What's Implemented ✅
- `src/pipeline/cache.rs` — `DdsCache` with zstd compression, atomic writes, DDM metadata
- Config: `dds_cache_size_mb`, `enable_dds_cache`, `dds_memory_cache_mb`, `chunk_memory_cache_mb`
- DDS disk cache wired into DdsFileSystem
- In-memory LRU for chunks (via TileFetcher LruCache)
- In-memory LRU for DDS tiles (via DdsFileSystem LruCache)
- Upserving: higher-zoom DDS tiles used when lower-zoom requested
- Fallback system: `FallbackLevel::Cache`, `Downserve`, `Network`, `Solid`
- Cache management UI: sliders, clear buttons, enable toggle

### What's Missing ❌
- **JPEG disk cache** — Raw downloaded tiles never persisted to disk
- **DDS in-memory cache size from config** — Hardcoded to 256 entries, config value not used
- **DiskBudgetManager eviction** — Not wired up, cache grows without bound
- FallbackLevel::None (disabled)

### The Problem
Every app restart = all tiles re-downloaded and re-assembled from scratch. For a 30-minute flight covering 50 tiles at ZL16:
- **First session**: ~50 tiles × 256 chunks × 50KB JPEG = ~640MB downloads + decode + compress
- **Subsequent sessions**: same cost (no cache benefit)
- **Python version**: downloads once, caches to disk, next session instant

---

## Design

### 1. Config — Cache Settings

**File**: `src/config.rs`

Add cache size fields:
```rust
pub struct AutoOrthoConfig {
    // ... existing fields ...

    // JPEG tile cache (raw downloaded chunks)
    pub jpeg_cache_size_mb: u64,        // default: 2048 (2 GB)
    pub enable_jpeg_cache: bool,        // default: true

    // DDS tile cache (pre-built textures)
    pub dds_cache_size_mb: u64,         // default: 4096 (4 GB)
    pub enable_dds_cache: bool,         // default: true
}
```

### 2. JPEG Disk Cache

**File**: `src/tiles/jpeg_cache.rs` (new)

Persistent cache for raw downloaded JPEG chunks before assembly:
```
cache_dir/
  jpeg/
    {row}_{col}_{maptype}_z{zoom}.jpg.zst
    {row}_{col}_{maptype}_z{zoom}.jcm       # metadata: fetch time, etag, size
```

**Structure**:
```rust
/// JPEG cache metadata (JSON sidecar)
#[derive(Serialize, Deserialize)]
pub struct JpegCacheMeta {
    pub fetched: f64,        // Unix timestamp
    pub etag: Option<String>,
    pub size_bytes: u64,
    pub zoom: u32,
    pub maptype: String,
    pub row: u32,
    pub col: u32,
}

/// JPEG disk cache
pub struct JpegCache {
    cache_dir: PathBuf,
    max_size_bytes: u64,
    current_bytes: u64,
    index: LruCache<String, u64>, // key → size (LRU eviction)
    in_memory: HashMap<String, Arc<Vec<u8>>>, // hot read cache
}

impl JpegCache {
    pub fn new(cache_dir: PathBuf, max_size_bytes: u64) -> Self;
    pub fn get(&self, key: &str) -> Option<Arc<Vec<u8>>>;
    pub fn put(&self, key: &str, data: Vec<u8>, meta: &JpegCacheMeta) -> Result<(), CacheError>;
    pub fn remove(&self, key: &str) -> Result<(), CacheError>;
    pub fn clear(&self) -> Result<(), CacheError>;
    pub fn size_bytes(&self) -> u64;
    pub fn max_size_bytes(&self) -> u64;
    pub fn entry_count(&self) -> usize;
}
```

**Key design decisions**:
- Keep in-memory LRU layer (small, e.g., 256 entries) for hot tiles — avoids disk I/O on repeat reads
- Zstd compression for cache files (~50% compression on JPEG)
- Atomic writes via temp file + rename
- On startup: scan cache directory to rebuild index (handles crashes mid-write)
- Staleness: re-fetch if provider returns different ETag or size changed

### 3. Wire JPEG Cache into TileFetcher

**File**: `src/tiles/fetcher.rs`

Modify `TileFetcher` to use disk cache:
```rust
pub struct TileFetcher {
    chunks: Arc<RwLock<HashMap<String, Chunk>>>,  // in-flight chunks only
    provider: Arc<dyn TileProvider>,
    jpeg_cache: Arc<JpegCache>,  // NEW: persistent cache
    in_memory_cache: Arc<JpegCache>,  // hot read cache
}

impl TileFetcher {
    pub fn new(provider: Arc<dyn TileProvider>, jpeg_cache: Arc<JpegCache>) -> Self;

    pub async fn get_chunk_data(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        zoom: u32,
    ) -> Result<Option<Vec<u8>>, ChunkError> {
        let key = format!("{}_{}_{}_{}", row, col, maptype, zoom);

        // 1. Check in-memory hot cache (fast path)
        if let Some(data) = self.in_memory_cache.get(&key) {
            return Ok(Some(data.to_vec()));
        }

        // 2. Check disk cache
        if let Some(data) = self.jpeg_cache.get(&key) {
            // Promote to in-memory cache
            self.in_memory_cache.put(key.clone(), data.clone(), &JpegCacheMeta::default());
            return Ok(Some(data.to_vec()));
        }

        // 3. Fetch from network
        let result = self.provider.fetch(row, col, zoom).await;

        match result {
            Ok(data) => {
                // Store to disk cache (async, non-blocking)
                let meta = JpegCacheMeta { ... };
                if let Err(e) = self.jpeg_cache.put(&key, data.clone(), &meta) {
                    warn!("Failed to cache JPEG: {}", e);
                }
                // Store to in-memory cache
                self.in_memory_cache.put(key, data.clone(), &meta);
                Ok(Some(data))
            }
            Err(e) => Err(ChunkError::DownloadFailed(e.to_string())),
        }
    }

    pub async fn clear_cache(&self) {
        self.jpeg_cache.clear().ok();
        self.in_memory_cache.clear();
    }
}
```

### 4. Wire DDS Cache into DdsFileSystem

**File**: `src/fuse/filesystem.rs`

Modify `DdsFileSystem` to use persistent cache:
```rust
pub struct DdsFileSystem {
    parser: DdsPathParser,
    fetcher: Arc<TileFetcher>,
    format: DdsFormat,
    dds_cache: Arc<DdsCache>,  // CHANGE: was in-memory HashMap
    memory_cache: Mutex<HashMap<String, Arc<Vec<u8>>>>, // hot read cache (small)
    root: Option<PathBuf>,
}

impl DdsFileSystem {
    pub fn new(
        fetcher: Arc<TileFetcher>,
        dds_cache: Arc<DdsCache>,
    ) -> Self;

    pub async fn read_dds(&self, path: &str) -> Result<Arc<Vec<u8>>, FuseError> {
        let tile_key = self.parser.parse_dds(path)?;

        // 1. Check hot memory cache
        {
            let cache = self.memory_cache.lock().expect("poisoned");
            if let Some(dds) = cache.get(&tile_key) {
                return Ok(dds.clone());
            }
        }

        // 2. Check persistent DDS cache
        match self.dds_cache.get(&tile_key) {
            Ok((dds_data, meta)) => {
                // Promote to memory cache
                let arc = Arc::new(dds_data);
                {
                    let mut cache = self.memory_cache.lock().expect("poisoned");
                    cache.insert(tile_key.clone(), arc.clone());
                }
                return Ok(arc);
            }
            Err(CacheError::KeyNotFound) => { /* fall through */ }
            Err(e) => warn!("Cache read error: {}", e),
        }

        // 3. Generate DDS (assemble + compress)
        let dds_data = self.generate_tile(&tile_key).await?;

        // 4. Store to persistent cache
        let meta = DdsCacheMetadata { ... };
        if let Err(e) = self.dds_cache.put(tile_key.clone(), &dds_data, &meta) {
            warn!("Failed to cache DDS: {}", e);
        }

        // 5. Store to memory cache
        let arc = Arc::new(dds_data);
        {
            let mut cache = self.memory_cache.lock().expect("poisoned");
            cache.insert(tile_key, arc.clone());
        }

        Ok(arc)
    }

    pub fn clear_cache(&self) {
        self.dds_cache.clear().ok();
        self.memory_cache.lock().expect("poisoned").clear();
    }
}
```

### 5. Cache Size Management

**File**: `src/pipeline/budget.rs`

The `DiskBudgetManager` handles LRU eviction. Wire it into caches:
```rust
impl DdsCache {
    pub fn with_budget(cache_dir: PathBuf, max_size_bytes: u64) -> Self {
        let budget = Arc::new(Mutex::new(DiskBudgetManager::new(max_size_bytes, cache_dir.clone())));
        Self {
            cache_dir,
            budget,
            // ...
        }
    }

    pub fn put(&mut self, key: String, data: &[u8], metadata: &DdsCacheMetadata) -> Result<(), CacheError> {
        let size = zstd::encode_all(data, 3)?.len() as u64;

        // Check if we need to evict
        {
            let mut budget = self.budget.lock().expect("poisoned");
            if budget.would_exceed(size) {
                // Evict LRU items until we have space
                while budget.would_exceed(size) && !budget.is_empty() {
                    if let Some((evicted_key, _)) = budget.pop_lru() {
                        self.remove(&evicted_key)?;
                    }
                }
            }
            budget.add_file(key.clone(), size)?;
        }

        // Write to disk atomically
        // ...
    }
}
```

### 6. Startup — Rebuild Index from Disk

Both caches need to rebuild their index on startup (handles crashes, manual deletions):
```rust
impl DdsCache {
    pub fn open(cache_dir: PathBuf, max_size_bytes: u64) -> Result<Self, CacheError> {
        let mut cache = Self::with_budget(cache_dir, max_size_bytes);

        // Scan existing files to rebuild index
        if cache.cache_dir.exists() {
            for entry in std::fs::read_dir(&cache.cache_dir)? {
                let entry = entry?;
                let name = entry.file_name();
                if name.ends_with(".dds.zst") {
                    let key = name.to_string_lossy().trim_end_matches(".dds.zst").to_string();
                    let size = entry.metadata()?.len();
                    cache.budget.lock().expect("poisoned").add_file(key, size).ok();
                }
            }
        }

        Ok(cache)
    }
}
```

### 7. Cache Health — Staleness & Healing

When loading a cached DDS, check if it needs rebuilding:
```rust
impl DdsFileSystem {
    pub async fn read_dds(&self, path: &str) -> Result<Arc<Vec<u8>>, FuseError> {
        // ...

        // Check cache staleness
        if let Ok((_, meta)) = self.dds_cache.get_metadata(&tile_key) {
            if meta.is_stale(self.format.as_str(), current_max_zoom) {
                debug!("Cached DDS stale, regenerating: {}", tile_key);
                self.dds_cache.remove(&tile_key).ok();
            } else if meta.needs_healing() {
                // Had missing chunks — try to rebuild if budget allows
                debug!("Healing DDS: {}", tile_key);
                // Could spawn background healing task
            }
        }

        // ... generate if not cached
    }
}
```

### 8. UI — Cache Management

**File**: `src/ui/screens/settings.rs` (or new `cache.rs` screen)

Add cache section to Settings:
```
┌─ Cache Settings ──────────────────────────────────┐
│                                                 │
│  JPEG Cache                                     │
│  ┌────────────────────────────────────────────┐  │
│  │ ████████████████░░░░░░░░░░  1.2 / 2.0 GB  │  │
│  └────────────────────────────────────────────┘  │
│  [Clear JPEG Cache]                             │
│                                                 │
│  DDS Cache                                      │
│  ┌────────────────────────────────────────────┐  │
│  │ ████████░░░░░░░░░░░░░░░░  800 / 4.0 GB   │  │
│  └────────────────────────────────────────────┘  │
│  [Clear DDS Cache]  [Clear All Caches]         │
│                                                 │
│  JPEG Cache Size:  [====|========]  2 GB       │
│  DDS Cache Size:   [========|====]  4 GB       │
│                                                 │
│  ☑ Enable JPEG Cache                           │
│  ☑ Enable DDS Cache                           │
└─────────────────────────────────────────────────┘
```

### 9. Config — Max Size Sliders

**File**: `src/config.rs`

Add validation:
```rust
impl AutoOrthoConfig {
    pub fn validate(&mut self) {
        self.jpeg_cache_size_mb = self.jpeg_cache_size_mb.clamp(256, 32768);
        self.dds_cache_size_mb = self.dds_cache_size_mb.clamp(256, 32768);
    }
}
```

---

## Key Files

| File | Changes |
|------|---------|
| `src/config.rs` | Add `jpeg_cache_size_mb`, `dds_cache_size_mb`, enable flags |
| `src/tiles/jpeg_cache.rs` | New: JPEG disk cache with LRU eviction |
| `src/tiles/fetcher.rs` | Wire JPEG cache, add `clear_cache()` |
| `src/pipeline/cache.rs` | Wire `DiskBudgetManager`, add `open()` for index rebuild |
| `src/pipeline/budget.rs` | Minor: ensure eviction works correctly with `DdsCache` |
| `src/fuse/filesystem.rs` | Wire DDS disk cache, staleness check, healing |
| `src/lib.rs` | Create caches at startup, pass to `TileFetcher` and `DdsFileSystem` |
| `src/ui/screens/settings.rs` | Add cache management UI |

---

## Implementation Order

1. **[x] Config**: Add cache size fields and validation ✅
2. **[ ] JPEG cache**: Create `jpeg_cache.rs` with disk persistence + LRU eviction ❌
3. **[x] Wire JPEG cache**: In-memory LRU via TileFetcher ✅
4. **[x] Wire DDS cache**: `DdsFileSystem` uses `DdsCache` (disk) + memory layer ✅
5. **[ ] Budget eviction**: Wire `DiskBudgetManager` into `DdsCache::put()` ❌
6. **[x] Startup rebuild**: `DdsCache::open()` rebuilds index from disk ✅
7. **[x] Staleness check**: `meta.is_stale()` checked on cache read ✅
8. **[ ] Healing**: Background rebuild of tiles with `needs_healing()` ❌
9. **[x] UI**: Cache size bars, clear buttons, size sliders in Settings ✅
10. **[x] Integration**: Caches created at startup in main.rs ✅
11. **[x] Tests**: Unit tests for cache operations ✅

## Remaining Work

- **[ ] Add JPEG disk cache** - Highest priority missing feature
- **[ ] Wire DDS in-memory cache size from config** - Currently hardcoded to 256
- **[ ] Add DiskBudgetManager eviction** - Cache grows without bound
- **[ ] Add FallbackLevel::None option**
- **[ ] Optional: Background healing of incomplete tiles

---

## Testing Plan

1. **Unit tests**: LRU eviction, staleness detection, atomic writes
2. **Startup rebuild test**: Kill process mid-write, restart, verify index rebuilds correctly
3. **Cache hit test**: Fetch tile, restart, verify cache hit (no network)
4. **Eviction test**: Fill cache beyond limit, verify oldest entries evicted
5. **Memory pressure test**: Verify in-memory hot cache doesn't grow unbounded

---

## Performance Notes

| Cache | Avg Size per Entry | Compression | Disk I/O |
|-------|-------------------|-------------|----------|
| JPEG chunk (256×256) | ~50KB raw → ~20KB zstd | ~60% | Read on cache miss |
| DDS tile (4096×4096 BC1) | ~11MB raw → ~4MB zstd | ~65% | Read on cache miss |

- **Zstd level 3** is fast enough for background caching
- **In-memory hot cache**: 256 JPEG entries (~5MB) + 64 DDS entries (~250MB) for hot tiles
- **Startup index scan**: O(n) where n = number of cache files, ~1-2s for 10GB cache
- **Ephemeral approach**: Use OS temp directory for DDS cache (like autoortho4xplane), or persistent cache_dir for both

---

## BCn Compression (Performance Upgrade)

### Current State
`src/pipeline/compress.rs` uses a hand-rolled pure Rust BC1/BC3 compressor — correct but not optimized. The code itself notes: *"Can be replaced with ISPC texcomp FFI for higher throughput later."*

### Goal
Replace with **texpresso** v2 — a pure Rust BCn texture compression suite that is significantly faster than hand-rolled code, with no C dependencies.

### Crate
- **Package**: [`texpresso`](https://crates.io/crates/texpresso) v2.0.2
- **License**: MIT
- **Pure Rust**: Yes — no C or system library dependencies
- **Formats**: BC1 (DXT1), BC3 (DXT5), BC4, BC5, BC7, ETC1, ASTC

### Cargo.toml Changes
```toml
texpresso = "2.0"
# Keep compress.rs as fallback behind feature flag
```

### compress.rs Changes
```rust
// Before: hand-rolled compression
use crate::pipeline::compress::{compress_bc1_block, compress_bc3_block};

// After: texpresso
use texpresso::{bc1, bc3, CompressionConfig};

pub fn compress_image(data: &[u8], width: u32, height: u32, format: DdsFormat) -> Vec<u8> {
    let config = CompressionConfig::default();
    match format {
        DdsFormat::BC1 => bc1::compress(data, width, height, &config),
        DdsFormat::BC3 => bc3::compress(data, width, height, &config),
    }
}
```

### Benchmark Considerations
- texpresso is parallelized internally using rayon — each block compression can run concurrently
- The current hand-rolled code is single-threaded per call
- Expected improvement: 3-10x faster BCn compression with texpresso + rayon
- Important: don't double-parallelize (either texpresso internally or rayon at tile level, not both)

### Feature Flag Approach
```toml
[features]
default = ["texpresso"]
texpresso = ["dep:texpresso"]
compress-fallback = []  # keep hand-rolled compress.rs
```

### Implementation Order
1. Add `texpresso` to Cargo.toml
2. Benchmark current `compress_image()`: compress 100 tiles, measure total time
3. Update `compress.rs` to use texpresso behind feature flag
4. Benchmark with texpresso enabled, compare times
5. If improvement is significant, make texpresso the default
6. Consider removing hand-rolled code if texpresso is always used

### Notes
- ISPC texcomp (Intel's C/ISPC library) was considered but requires C compilation — excluded per pure-Rust requirement
- texpresso is the best pure-Rust option available as of 2026
- Its performance is competitive with ISPC texcomp for BC1/BC3 at quality settings appropriate for satellite imagery

---

## Edge Cases

| Case | Handling |
|------|---------|
| Cache directory deleted externally | `open()` handles gracefully, creates fresh index |
| Partial write (crash mid-write) | Temp file cleanup on startup, atomic rename prevents partial reads |
| Zoom level changed | `is_stale()` returns true, tile regenerated |
| Provider changed | JPEG cache invalidated (different key for maptype), OK |
| Disk full | `put()` returns error, tile still served in-memory for session |
| Corrupt JPEG in cache | Checksum verification on read, re-fetch if invalid |
| Very large cache (100GB+) | Index rebuild may be slow — consider async scan with progress |
