# ADR: Extract FlightDataService from DatarefTracker

**Date:** 2026-06-25  
**Status:** Implemented  
**Supersedes:** N/A (new architecture decision)

---

## Context

`DatarefTracker` in `src/xplane/dataref.rs` is a god struct that bundles three concerns:
1. **UDP communication** — subscribes to X-Plane datarefs via UDP, runs reconnection loop
2. **Averaging** — maintains 5 sliding-window averagers (lat, lon, alt, hdg, spd)
3. **Thread-safe snapshot** — provides `get_flight_data()` and `get_averages()` to consumers

The struct holds 6 separate `RwLock`s. Understanding and testing any single concern requires understanding all three. Consumers (prefetch, webui, ui) only need the snapshot — they don't need UDP or averaging.

## Decision

**Split `DatarefTracker` into three focused modules:**

### 1. `FlightDataStore` (in `dataref.rs`)

Pure data holder with named-fields update:

```rust
pub struct FlightDataStore {
    data: Arc<RwLock<FlightData>>,
    lat_avg: Arc<RwLock<FlightDataAverager>>,
    lon_avg: Arc<RwLock<FlightDataAverager>>,
    alt_avg: Arc<RwLock<FlightDataAverager>>,
    hdg_avg: Arc<RwLock<HeadingAverager>>,
    spd_avg: Arc<RwLock<FlightDataAverager>>,
}

impl FlightDataStore {
    pub fn update(
        &self,
        lat: f64,
        lon: f64,
        alt_agl_m: f32,
        heading: f32,
        ground_speed_mps: f32,
        local_time_sec: f32,
        pressure_alt_ft: f32,
        sun_pitch: f32,
    ) { ... }

    pub fn get_flight_data(&self) -> FlightData { ... }
    pub fn get_averages(&self) -> Option<FlightAverages> { ... }
    pub fn mark_disconnected(&self) { ... }
    pub fn clear_averages(&self) { ... }
}
```

- Implements `FlightDataTracker` trait (same interface, no caller changes)
- Averagers updated internally on every `update()` call
- `FlightAverages` stays in `dataref.rs`
- `FlightData` becomes a builder pattern: `FlightData::new().lat(45.0).lon(90.0)...`

### 2. `udp_loop.rs` (new file in `xplane/`)

Extracted from bottom of `dataref.rs`:

```rust
// xplane/udp_loop.rs
pub async fn run_tracker(
    tracker: Arc<dyn FlightDataTracker>,
    xplane_addr: SocketAddr,
    shutdown: tokio::sync::watch::Receiver<bool>,
) { ... }

async fn connect_and_track(
    tracker: &dyn FlightDataTracker,
    addr: SocketAddr,
    shutdown: &tokio::sync::watch::Receiver<bool>,
) -> Result<(), XPlaneError> { ... }
```

- Contains `datarefs` constants (LATITUDE, LONGITUDE, etc.)
- Contains `run_tracker()` and `connect_and_track()`
- Calls `tracker.update(lat, lon, alt_agl_m, ...)` with named fields

### 3. No new module for averagers

`FlightDataAverager` and `HeadingAverager` stay in `averagers.rs` — they're already extracted. They're used internally by `FlightDataStore`.

## What Changes

| File | Change |
|------|--------|
| `src/xplane/dataref.rs` | Remove `DatarefTracker`, add `FlightDataStore` + `FlightData` builder. Remove `run_tracker()` and `connect_and_track()`. Keep `FlightAverages`. |
| `src/xplane/udp_loop.rs` | **New.** Contains `run_tracker()`, `connect_and_track()`, `datarefs` constants. |
| `src/xplane/mod.rs` | Add `pub mod udp_loop;`. Re-export `FlightDataStore`. |
| `src/xplane/traits.rs` | No change. `FlightDataTracker` trait stays identical. |
| `src/main.rs` | Change `DatarefTracker::new()` → `FlightDataStore::new()`. Change `run_tracker` import path. |
| `src/app_context.rs` | Change `DatarefTracker::new()` → `FlightDataStore::new()`. |
| `src/webui/mod.rs` | No change (uses `Arc<dyn FlightDataTracker>`). |
| `src/ui/` | No change (uses `Arc<dyn FlightDataTracker>`). |

## What Doesn't Change

- `FlightDataTracker` trait — identical interface
- `FlightDataAverager` / `HeadingAverager` — stay in `averagers.rs`
- `RrefCodec` — stays in `codec.rs`
- `FlightPlan` / `FlightFix` — stay in `simbrief.rs`
- All consumers (prefetch, webui, ui) — no changes needed

## Benefits

- **Locality:** Each concern in one file. Testing `FlightDataStore` doesn't need UDP.
- **Leverage:** `FlightDataStore` usable by webui/prefetch without pulling UDP code.
- **Testability:** `FlightDataStore` trivially testable with `update()` calls. Builder pattern makes test setup explicit.
- **Readability:** `dataref.rs` shrinks from ~500 lines to ~250 lines. `udp_loop.rs` gets the ~180 lines of UDP logic.

## Test Strategy

- **Unit tests for `FlightDataStore`:** update, averaging, mark_disconnected, clear_averages, builder pattern
- **Integration test for `udp_loop`:** mock socket, verify tracker updates
- **Existing `FlightDataTracker` trait tests:** stay as-is, now test `FlightDataStore` impl

## Consequences

- All existing code using `Arc<dyn FlightDataTracker>` continues to work unchanged
- `DatarefTracker` name is retired; `FlightDataStore` is the new concrete type
- The `datarefs` constants move from `dataref.rs` to `udp_loop.rs` — update imports

## Revisit If

- A third averaging strategy appears (e.g., exponential moving average) — consider extracting averagers then
- The UDP protocol changes significantly — `udp_loop.rs` is already isolated
- Consumers need raw data without averaging — add `get_raw_data()` to `FlightDataStore`

---

*This ADR is a living document. Update when the split is implemented and lessons are learned.*
