# AutoOrtho-RS

Pure Rust reimplementation of AutoOrtho: X-Plane satellite scenery, high-performance tile imagery, cross-platform.

## Language

**Tile**:
A 256×256 Web Mercator satellite image fetched from a tile provider. Multiple tiles are assembled into a DDS output tile. NOT a DSF tile.
_Avoid_: imagery tile, DSF tile

**DSF Tile**:
A 1×1 degree terrain tile in X-Plane's Distribution Scenery Format. Contains terrain mesh and object placements — not textures. Named by southwest corner coordinates (e.g., `+42-071.dsf`). AutoOrtho downloads scenery packs of DSF tiles; it does not generate them.
_Avoid_: terrain tile, scenery tile (ambiguous without "DSF")

**Scenery Pack**:
A regional collection of DSF tiles downloaded from the autoortho-scenery GitHub releases (e.g., `z_ao_na` = North America, `z_ao_eu` = Europe). Installed under `{cache_dir}/scenery/z_autoortho/`.
_Avoid_: scenery package (X-Plane's term for any folder under Custom Scenery), tile pack

**Chunk**:
One 256×256 JPEG image tile. 256 chunks (16×16 grid) are assembled into one DDS tile. An implementation detail of the assembly pipeline.
_Avoid_: subtile (Python AutoOrtho terminology)

**Mount**:
The FUSE (macOS/Linux) or Dokan (Windows) virtual filesystem exposed to X-Plane at `{xplane}/Custom Scenery/z_autoortho/`. Serves virtual DDS tiles on demand and passes real file requests through to the scenery data directory.

**Zoom Level**:
Web Mercator slippy tile zoom (0–28). Used for both fetching and rendering image tiles. Higher zoom = more detail, smaller ground coverage per tile.
_Avoid_: DSF zoom (DSF tiles define coverage, not zoom)

**Dynamic Zoom**:
Altitude-adaptive selection of fetch zoom level. At high altitude the aircraft cannot resolve fine detail, so a lower zoom (larger tiles, less data) is acceptable. Implementation: `DynamicZoom::zoom_for_altitude_agl()`.
_Avoid_: auto-zoom (too vague)

**Mipmap Level**:
Rendering quality multiplier for DDS generation. Mipmap 0 = max detail (max_zoom, 16×16 chunks); Mipmap 1 = max_zoom - 1 (8×8 chunks → 2048×2048 pixels), and so on.
_Avoid_: render zoom

**Tile Provider / Maptype**:
A tile imagery service identified by a short code. Codes: "BI" (Bing Maps), "GO2" (Google Maps), "ARC" (ArcGIS), etc. Used both as X-Plane's filesystem identifier in DDS paths (e.g., `+37-122_BI16.ter`) and as the provider config key.
_Avoid_: source (only used in Apple token service response parsing)

**Provider Override**:
A custom map's per-cell assignment of a different tile provider than the default `tile_provider` config. Allows specific geographic areas (defined by lat/lon cells) to use different imagery sources. Resolved at fetch time by `get_provider_for_tile()`.

**Upserving**:
Serving a higher-resolution cached tile when a lower-resolution tile was requested. The cache contains the high-res tile; the request asks for lower-res; the high-res is used. Improves quality over expected.
_Avoid_: supersampling (not accurate)

**Fallback**:
When a requested tile is not available, finding a lower-zoom (lower-resolution) cached tile instead. Used when a tile is missing due to network failure, not yet downloaded, or outside configured zoom range. The found tile may be visually degraded via upscaling.
_Avoid_: downserve (the scaling operation happens in the pipeline, not here)

**Assemble**:
Combining 256 JPEG chunks (16×16 grid) into a single DDS tile, including generating the mipmap chain and compressing to BC1/BC3 format. Entry point: `assemble_tile()` in `tiles/assembler.rs`.
_Avoid_: render, generate (too generic)

**Prefetch**:
Proactively fetching tiles ahead of the aircraft based on a SimBrief flight plan. `run_simbrief_prefetch()` in `main.rs` walks the route and enqueues tiles along the path before the aircraft reaches them.

**Buffer**:
In-memory holding area for recently-used tiles. Managed by `TileFetcher` via an LRU cache. Tiles in the buffer avoid a network round-trip.

**Cache**:
Persistent disk storage of compiled DDS textures. Managed by `DdsCache` in `pipeline/cache.rs`. Survives sessions; used for fallback resolution and upserving.

## Flagged Ambiguities

**`FallbackLevel::Downserve` should be renamed to `Blur`**:
The `Downserve` variant is documented as "scale from lower-resolution tile" but the scaling does not occur in `fallback.rs` — it happens later in `pipeline/image.rs` via `Image::upscale()` (nearest-neighbor, producing visible aliasing). "Blur" accurately describes the visual effect and distinguishes this level from the identical-behavior `Cache` variant. See: `src/tiles/fallback.rs:14`, `src/ui/screens/settings.rs:28`.

## Example Dialogue

> "The aircraft is approaching KJFK at 3,000 ft AGL. Dynamic Zoom selects ZL19. The fetcher requests 256 chunks at ZL19 for the surrounding area. The first chunk is a cache miss, so it goes to the network queue. While waiting, the FallbackSystem finds a ZL16 tile on disk (within max_zoom_gap=4) and the pipeline upscales it via nearest-neighbor — a Blur fallback. Meanwhile, SimBrief prefetch has already enqueued tiles along the upcoming route at ZL17."