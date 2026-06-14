# User Guide

This guide covers how to use AutoOrtho after installation. For installation instructions, see [INSTALLATION.md](INSTALLATION.md).

## Getting Started

### First-Time Setup

1. **Launch AutoOrtho:**
   ```bash
   ./autoortho --gui
   ```

2. **Setup Wizard** will appear if no configuration exists:
   - Browse to your X-Plane folder
   - Select a tile provider (default: ArcGIS)
   - Configure zoom range (default: 10-18)
   - Click "Save & Continue"

3. **Configure X-Plane:**
   - Open X-Plane settings
   - Navigate to Scenery > Settings
   - Add the AutoOrtho mount path as a scenery pack
   - Place it at the top of the priority list

4. **Start AutoOrtho:**
   - Click "Start" in the AutoOrtho UI
   - The virtual filesystem will mount
   - X-Plane will now load satellite imagery

## User Interface

### Dashboard Screen

The main screen showing:
- **Connection Status**: Shows X-Plane connection state
- **Flight Info**: Current position, altitude, speed
- **Tile Progress**: Download progress and statistics
- **Web UI Link**: Quick access to web dashboard

### Settings Screen

Access via the gear icon or menu.

**X-Plane Settings:**
- X-Plane folder path
- UDP host and port
- SimBrief integration

**Scenery Settings:**
- Tile provider selection
- Zoom range (min/max)
- Cache directory
- Disk cache size

**Display Settings:**
- Night exclusion toggle
- Night threshold slider
- Season selection
- Season saturation sliders

**Fallback Settings:**
- Fallback level (Cache/Downserve/Network/Solid)
- Max zoom gap slider
- Cache fallback toggle

**Advanced Settings:**
- DDS memory cache size
- Chunk memory cache size
- Log level

### Developer Screen

For testing and diagnostics:
- Tile fetch tester (enter lat/lon/zoom)
- Fallback system tester
- Provider presets
- Cache management

### Web UI

Access at http://localhost:5847 (or configured port):

- **Flight Map** (`/map`): Live flight tracking on OpenStreetMap
- **Cache Viewer** (`/cache`): Browse downloaded tiles
- **Custom Map** (`/custommap`): Override providers per tile
- **Stats** (`/stats`): Performance metrics

## Common Tasks

### Changing Tile Provider

1. Go to Settings > Scenery
2. Select provider from dropdown
3. Settings save automatically
4. New tiles will use the new provider

### Adjusting Zoom Range

Lower zoom = less detail, faster loading
Higher zoom = more detail, more data

1. Settings > Scenery
2. Adjust min/max zoom sliders
3. Existing cache is preserved
4. New tiles will use new zoom range

### Enabling Night Mode

1. Settings > Display
2. Toggle "Night Exclusion" ON
3. Adjust "Night Threshold" if needed
   - Lower values = darker before switching
   - Default: -4.0 degrees

### Using Seasonal Adjustments

1. Settings > Display
2. Select season from dropdown
3. Adjust saturation per season (optional)
4. New tiles will have seasonal colors

### Prefetching a Route

1. Settings > SimBrief
2. Enter your User ID Number
3. Click "Fetch Route"
4. Go to Dashboard
5. Click "Prefetch Route"
6. Watch progress in tile stats

### Clearing Cache

**Via GUI:**
1. Developer > Cache Management
2. Click "Clear DDS Cache"
3. Confirm

**Via Web UI:**
1. Navigate to /cache
2. Use clear buttons

### Custom Map Overrides

Override the tile provider for specific regions:

1. Open Web UI at http://localhost:5847/custommap
2. Navigate to the area you want to override
3. Select a different provider
4. Save changes

## Advanced Features

### Dynamic Zoom

Automatically adjusts zoom based on altitude:
- Low altitude → Higher zoom (more detail)
- High altitude → Lower zoom (wider area)

Enable in Settings > Scenery > Dynamic Zoom

### Fallback System

Configure behavior for missing tiles:

| Level | Behavior |
|-------|----------|
| Cache | Use lower-zoom cached tiles |
| Downserve | Scale from cached lower-res tiles |
| Network | Download on-demand |
| Solid | Show solid color |

### Prefetch Routes

Pre-download tiles for SimBrief flight plans:

1. Enter SimBrief User ID
2. Fetch route
3. Click "Prefetch Route"
4. Tiles download in background

## Tips and Best Practices

### Performance

- **Start with lower zoom** (10-14) for faster initial loading
- **Use SSD** for cache storage
- **Close other bandwidth-heavy apps** during initial prefetch

### Storage

- **Monitor cache size** in Settings > Scenery
- **Use disk cache size limit** to prevent unbounded growth
- **Clear cache periodically** to free space

### Network

- **Stable connection required** for live tile fetching
- **Pre-fetch routes** before flights
- **Regional providers** may be faster (e.g., NAIP for US)

### X-Plane

- **Place AutoOrtho at top** of scenery priority
- **Reload scenery** after changing providers
- **Graphics settings** affect texture quality

## Troubleshooting

### X-Plane Not Connecting

**Symptom:** Dashboard shows "X-Plane not connected"

**Solutions:**
1. **Ensure X-Plane is running an active flight** — The RREF UDP protocol only works when a flight is loaded (not at the main menu, aircraft selection, or "Reading new scenery files" screen). Start a flight first.
2. **Check UDP port matches** (default: 49000) — Verify `xplane_port` in Settings matches X-Plane's UDP port.
3. **Verify firewall allows UDP** — On Windows, allow `autoortho.exe` through Windows Defender Firewall for both Private and Public networks. On macOS/Linux, ensure UDP port 49000 is not blocked.
4. **Restart AutoOrtho after X-Plane starts** — If AutoOrtho started before X-Plane, click "Restart" in the Dashboard or restart the app.
5. **Check `xplane_host` config** — If X-Plane is on a different machine, set `xplane_host` to that machine's IP (not `127.0.0.1`).
6. **Enable debug logging** — Run with `RUST_LOG=debug` to see connection attempts and timeout messages in the logs.

### No Imagery Showing

**Symptom:** Terrain without satellite overlay

**Solutions:**
1. Verify AutoOrtho is running (Dashboard green)
2. Check X-Plane scenery settings
3. Try different tile provider
4. Enable debug logging for errors

### Slow Tile Loading

**Symptom:** Blurry textures, tiles loading slowly

**Solutions:**
1. Check internet connection
2. Lower zoom range
3. Use nearby provider (NAIP for US)
4. Increase memory cache size
5. Clear cache if corrupted

### Memory Issues

**Symptom:** High RAM usage, crashes

**Solutions:**
1. Reduce memory cache settings
2. Lower zoom range
3. Close other memory-heavy apps

### Permission Denied (macOS/Linux)

**Symptom:** "Permission denied" errors

**Solutions:**
1. Install macFUSE/Dokan2 properly
2. Grant extension permissions in system settings
3. Run as standard user (not root)

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+,` | Open Settings |
| `Ctrl+R` | Reload Config |
| `Ctrl+Q` | Quit |

## Command-Line Options

```bash
./autoortho [OPTIONS]

Options:
  --gui              Launch desktop UI (default)
  --xplane <PATH>    Set X-Plane path and start
  --port <PORT>      Set web UI port (default: 5847)
  --help             Show help
```

## Getting Help

- **Issues**: https://github.com/egeland/autoortho-rs/issues
- **Discussions**: https://github.com/egeland/autoortho-rs/discussions
- **Logs**: Check console output or `~/.cache/autoortho/logs/`
