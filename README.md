# AutoOrtho Rust Rewrite

A pure Rust reimplementation of AutoOrtho, replacing the original Python/C codebase with idiomatic Rust for improved performance, safety, and maintainability.

## Architecture

### Core Components

**Phase 2 - Image Pipeline** (`src/pipeline/`)
- `decode.rs`: JPEG decoding with buffer pool management
- `dds.rs`: DDS header generation and mipmap chain sizing
- `cache.rs`: Zstd-compressed DDS disk caching
- `budget.rs`: LRU eviction for disk budget management
- `image.rs`: RGBA image manipulation (paste, fill operations)

**Phase 3 - Tile Engine** (`src/tiles/`)
- `coords.rs`: Web Mercator slippy tile conversions, lat/lon math
- `chunk.rs`: 256×256 tile chunk state machine (Missing→Fetching→Cached→Error)
- `tile.rs`: 4096×4096 tile assembly from 16×16 chunk grids
- `prefetch.rs`: Spatial prefetching with heading-based prioritization
- `provider.rs`: Pluggable tile source interface (Google Maps, Bing Maps, ArcGIS)
- `fetcher.rs`: Concurrent chunk fetching with RwLock-based caching

**Phase 4 - FUSE Filesystem** (`src/fuse/`)
- `mod.rs`: DDS path parser and virtual file size calculation
- `filesystem.rs`: Virtual filesystem operations (getattr, read, listdir)

**Phase 5 - X-Plane Integration** (`src/xplane/`)
- `mod.rs`: RREF protocol codec, flight data averager, heading circular mean
- `udp.rs`: Async UDP client for X-Plane communication

**Phase 6 - Ancillary Features**
- `seasons.rs`: Seasonal saturation adjustment (HSL color space)
- `time_exclusion.rs`: Sun elevation-based night exclusion
- `dynamic_zoom.rs`: Altitude-to-zoom scaling
- `altitude_predictor.rs`: Linear altitude interpolation
- `downloader.rs`: Scenery pack download manager (zip/7z support)
- `stats.rs`: Metrics accumulation with Arc<Mutex> for concurrency

**Phase 7 - Web UI** (`src/webui/`)
- WebServer stub for Axum-based REST API

**Phase 8 - Desktop UI** (`src/ui/`)
- DesktopUI stub for egui-based interface

## Configuration

AutoOrthoConfig supports INI-based configuration with sensible defaults:

```rust
let config = AutoOrthoConfig::default_config();
// config.mount_dir = "/mnt/autoortho"
// config.cache_dir = "~/.cache/autoortho"
// config.xplane_host = "127.0.0.1"
// config.xplane_port = 49000
// config.tile_provider = "GO2" (Google Maps)
// config.min_zoom = 10, max_zoom = 18
// config.enable_night_exclusion = true
```

## Usage

### Building

```bash
cargo build --release
```

### Running

```bash
./target/release/autoortho
```

The application will:
1. Load configuration
2. Initialize tile provider
3. Create tile fetcher with in-memory caching
4. Start virtual filesystem (pending platform-specific FUSE binding)
5. Listen for X-Plane UDP on configured host/port
6. Run until Ctrl+C shutdown

## Test Coverage

**168 tests total:**
- 156 unit tests across all modules
- 12 integration tests exercising full pipeline

Key test areas:
- Tile coordinate conversions (Web Mercator math)
- DDS compression and header generation
- Chunk state machine transitions
- X-Plane RREF protocol encoding/decoding
- Flight data averaging with sliding windows
- Heading circular averaging (0°/360° wrap-around)
- Time exclusion with day/night thresholds
- Seasonal saturation in HSL color space
- Provider factory pattern and URL generation

## Dependencies

