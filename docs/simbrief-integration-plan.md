# SimBrief Integration — Implementation Plan

## Overview

Integrate SimBrief flight plan data to enhance Dynamic Zoom and Prefetching:
- **Dynamic Zoom**: Use planned AGL altitudes from waypoints instead of velocity-based predictions
- **Prefetching**: Prioritize tiles by SimBrief's calculated time-to-encounter along the flight path

Reference: [autoortho4xplane docs/performance.md](https://github.com/ProgrammingDinosaur/autoortho4xplane/blob/develop/docs/performance.md) (SimBrief Integration section)

---

## Existing Code

`src/xplane/simbrief.rs` already exists:
- `SimBriefClient` struct for fetching OFP via REST API
- `FlightPlan` struct with parsed waypoints, cruise altitude, aircraft type, estimated times
- `simbrief_fetch()` function: GET request to SimBrief API, parse XML response
- `SimBriefError` enum for error handling
- SID/STAR gap handling (interpolates altitude through procedural procedures)

`src/altitude_predictor.rs`:
- `AltitudePredictor` trait (exists in the Python, needs to be ported/used)
- Used by dynamic_zoom to predict altitude at tile arrival time

`src/dynamic_zoom.rs`:
- `DynamicZoom` struct with quality steps
- `zoom_for_distance()` — selects zoom level based on predicted altitude

`src/tiles/prefetch.rs`:
- `SpatialPrefetcher` struct
- Currently uses velocity-based prediction only

`src/config.rs`:
- `AppConfig` struct — needs `simbrief_user_id` field

`src/ui/`:
- iced UI screens in `src/ui/screens/`

---

## Design

### 1. Config Field

**File**: `src/config.rs`

Add to `AppConfig`:
```rust
pub struct SimBriefConfig {
    pub user_id: Option<String>,
    pub use_flight_data: bool,
    pub route_consideration_radius_nm: f64,   // default 50.0
    pub route_deviation_threshold_nm: f64,      // default 40.0
    pub route_prefetch_radius_nm: f64,          // default 40.0
}
```

Add to `AppConfig::default()`:
```rust
simbrief: SimBriefConfig {
    user_id: None,
    use_flight_data: false,
    route_consideration_radius_nm: 50.0,
    route_deviation_threshold_nm: 40.0,
    route_prefetch_radius_nm: 40.0,
},
```

### 2. SimBrief Client (simbrief.rs — existing, enhance)

**File**: `src/xplane/simbrief.rs`

Current `SimBriefClient::new()` takes `user_id`. Need to add:
- `fetch_flight_plan(&self, user_id: &str) -> Result<FlightPlan, SimBriefError>`
- `OcpfFlightPlan` struct (currently `FlightPlan`) with full OFP data:
  - `origin`, `destination`, `aircraft` (type, name)
  - `cruise_altitude_ft`
  - `waypoints: Vec<Waypoint>` — each with lat, lon, altitude_ft, name, is_airport, estimated_time
  - `estimated_total_time_min`

**Waypoint struct**:
```rust
pub struct Waypoint {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub altitude_ft: Option<f64>,  // None for enroute waypoints
    pub estimated_time_min: Option<f64>,
    pub is_airport: bool,
    pub is_sid_star: bool,
}
```

Parse SimBrief XML OFP response (same format as Python `simbrief.py`):
- Endpoint: `https://www.simbrief.com/api/xml.php?userid=<user_id>`
- Parse `<waypoint>` elements from `<route>` block
- SID: waypoints from origin to first enroute (procedural, interpolate altitude)
- STAR: waypoints from last enroute to destination (procedural, interpolate altitude)
- Use `estimated_time` from SimBrief's times array

**Errors to handle**:
- `InvalidCredentials` — user ID not found
- `NoFlightPlan` — no recent flight plan
- `ParseError` — malformed XML
- `NetworkError` — HTTP failure

### 3. AppState — Hold Loaded Flight Plan

**File**: `src/lib.rs` (or new `src/simbrief_integration.rs`)

Create `SimBriefState`:
```rust
pub struct SimBriefState {
    pub flight_plan: Option<Arc<FlightPlan>>,
    pub last_fetch: Instant,
    pub fetch_error: Option<String>,
}
```

Update `AppState` to include `simbrief: SimBriefState`.

### 4. SimBrief Settings Screen

**File**: `src/ui/screens/simbrief_settings.rs` (new)

Add a new tab/section for SimBrief in the Settings area.

**UI Elements**:
1. **SimBrief User ID** — text input field
   - Placeholder: "Enter your SimBrief User ID"
   - Helper text: "Find it in SimBrief → Settings → API/OFP Settings"
2. **Fetch Flight Data** — button
   - On click: call `SimBriefClient::fetch_flight_plan()`, show spinner
   - On success: display route summary (origin → destination, aircraft, cruise altitude)
   - On error: display error message inline
3. **Use Flight Data** — toggle switch (only visible when flight plan is loaded)
   - Label: "Use SimBrief flight data for Dynamic Zoom and Prefetching"
4. **Route Settings** (collapsible section, visible when toggle is on):
   - **Consideration Radius** — slider, 10-200 nm, default 50
   - **Deviation Threshold** — slider, 5-100 nm, default 40
   - **Prefetch Radius** — slider, 10-150 nm, default 40
   - Each with labels showing current value

**Layout**: Could be a subsection under Settings → Setup tab, or its own sub-tab.

**Integration with existing Settings**: Add SimBrief section to the existing Settings screen rather than a new screen.

### 5. Dynamic Zoom — SimBrief Mode

**File**: `src/dynamic_zoom.rs`

Modify `DynamicZoom::zoom_for_tile()` to accept optional `&FlightPlan` and current aircraft position:
```rust
pub fn zoom_for_tile_with_plan(
    &self,
    tile: &TileId,
    aircraft_pos: &GeoPoint,
    aircraft_agl_ft: f64,
    flight_plan: &FlightPlan,
    consideration_radius_nm: f64,
) -> u8
```

Algorithm (per autoortho4xplane):
1. Find waypoints within `consideration_radius_nm` of the tile's center
2. Use the **lowest AGL altitude** among nearby waypoints:
   - `lowest_msl = min(waypoint.altitude_ft for waypoints)`
   - `highest_ground = max(terrain_elevation_ft at waypoint)`
   - `agl = lowest_msl - highest_ground` (most conservative = highest quality)
3. If no waypoints within radius, fall back to DataRef-based prediction
4. Map predicted AGL to zoom level using quality steps

**Terrain elevation lookup**: Use existing `tiles/coords.rs` or a simple HGT tile fetch. For simplicity, could use a static terrain database or just use MSL altitude as upper bound.

### 6. Prefetcher — SimBrief Mode

**File**: `src/tiles/prefetch.rs`

Modify `SpatialPrefetcher` to accept optional `&FlightPlan`:
```rust
pub struct PrefetcherContext {
    pub flight_plan: Option<Arc<FlightPlan>>,
    pub aircraft_pos: GeoPoint,
    pub aircraft_time_min: f64,  // interpolated time along route
    pub prefetch_radius_nm: f64,
    pub lookahead_min: i64,
}
```

Algorithm (per autoortho4xplane):
1. Project aircraft position onto route → find current segment and interpolation factor
2. Interpolate aircraft's estimated time along route using SimBrief waypoint times
3. Walk forward along route at regular intervals (e.g., every 1 min of flight time)
4. At each point, calculate perpendicular distance to route (for `route_prefetch_radius_nm`)
5. Generate tiles within the prefetch corridor at appropriate zoom levels
6. Sort by time-to-encounter (ascending) for priority queue
7. Skip tiles behind aircraft or beyond lookahead window

**Off-route detection**:
- Calculate minimum distance from aircraft to any route segment
- If > `route_deviation_threshold_nm`, fall back to velocity-based prefetching

### 7. API — Fetch Flight Plan Endpoint

**File**: `src/webui/` (if web UI exists)

Add REST endpoint for fetching flight plan:
```
POST /api/simbrief/fetch
Body: { "user_id": "..." }
Response: { "success": true, "route": "... → ...", "aircraft": "...", "cruise_ft": 35000 }
       or { "success": false, "error": "..." }
```

This allows the web UI to also trigger a flight plan fetch.

### 8. CLI — --simbrief-fetch Flag

**File**: `src/main.rs`

Add optional flag to fetch flight plan on startup:
```rust
--simbrief-fetch    Fetch SimBrief flight plan on startup using saved user ID
--simbrief-user-id  SimBrief user ID (overrides config)
```

---

## Implementation Order

1. **[x] Config**: Add `simbrief_user_id` to `AutoOrthoConfig`
2. **[x] Client**: `simbrief.rs` already has `FlightFix`, full OFP parsing, prefetch points, on-route detection
3. **[x] State**: SimBrief state on `AppState` (fetching, route summary, fixes, show_details)
4. **[x] Settings UI**: SimBrief User ID Number input with tooltip in Settings
5. **[x] Dashboard UI**: Fetch button, route preview, expandable waypoint list with TOC/TOD highlighting, airport field elevations
6. **[x] Route Settings UI**: Sliders for consideration radius, deviation threshold, prefetch radius in Settings
7. **[ ] Dynamic Zoom**: Implement SimBrief-based zoom selection in `dynamic_zoom.rs`
8. **[ ] Prefetcher**: Implement SimBrief-based prefetching in `prefetch.rs`
9. **[ ] Toggle**: Wire up "Use Flight Data" toggle to enable/disable SimBrief mode
10. **[ ] API**: Add `/api/simbrief/fetch` endpoint (optional)
11. **[ ] CLI**: Add `--simbrief-fetch` flag (optional)
12. **[ ] Tests**: Unit tests for OFP parsing, integration tests for fetch + interpolate

---

## Key Files

| File | Changes |
|------|---------|
| `src/config.rs` | Add `SimBriefConfig` struct |
| `src/xplane/simbrief.rs` | Enhance with `Waypoint`, full OFP parsing, error handling |
| `src/lib.rs` | Add `SimBriefState` to `AppState` |
| `src/dynamic_zoom.rs` | Add SimBrief-based `zoom_for_tile_with_plan()` |
| `src/tiles/prefetch.rs` | Add SimBrief-based prefetching with `PrefetcherContext` |
| `src/ui/screens/settings.rs` | Add SimBrief settings section |
| `src/webui/` | Add `/api/simbrief/fetch` endpoint (optional) |
| `src/main.rs` | Add `--simbrief-fetch` CLI flag (optional) |

---

## Testing Plan

1. **Unit tests**: Parse sample SimBrief XML, verify waypoint extraction
2. **Mock tests**: Use recorded API responses for offline testing
3. **Integration test**: Fetch real flight plan (requires valid user ID)
4. **UI test**: Enter user ID, click fetch, verify route display
5. **End-to-end test**: Load flight plan, fly route in X-Plane, verify prefetch tiles match route

---

## Performance Considerations

- Flight plan is fetched once per session (or on-demand)
- Waypoint interpolation for SID/STAR is done once at load time
- Prefetch queue is regenerated each tick (cheap — waypoints are small)
- Off-route detection is O(n) in number of route segments (n is small, ~few hundred)

---

## Edge Cases

| Case | Handling |
|------|---------|
| No flight plan loaded | Fall back to DataRef-based prediction |
| Aircraft off-route (>threshold) | Fall back to DataRef-based prediction |
| Empty waypoint list | Treat as no flight plan |
| SimBrief API down | Show error, continue with DataRef fallback |
| Invalid user ID | Show "Invalid credentials" error |
| Very long route (>1000 waypoints) | Cap interpolation, log warning |
| Holding patterns | Use incorrect altitude (documented limitation) |
| Multi-leg flight | Only most recent plan used (documented limitation) |
