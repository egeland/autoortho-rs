[![crates.io](https://img.shields.io/crates/v/autoortho_lib.svg)](https://crates.io/crates/autoortho_lib)
[![crates.io](https://img.shields.io/crates/d/autoortho_lib.svg)](https://crates.io/crates/autoortho_lib)

# AutoOrtho Rust

A pure Rust reimplementation of AutoOrtho for X-Plane satellite scenery, providing high-performance tile-based imagery with cross-platform support.

## Features

- **Multiple Tile Providers**: Google Maps, Bing Maps, ArcGIS, USGS NAIP, USGS Topo, EOX, Firefly
- **Cross-Platform FUSE Filesystem**: Linux (libfuse), macOS (macFUSE), Windows (Dokan2)
- **Real-Time Updates**: Web UI with live flight tracking via WebSocket
- **Intelligent Caching**: DDS tile disk cache with LRU eviction and zstd compression
- **Dynamic Zoom**: Altitude-based zoom level adjustment
- **Fallback System**: Graceful degradation for missing tiles (cache lookup, downserve, solid color)
- **Night Exclusion**: Automatic night mode based on sun elevation
- **Seasonal Adjustments**: Spring, Summer, Autumn, Winter saturation modes
- **SimBrief Integration**: Import flight plans for route-based prefetching

## Architecture

### Core Components

**Image Pipeline** (`src/pipeline/`)

- `decode.rs`: JPEG decoding with buffer pool management
- `dds.rs`: DDS header generation and mipmap chain sizing
- `cache.rs`: Zstd-compressed DDS disk caching
- `budget.rs`: LRU eviction for disk budget management
- `image.rs`: RGBA image manipulation (paste, fill operations)

**Tile Engine** (`src/tiles/`)

- `coords.rs`: Web Mercator slippy tile conversions, lat/lon math
- `chunk.rs`: 256×256 tile chunk state machine (Missing→Fetching→Cached→Error)
- `tile.rs`: 4096×4096 tile assembly from 16×16 chunk grids
- `prefetch.rs`: Spatial prefetching with heading-based prioritization
- `provider.rs`: Pluggable tile source interface
- `fetcher.rs`: Concurrent chunk fetching with RwLock-based caching
- `fallback.rs`: Fallback system for missing tiles

**FUSE Filesystem** (`src/fuse/`)

- `filesystem.rs`: Virtual filesystem operations (getattr, read, listdir)
- `mount.rs`: Linux/macOS FUSE mounting via unifuse
- `mount_win.rs`: Windows Dokan2 mounting

**X-Plane Integration** (`src/xplane/`)

- `dataref.rs`: RREF protocol codec, flight data tracker
- `simbrief.rs`: SimBrief flight plan parsing and import
- `udp.rs`: Async UDP client for X-Plane communication

**Web UI** (`src/webui/`)

- REST API endpoints for configuration and stats
- WebSocket for live position updates
- Custom map tile provider editor
- Cache viewer

**Desktop UI** (`src/ui/`)

- Setup wizard for first-time configuration
- Settings screen for all options
- Dashboard with flight tracking and stats
- Developer tools for testing

## Configuration

AutoOrtho uses a `config.toml` file stored in the platform config directory:

- **Linux**: `~/.config/autoortho/config.toml`
- **macOS**: `~/Library/Application Support/autoortho/config.toml`
- **Windows**: `%APPDATA%\autoortho\config.toml`

### Key Configuration Options

```toml
[xplane]
xplane_path = "/path/to/X-Plane 12"
xplane_host = "127.0.0.1"
xplane_port = 49000

[scenery]
tile_provider = "ARC"
min_zoom = 10
max_zoom = 18
cache_dir = "~/.cache/autoortho"

[display]
enable_night_exclusion = true
season = "Disabled"

[logging]
log_rotation = "daily"  # Options: "daily", "hourly", "never"

[fallback]
level = "Cache"
max_zoom_gap = 4
cache_fallback = true
solid_color = [66, 77, 55]

[advanced]
dds_memory_cache_mb = 256
chunk_memory_cache_mb = 512
```

## Building

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- **macOS**: macFUSE 4.x (install via Homebrew)
- **Linux**: libfuse-dev (install via package manager)
- **Windows**: Dokan2 (download from [github.com/dokan-dev/dokany](https://github.com/dokan-dev/dokany))

### Build Commands

```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Usage

### Desktop UI Mode

```bash
./target/release/autoortho --gui
```

Launches the desktop UI with setup wizard, settings, and dashboard.

### Command-Line Mode

```bash
./target/release/autoortho --xplane /path/to/X-Plane
```

Runs without GUI, using configuration from config file.

### Environment Variables

- `RUST_LOG`: Set log level via tracing EnvFilter (e.g., `RUST_LOG=debug`)
- `RUST_BACKTRACE`: Enable backtraces (set to `1`)

### Log Files

Logs are written to a file by default (not stdout/stderr):

- **Linux**: `~/.local/share/autoortho/autoortho.log`
- **macOS**: `~/Library/Application Support/autoortho/autoortho.log`
- **Windows**: `%LOCALAPPDATA%\autoortho\autoortho.log`

Rotation is configured via `log_rotation` in `config.toml`.

## Test Coverage

**363+ tests** across all modules:

- Unit tests for tile coordinate math, DDS generation, chunk state machine
- Integration tests for full tile pipeline
- Protocol tests for X-Plane RREF codec
- Provider tests for all tile sources

## Dependencies

| Purpose | Crate |
|---------|-------|
| Async runtime | tokio |
| Parallelism | rayon |
| HTTP client | reqwest |
| Compression | zstd, texpresso |
| LRU cache | lru |
| Web framework | axum |
| WebSocket | tokio-tungstenite |
| UI framework | iced |
| Image decoding | image |
| Config | config-rs + serde |
| Error handling | thiserror |
| Date/time | chrono |
| Filesystem | unifuse, dokan |
| Platform detection | sysinfo |
| Logging | tracing, tracing-subscriber, tracing-appender |

## Platform Support

| Platform | Filesystem | Status |
|----------|------------|--------|
| macOS | macFUSE | ✅ Tested |
| Linux | libfuse | ✅ Tested |
| Windows | Dokan2 | ✅ Tested |

**Windows GUI Note**: Release builds (`--release`) use `windows_subsystem = "windows"` and will not show a console window. Debug builds retain the console for development. Logs are written to `%LOCALAPPDATA%\autoortho\autoortho.log`.

## Architecture Decisions

1. **Async/Await**: Uses tokio throughout for non-blocking I/O
2. **Trait-based Providers**: TileProvider trait allows swappable tile sources
3. **parking_lot RwLock**: Better performance than std Mutex for concurrent reads
4. **Single Tokio Runtime**: Shared runtime for all async components
5. **Broadcast Channel**: WebSocket clients receive position updates via broadcast
6. **Fallback Levels**: Cache → Downserve → Network → Solid for graceful degradation

## Performance

Key optimizations:

- Parallel JPEG decoding with rayon
- Concurrent HTTP fetching with bounded LRU caches
- Zstd compression for disk cache (3-5x ratio)
- Memory-bounded caches with configurable limits
- TCP keepalive for connection pooling

Benchmark results (2026-03-29):

| Operation | Time |
|-----------|------|
| BC1 compression (256×256) | ~540 µs |
| BC3 compression (256×256) | ~810 µs |
| JPEG decode (256×256) | ~290 ns |
| Coordinate conversion | ~16 ns |

## License

Licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- GPL-3.0 license ([LICENSE-GPL](LICENSE-GPL))

## References

- [Original AutoOrtho](https://github.com/ProgrammingDinosaur/autoortho)
- [Changelog](CHANGELOG.md)