| Purpose | Crate |
|---------|-------|
| Async runtime | tokio |
| Parallelism | rayon |
| HTTP | reqwest |
| Compression | zstd |
| LRU cache | lru |
| Config/INI | config-rs + serde |
| Web framework | axum (stub) |
| UI | egui (stub) |
| Image decoding | image |
| Error handling | thiserror |
| Date/time | chrono |
| Async traits | async-trait |

## Implementation Status

### Completed
- ✅ Configuration system (INI-based)
- ✅ JPEG decoding (image crate)
- ✅ DDS header generation with mipmap chains
- ✅ Web Mercator tile math and quadkey encoding
- ✅ Chunk state machine and fetching
- ✅ Tile provider interface with Google/Bing/ArcGIS implementations
- ✅ Concurrent tile fetching with RwLock caching
- ✅ Virtual filesystem path parsing and structure
- ✅ X-Plane RREF protocol codec and UDP client
- ✅ Flight data averaging (sliding window)
- ✅ Heading circular averaging
- ✅ Time exclusion (sun elevation-based)
- ✅ Dynamic zoom (altitude-based)
- ✅ Seasonal saturation (HSL adjustment)
- ✅ Stats tracking
- ✅ Comprehensive test suite

### Pending
- 🔄 Actual FUSE filesystem mounting (fuser crate or platform-specific)
  - Linux: libfuse via fuser
  - macOS: macFUSE via fuser (build issues in previous attempt)
  - Windows: WinFsp
- 🔄 BC1/BC3 DDS compression (currently placeholder)
  - Option A: FFI to existing libispc_texcomp binaries
  - Option B: Pure Rust BC3 implementation
- 🔄 Web UI routes (dashboard, stats, custom map)
- 🔄 Desktop UI implementation
- 🔄 X-Plane UDP receiver (currently client only)
- 🔄 SimBrief OFP API integration
- 🔄 GitHub Actions CI/CD pipeline
- 🔄 Release packaging and distribution

## Architecture Decisions

1. **Async/Await**: Uses tokio throughout for non-blocking I/O (HTTP, UDP, file operations)
2. **Trait-based Providers**: TileProvider trait allows swappable tile sources
3. **Arc<RwLock> for Caching**: Shared immutable fetch results with reader-writer locks
4. **Placeholder DDS Compression**: Structure is correct but actual BC1/BC3 implementation deferred
5. **No FUSE Binding Yet**: Filesystem interface defined but platform-specific mount logic pending
6. **State Machine for Chunks**: Clear transitions prevent invalid operations

## Performance Considerations

- **Parallel JPEG Decoding**: rayon for CPU-bound image operations
- **Concurrent HTTP Fetching**: tokio tasks for simultaneous tile downloads
- **Memory Pooling**: Pre-allocated buffer pools reduce allocation overhead
- **LRU Eviction**: Disk budget enforcement via lru crate
- **Zstd Compression**: Fast, high-ratio compression for disk cache

## Next Steps

1. **FUSE Integration**: Implement platform-specific filesystem mounting
2. **DDS Compression**: Add actual BC1/BC3 compression (FFI or pure Rust)
3. **UDP Listener**: Complete X-Plane flight data receiver
4. **Web UI**: Axum routes for map, stats, custom map editor
5. **Desktop UI**: egui window with setup wizard and diagnostics
6. **CI/CD**: GitHub Actions workflows for multi-platform builds
7. **Benchmarks**: Compare Rust performance vs. original Python/C

## References

- [Plan](https://github.com/ProgrammingDinosaur/autoortho-rs/blob/main/PLAN.md)
- Original Python/C source: [github.com/ProgrammingDinosaur/autoortho](https://github.com/ProgrammingDinosaur/autoortho)
- Web Mercator: [Wikipedia](https://en.wikipedia.org/wiki/Web_Mercator_projection)
- RREF Protocol: X-Plane UDP protocol documentation
- DDS Format: [Microsoft DirectDraw Surface Specification](https://docs.microsoft.com/en-us/windows/win32/direct3ddds/dx-graphics-dds)
