# Configuration Reference

AutoOrtho stores its configuration in a `config.toml` file in the platform-specific config directory.

## Config File Location

| Platform | Path |
|----------|-------|
| Linux | `~/.config/autoortho/config.toml` |
| macOS | `~/Library/Application Support/autoortho/config.toml` |
| Windows | `%APPDATA%\autoortho\config.toml` |

## Editing Configuration

**Using the GUI (recommended):**
1. Launch AutoOrtho with `./autoortho --gui`
2. Navigate to Settings
3. Changes are saved automatically

**Manually:**
1. Open the config file in a text editor
2. Make changes and save
3. Restart AutoOrtho (or click "Reload Config" in Settings)

## All Configuration Options

### `[xplane]` - X-Plane Connection

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `xplane_path` | string | "" | Path to X-Plane folder (e.g., `/Applications/X-Plane 12`) |
| `xplane_host` | string | "127.0.0.1" | UDP host for X-Plane communication |
| `xplane_port` | integer | 49000 | UDP port for X-Plane RREF protocol |

### `[scenery]` - Tile and Scenery Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `tile_provider` | string | "ARC" | Tile provider ID (see Providers below) |
| `min_zoom` | integer | 10 | Minimum zoom level (higher = more detail, more data) |
| `max_zoom` | integer | 18 | Maximum zoom level |
| `cache_dir` | string | "~/.cache/autoortho" | Directory for tile cache storage |
| `disk_cache_size_gb` | integer | 50 | Maximum disk space for DDS tile cache |

### `[display]` - Visual Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable_night_exclusion` | boolean | true | Automatically switch to dark tiles at night |
| `night_threshold_deg` | float | -4.0 | Sun elevation threshold for night mode (degrees) |
| `season` | string | "Disabled" | Seasonal adjustment mode |
| `spring_saturation` | float | 1.0 | Saturation multiplier for Spring (0.0-2.0) |
| `summer_saturation` | float | 1.0 | Saturation multiplier for Summer (0.0-2.0) |
| `autumn_saturation` | float | 1.0 | Saturation multiplier for Autumn (0.0-2.0) |
| `winter_saturation` | float | 1.0 | Saturation multiplier for Winter (0.0-2.0) |

### `[fallback]` - Fallback System

The fallback system provides graceful degradation when satellite imagery is unavailable.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `level` | string | "Cache" | Fallback behavior when tiles are missing |
| `max_zoom_gap` | integer | 4 | Maximum zoom levels to downserve (1-8) |
| `cache_fallback` | boolean | true | Check cache for lower-zoom tiles |
| `solid_color` | array | [66, 77, 55] | RGB fallback color for solid tiles |

**Fallback Levels:**
- `Cache` - Use lower-zoom cached tiles if available
- `Downserve` - Scale from lower-resolution cached tiles
- `Network` - Download tiles on-demand (no fallback)
- `Solid` - Display solid color for missing tiles

### `[advanced]` - Performance Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `dds_memory_cache_mb` | integer | 256 | Memory limit for DDS tile cache |
| `chunk_memory_cache_mb` | integer | 512 | Memory limit for JPEG chunk cache |

## Tile Providers

AutoOrtho supports multiple tile providers. Each has different characteristics:

| ID | Name | Coverage | Quality | Notes |
|----|------|---------|---------|-------|
| `ARC` | ArcGIS | Global | High | Default, reliable |
| `GO2` | Google Maps | Global | Very High | Requires browser User-Agent |
| `BI` | Bing Maps | Global | High | Uses quadkey tiles |
| `NAIP` | USGS NAIP | US only | Medium-High | Aerial imagery |
| `USGS` | USGS Topo | US only | Medium | Topographic maps |
| `EOX` | EOX Sentinel | Global | Medium | Cloud-free Sentinel-2 |
| `FIREFLY` | Firefly | Global | High | Low-light optimized |

## Seasonal Modes

| Mode | Description |
|------|-------------|
| `Disabled` | No seasonal adjustment |
| `Spring` | Spring vegetation colors |
| `Summer` | Default summer imagery |
| `Autumn` | Autumn foliage colors |
| `Winter` | Desaturated winter colors |

Saturation values below 1.0 reduce color intensity; values above 1.0 increase it.

## Environment Variables

These can be set in your shell before running AutoOrtho:

| Variable | Values | Description |
|----------|--------|-------------|
| `RUST_LOG` | `error`, `warn`, `info`, `debug`, `trace` | Logging level |
| `RUST_BACKTRACE` | `0`, `1` | Enable/disable backtraces |

Example:
```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./autoortho --gui
```

## Full Example Configuration

```toml
[xplane]
xplane_path = "/Applications/X-Plane 12"
xplane_host = "127.0.0.1"
xplane_port = 49000

[scenery]
tile_provider = "ARC"
min_zoom = 10
max_zoom = 18
cache_dir = "~/.cache/autoortho"
disk_cache_size_gb = 50

[display]
enable_night_exclusion = true
night_threshold_deg = -4.0
season = "Disabled"
spring_saturation = 1.2
summer_saturation = 1.0
autumn_saturation = 0.9
winter_saturation = 0.7

[fallback]
level = "Cache"
max_zoom_gap = 4
cache_fallback = true
solid_color = [66, 77, 55]

[advanced]
dds_memory_cache_mb = 256
chunk_memory_cache_mb = 512
```

## Troubleshooting

**Config not saving?**
- Ensure the config directory exists
- Check file permissions

**Provider not working?**
- Some providers may be blocked in certain regions
- Try a different provider (ARC is most reliable)

**Memory issues?**
- Reduce `dds_memory_cache_mb` and `chunk_memory_cache_mb`
- Lower zoom range (reduce `max_zoom`)

**Missing tiles?**
- Increase `max_zoom_gap` in fallback settings
- Clear cache and re-download
