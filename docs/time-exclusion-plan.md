# Night Time Exclusion — Implementation Plan

## Overview

Automatically disable AutoOrtho satellite imagery during night hours in the simulator, redirecting DSF terrain reads to X-Plane's global scenery for better night visuals.

Reference: [autoortho4xplane docs/performance.md](https://github.com/ProgrammingDinosaur/autoortho4xplane/blob/develop/docs/performance.md) (Time-Based Exclusion section)

---

## Current State

### What's Implemented
- `src/time_exclusion.rs` — `TimeExclusion` struct with sun pitch calculation and `is_night()`/`is_day()`/`day_phase()` methods
- `src/config.rs` — `enable_night_exclusion: bool`, `night_threshold: f32`, `day_threshold: f32` config fields
- `src/xplane/dataref.rs` — `sun_pitch` dataref from X-Plane (sim/graphics/scenery/sun_pitch_degrees)
- `src/ui/screens/settings.rs` — UI toggle + night threshold slider
- `src/ui/screens/dashboard.rs` — Dashboard status display

### What's Missing
- **DSF read interception** — FUSE filesystem never checks exclusion state
- **Global scenery redirect** — No logic to serve X-Plane global DSF files during exclusion
- **Sim time source** — Currently uses real-world `Local::now()`, not X-Plane's sim time
- **Decision preservation** — No handling of temporary disconnections (e.g., scenery reload)
- **Active DSF protection** — DSF files currently in use by X-Plane should not be redirected
- **State wiring** — `TimeExclusion` is never instantiated or used in the app

---

## Design

### 1. SimTimeSource — Get Time from X-Plane

**File**: `src/time_exclusion.rs` (or `src/xplane/time_source.rs`)

The Python autoortho4xplane uses `sim/time/local_time_sec` (seconds since midnight in sim time). We have this dataref in `dataref.rs`.

Create a `SimTimeSource` enum:
```rust
pub enum TimeSource {
    /// Use X-Plane's sim time (when connected and available)
    SimTime { local_time_sec: f64 },
    /// Fallback: use system real time (before flight starts)
    RealTime,
}

impl SimTimeSource {
    pub fn from_dataref(data: &FlightData) -> Self {
        if data.local_time_sec >= 0.0 {
            SimTime::SimTime { local_time_sec: data.local_time_sec }
        } else {
            SimTime::RealTime
        }
    }

    pub fn local_hour(&self) -> f64 {
        match self {
            SimTime::SimTime { local_time_sec } => (local_time_sec / 3600.0) % 24.0,
            SimTime::RealTime => Local::now().hour() as f64 + Local::now().minute() as f64 / 60.0,
        }
    }
}
```

Note: The current `TimeExclusion::current_sun_pitch()` uses `Local::now()` (real time). Need to replace with sim time or keep as fallback.

### 2. Sun Pitch from Sim Time

**File**: `src/time_exclusion.rs`

The sun pitch calculation should work from sim time (hour of day) rather than real time, since we're matching the simulator's day/night cycle.

Enhance `TimeExclusion`:
```rust
impl TimeExclusion {
    /// Calculate sun pitch for a given hour of day (0-24) and day of year (1-366)
    pub fn sun_pitch_at(hour: f64, day_of_year: u16, latitude: f64) -> f32 {
        // Solar declination: +23.5° at summer solstice, -23.5° at winter solstice
        let day_angle = (day_of_year as f64 - 81.0) * 360.0 / 365.0;
        let declination = 23.5 * day_angle.to_radians().sin();

        // Hour angle: 0 at noon, ±90 at sunrise/sunset, ±180 at midnight
        let hour_angle = (hour - 12.0) * 15.0;

        // Simplified altitude at equator (for now)
        let altitude = hour_angle.to_radians().cos() * declination;

        altitude as f32
    }

    /// Check exclusion state using sim time and X-Plane sun pitch
    pub fn check_exclusion(
        &self,
        sun_pitch: f32,
        enable_flag: bool,
        default_to_exclusion: bool,
    ) -> ExclusionState
    {
        match (enable_flag, sun_pitch >= 0.0) {
            (false, _) => ExclusionState::Active,
            (true, true) if sun_pitch > self.night_threshold => ExclusionState::Inactive,
            (true, false) => ExclusionState::Active,
            (true, _) => ExclusionState::Inactive,
        }
    }
}
```

Actually, the existing approach using X-Plane's `sun_pitch_degrees` dataref is better — it accounts for latitude and season automatically.

### 3. AppState — Hold Exclusion State

**File**: `src/lib.rs`

Update `AppState`:
```rust
pub struct AppState {
    // ... existing fields ...
    pub exclusion_state: Arc<AtomicExclusionState>,
    pub last_sun_pitch: Arc<Mutex<f32>>,
}

pub enum ExclusionState {
    Active,   // Night — use global scenery
    Inactive, // Day — use AutoOrtho imagery
}

use std::sync::atomic::{AtomicBool, Ordering};
pub struct AtomicExclusionState {
    active: AtomicBool,
    preserved: AtomicBool,
    preserved_timestamp: Mutex<Instant>,
}

impl AtomicExclusionState {
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn update(&self, sun_pitch: f32, threshold: f32) {
        let new_active = sun_pitch <= threshold;
        self.active.store(new_active, Ordering::SeqCst);
        self.preserved.store(false, Ordering::SeqCst);
    }

    /// Preserve current state during temporary disconnection
    pub fn preserve(&self) {
        let was_active = self.active.load(Ordering::SeqCst);
        self.preserved.store(true, Ordering::SeqCst);
        *self.preserved_timestamp.lock().unwrap() = Instant::now();
    }
}
```

### 4. FUSE Filesystem — Exclusion-Aware Reads

**File**: `src/fuse/filesystem.rs`

Modify `DdsFileSystem` to accept exclusion state:
```rust
pub struct DdsFileSystem {
    parser: DdsPathParser,
    fetcher: Arc<TileFetcher>,
    format: DFormat,
    dds_cache: Mutex<HashMap<String, Arc<Vec<u8>>>>,
    root: Option<std::path::PathBuf>,
    exclusion_state: Option<Arc<AtomicExclusionState>>,  // NEW
    night_threshold: f32,  // NEW
    global_scenery_path: Option<PathBuf>,  // NEW
}
```

Modify `read_dsf()` to check exclusion and redirect:
```rust
impl DdsFileSystem {
    /// Read a DSF file, redirecting to global scenery during night exclusion
    pub async fn read_dsf(&self, path: &str) -> Result<Arc<Vec<u8>>, FuseError> {
        // Check if exclusion is active
        let exclusion_active = self
            .exclusion_state
            .as_ref()
            .map(|s| s.is_active())
            .unwrap_or(false);

        if exclusion_active {
            // Redirect to global scenery DSF
            if let Some(redirected) = self.try_global_dsf(path) {
                return Ok(redirected);
            }
            // If no global DSF found, fall through to serving AutoOrtho DSF
            warn!("Global DSF not found for {}, serving AutoOrtho DSF", path);
        }

        // Normal: serve AutoOrtho DSF (or generate if not cached)
        self.read_dsf_internal(path).await
    }

    /// Try to read DSF from X-Plane's global scenery
    fn try_global_dsf(&self, path: &str) -> Option<Arc<Vec<u8>>> {
        let global_path = self.global_scenery_path.as_ref()?;
        let global_dsf = global_path.join(path.strip_prefix("terrain/")?);
        std::fs::read(&global_dsf).ok().map(Arc::new)
    }
}
```

### 5. FUSE Mount — Wire Up Exclusion

**File**: `src/fuse/mount.rs`

Pass exclusion state and global scenery path to `DdsFileSystem`:
```rust
impl AutoOrthoFilesystem {
    pub fn new(
        fetcher: Arc<TileFetcher>,
        config: &AutoOrthoConfig,
        exclusion_state: Arc<AtomicExclusionState>,
        xplane_root: PathBuf,
    ) -> Self {
        let global_scenery = xplane_root.join("Resources").join("default scenery").join("default data").join("DSE");
        
        let fs = DdsFileSystem::with_exclusion(
            fetcher,
            config.cache_dir.clone().into(),
            exclusion_state,
            config.night_threshold,
            global_scenery,
        );

        // ... rest of setup
    }
}
```

### 6. Global Scenery Path Detection

**File**: `src/fuse/platform.rs` (or new)

Detect X-Plane global scenery location:
- **macOS**: `X-Plane.app/Contents/Resources/default scenery/default data/DSE/`
- **Windows**: `X-Plane 12/Resources/default scenery/default data/DSE/`
- **Linux**: `X-Plane/Resources/default scenery/default data/DSE/`

```rust
pub fn xplane_global_dse_path(xplane_root: &Path) -> PathBuf {
    xplane_root
        .join("Resources")
        .join("default scenery")
        .join("default data")
        .join("DSE")
}

pub fn find_xplane_global_dse() -> Option<PathBuf> {
    // Try common X-Plane install locations
    // On macOS, check /Applications/X-Plane*.app
    // On Windows, check C:\Program Files\ and standard locations
    // On Linux, check ~/X-Plane* and /opt/X-Plane*
}
```

### 7. Config — Additional Fields

**File**: `src/config.rs`

Add:
```rust
pub struct AutoOrthoConfig {
    // ... existing fields ...
    
    // Time exclusion (already exists)
    pub enable_night_exclusion: bool,
    pub night_threshold: f32,
    pub day_threshold: f32,
    
    // NEW
    pub default_to_exclusion: bool,  // default: false
}
```

### 8. UI — Settings Enhancements

**File**: `src/ui/screens/settings.rs`

Already has:
- ✅ Enable/Disable toggle
- ✅ Night threshold slider

Add:
- [ ] **Default to Exclusion** toggle — assume exclusion active when sim time unavailable
- [ ] **Start time** and **End time** — time-based exclusion (optional, sun pitch is simpler)
- [ ] **Current status** display — show "Active" / "Inactive" / "Twilight" in dashboard

### 9. Decision Preservation During Scenery Reload

**File**: `src/lib.rs` (AppState update loop)

When X-Plane dataref connection is lost (UDP disconnect):
1. **Preserve** the current exclusion decision
2. Continue using preserved state until reconnection
3. On reconnection, use fresh sim time data

```rust
async fn update_loop(state: Arc<AppState>) {
    loop {
        if let Some(data) = state.dataref_tracker.latest().await {
            state.exclusion_state.update(data.sun_pitch, state.config.night_threshold);
        } else {
            // Connection lost — preserve current state
            state.exclusion_state.preserve();
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

### 10. Active DSF Protection (Safety)

When exclusion activates mid-flight, DSF files that X-Plane has already loaded should not be redirected — serving the AutoOrtho DSF for those files prevents terrain popping/glitches.

**Approach**: Use a sliding window of recently served DSF paths (e.g., last 100 reads) and do NOT redirect those during exclusion:
```rust
impl DdsFileSystem {
    recently_served: Mutex<LruCache<String, Instant>>,  // path → last access time
    
    pub async fn read_dsf(&self, path: &str) -> Result<Arc<Vec<u8>>, FuseError> {
        // Track this read
        self.recently_served.lock().unwrap().insert(path.to_string(), Instant::now());
        
        // Check exclusion
        if self.exclusion_active() {
            // Don't redirect recently served DSFs
            if self.recently_served.lock().unwrap().contains_key(path) {
                return self.read_dsf_internal(path).await;
            }
            // Try global scenery redirect
            if let Some(data) = self.try_global_dsf(path) {
                return Ok(data);
            }
        }
        self.read_dsf_internal(path).await
    }
}
```

---

## Key Files

| File | Changes |
|------|---------|
| `src/time_exclusion.rs` | Add `TimeSource`, enhance sun pitch calc, add `ExclusionState` |
| `src/lib.rs` | Add `AtomicExclusionState` to `AppState`, wire up update loop |
| `src/config.rs` | Add `default_to_exclusion` field |
| `src/fuse/filesystem.rs` | Add exclusion state, `try_global_dsf()`, redirect logic |
| `src/fuse/mount.rs` | Pass exclusion state + global scenery path to `DdsFileSystem` |
| `src/fuse/platform.rs` | Add `find_xplane_global_dse()` |
| `src/ui/screens/settings.rs` | Add "Default to Exclusion" toggle |
| `src/ui/screens/dashboard.rs` | Show current exclusion status |

---

## Implementation Order

1. **[x] Settings UI**: Night exclusion toggle, night/day threshold sliders (editable)
2. **[x] FUSE exclusion-aware reads**: AtomicBool on DdsFileSystem, returns fallback DDS when active
3. **[x] Fallback DDS builder**: `build_fallback_dds()` in dds.rs — solid-color DDS without RGBA allocation
4. **[x] X-Plane sun_pitch wiring**: Background task reads DatarefTracker, updates exclusion flag every 5s
5. **[ ] Decision preservation**: Preserve state on disconnect (not yet implemented)
6. **[ ] Active DSF protection**: LRU cache of recently served paths (not yet implemented)
7. **[ ] Global scenery redirect**: Serve X-Plane's default DSFs instead of blank tiles (deferred — fallback tiles work)
8. **[ ] Default to exclusion**: Config option to assume night until sim data available
9. **[ ] Dashboard status**: Show current exclusion state (Active/Inactive)
10. **[ ] Tests**: Integration test for night redirect
11. **[ ] Documentation**: Document in `docs/performance.md`

---

## Testing Plan

1. **Unit tests**: Verify sun pitch calculation matches known values (noon = 0° hour angle, etc.)
2. **Manual test**: Enable exclusion, fly at night in X-Plane, verify global scenery used
3. **Scenery reload test**: Enable exclusion, trigger "Reload Scenery" in X-Plane, verify state preserved
4. **End-to-end test**: Byte-compare DSF served during exclusion vs global DSF

---

## Edge Cases

| Case | Handling |
|------|---------|
| X-Plane not connected | Use real-time sun pitch or `default_to_exclusion` |
| Global DSF not found | Fall back to AutoOrtho DSF (log warning) |
| Mid-flight exclusion toggle | Active DSF protection prevents terrain popping |
| Sim time jumps (flight to different region) | Use fresh sim time on each check |
| Very high latitude (>66.5°) | Sun may not rise/set — `sun_pitch` handles this naturally |
| X-Plane time paused | Uses current sun pitch, exclusion tracks paused time |
