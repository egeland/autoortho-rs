# X-Plane Plugin — Implementation Plan

## Overview

Build a thin X-Plane plugin in Rust that runs inside the simulator, replacing the UDP-based dataref polling with direct XPLM SDK calls and handling scenery pack management programmatically.

**Architecture**:
```
X-Plane (plugin runs inside here)
├── X-Plane Plugin (Rust / xplm crate)
│   ├── Direct dataref reads/writes (XPLMDataAccess API)
│   ├── Scenery pack ordering (XPLMUtilities / file APIs)
│   └── IPC to autoortho-rs (Unix socket / shared memory / UDP relay)
│
└── autoortho-rs (separate process)
    ├── Tile fetching, DDS generation, FUSE mount
    ├── Receives position/datat refs from plugin
    └── Serves DDS via FUSE / shared memory
```

Plugin itself does **no heavy work** — it's purely a coordination layer. All CPU-intensive work stays in autoortho-rs.

Reference: [X-Plane Plugin Development](https://developer.x-plane.com/article/developing-plugins/)

---

## Why a Plugin?

### Replacing UDP Polling

Currently `xplane/dataref.rs` uses UDP broadcast to receive datarefs from X-Plane. Problems:
- UDP can be dropped, delayed, or blocked by firewalls
- Polling interval adds latency
- Requires X-Plane to have network access enabled

With a plugin, datarefs are read **directly from inside the simulator** — no network, no drops, no latency.

### Scenery Pack Management

Currently `scenery_packs.ini` ordering must be done manually or through the UI. A plugin can:
- Detect when scenery packs change
- Automatically reorder entries so autoortho directories are always in the correct position
- Handle the case where users install/remove scenery through other tools

### Sim Start Detection

Currently autoortho-rs guesses when X-Plane has fully loaded by watching for UDP packets. A plugin can:
- Send a reliable "X-Plane started" message via IPC
- Notify when scenery is reloaded
- Report when the aircraft is spawned

---

## Existing Code to Reference

- `src/xplane/dataref.rs` — current UDP-based dataref tracker (8 datarefs)
- `src/xplane/udp.rs` — current UDP client
- `src/scenery/` — scenery pack discovery and INI management

---

## Design

### 1. Project Structure

Create a separate Rust crate: `autoortho-xpplugin/`

```
autoortho-xpplugin/
├── Cargo.toml          # xplm crate, shared IPC types
├── src/
│   ├── lib.rs          # Plugin entry points (XPluginStart, etc.)
│   ├── datarefs.rs    # Direct XPLM dataref reads
│   ├── scenery.rs      # scenery_packs.ini management
│   ├── ipc.rs          # IPC to autoortho-rs
│   └── main.rs         # Only for testing/debug
├── build.rs            # Link against XPLM SDK
└── README.md
```

The plugin is built as a `.xpl` (macOS), `.dll` (Windows), or `.so` (Linux) and placed in X-Plane's `Resources/plugins/` directory.

### 2. XPLM SDK Bindings

Use the `xplm` crate (or generate bindings from the SDK headers):

```toml
[dependencies]
xplm = "0.3"  # Or generate from SDK headers using bindgen
```

The XPLM SDK provides:
- `XPLMDataRef` — opaque handle to a dataref
- `XPLMGetDataf/d/i` — read float/double/int datarefs
- `XPLMSetDataf/d/i` — write datarefs
- `XPLMFindDataRef` — find dataref by name
- `XPLMGetDirectoryContents` — enumerate scenery directories

If the `xplm` crate is incomplete, generate bindings with `bindgen` from the SDK headers.

### 3. Plugin Entry Points

```rust
use xplm::prelude::*;

#[no_mangle]
pub unsafe extern "C" fn XPluginStart(
    out_name: *mut c_char,
    out_sig: *mut c_char,
    out_desc: *mut c_char,
) -> i32 {
    // Copy plugin info
    strcpy_ptr(out_name, b"AutoOrtho\0".as_ptr() as *const _);
    strcpy_ptr(out_sig, b"com.autoortho.plugin\0".as_ptr() as *const _);
    strcpy_ptr(out_desc, b"X-Plane bridge for AutoOrtho satellite scenery\0".as_ptr() as *const _);

    // Initialize subsystems
    ipc::init();
    datarefs::init();
    scenery::init();

    1 // success
}

#[no_mangle]
pub unsafe extern "C" fn XPluginStop() {
    datarefs::deinit();
    ipc::deinit();
}

#[no_mangle]
pub unsafe extern "C" fn XPluginEnable() -> i32 {
    datarefs::start_polling(); // non-blocking, updates shared state
    ipc::connect();
    1
}

#[no_mangle]
pub unsafe extern "C" fn XPluginDisable() {
    datarefs::stop_polling();
    ipc::disconnect();
}

#[no_mangle]
pub unsafe extern "C" fn XPluginReceiveMessage(
    from: xplm::XPLMPluginID,
    msg: i32,
    param: *mut std::ffi::c_void,
) {
    match msg {
        xplm::MSG_SCENERY_LOADED => {
            scenery::on_scenery_loaded();
            ipc::send_message(IpcMsg::SceneryLoaded);
        }
        xplm::MSG_AIRFRAME_LOADED => {
            ipc::send_message(IpcMsg::AircraftLoaded);
        }
        _ => {}
    }
}
```

### 4. Direct Dataref Reads

Instead of UDP, poll datarefs internally and write to shared state:

```rust
mod datarefs {
    use xplm::prelude::*;
    use std::sync::Mutex;

    static STATE: Mutex<FlightData> = Mutex::new(FlightData::default());

    // Cached dataref handles (find once, use forever)
    static LAT_REF: std::sync::OnceLock<xplm::DataRef<f64>> = std::sync::OnceLock::new();
    static LON_REF: std::sync::OnceLock<xplm::DataRef<f64>> = std::sync::OnceLock::new();
    static ALT_REF: std::sync::OnceLock<xplm::DataRef<f64>> = std::sync::OnceLock::new();
    static SUN_REF: std::sync::OnceLock<xplm::DataRef<f32>> = std::sync::OnceLock::new();
    // ... 8 datarefs total

    pub fn init() {
        // Find datarefs once at startup
        LAT_REF.get_or_init(|| xplm::DataRef::find("sim/flightmodel/position/latitude").unwrap());
        LON_REF.get_or_init(|| xplm::DataRef::find("sim/flightmodel/position/longitude").unwrap());
        // ...
    }

    pub fn poll() {
        // Called every frame or every N frames (don't spam)
        let mut state = STATE.lock().unwrap();
        state.lat = LAT_REF.get().map(|r| r.get());
        state.lon = LON_REF.get().map(|r| r.get());
        // ...
    }
}
```

Key insight: read datarefs in the plugin's polling loop, write to shared memory. autoortho-rs reads from shared memory — zero network overhead.

### 5. IPC — Plugin to autoortho-rs

Three options, in order of preference:

**Option A: Unix socket (recommended)**
```
Plugin writes to /tmp/autoortho-xp.sock
autoortho-rs reads and acts on messages
```
Simple, cross-platform (works on macOS/Linux), no special permissions.

**Option B: Shared memory**
```
Plugin writes flight data to shared memory region
autoortho-rs mmap reads it
```
Lowest latency, but requires platform-specific code.

**Option C: UDP relay (easiest, backward compatible)**
```
Plugin sends UDP packets to 127.0.0.1:49001
autoortho-rs already handles UDP
```
Drop-in replacement — just change the sender from X-Plane's built-in UDP to the plugin. Minimal code change on autoortho-rs side.

For MVP, use Option C (UDP relay). The plugin acts as the UDP broadcaster it replaces.

```rust
mod ipc {
    pub enum IpcMsg {
        Position { lat: f64, lon: f64, alt: f64 },
        SceneryLoaded,
        AircraftLoaded,
        SimRunning,
    }

    pub fn send(msg: IpcMsg) {
        // Option C: UDP to existing autoortho-rs receiver
        // This means autoortho-rs code doesn't need to change at all
        static SOCKET: std::sync::OnceLock<udp::Socket> = std::sync::OnceLock::new();
        let sock = SOCKET.get_or_init(|| udp::Socket::bind("127.0.0.1:0").unwrap());
        let buf = serde_json::to_vec(&msg).unwrap();
        sock.send_to(&buf, "127.0.0.1:49001").ok();
    }
}
```

### 6. Scenery Pack Management

```rust
mod scenery {
    pub fn on_sceneryLoaded() {
        // Read current scenery_packs.ini
        // Check if autoortho directories are in correct order
        // If not, reorder them (move z_autoortho to bottom, etc.)
        // Write back
    }

    pub fn ensure_autoortho_order() {
        let ini_path = xplm::get_xplane_path("Custom Scenery") + "/scenery_packs.ini";
        let mut entries = parse_ini(&ini_path);

        // Find autoortho entries
        let autoortho: Vec<_> = entries.iter()
            .filter(|e| e.name.starts_with("z_ao_") || e.name.starts_with("yAutoOrtho") || e.name == "z_autoortho")
            .cloned()
            .collect();

        // Remove from current position
        entries.retain(|e| !is_autoortho_entry(e));

        // Insert at correct position: before zzz_global_scenery, after yAutoOrtho_Overlays
        let insert_before = entries.iter()
            .position(|e| e.name == "zzz_global_scenery")
            .unwrap_or(entries.len());

        for (i, entry) in autoortho.into_iter().enumerate() {
            entries.insert(insert_before + i, entry);
        }

        write_ini(&ini_path, &entries);
    }
}
```

### 7. Shared Types with autoortho-rs

Create a shared crate for IPC types:

```
autoortho-shared/
├── src/
│   └── lib.rs    # FlightData, IpcMsg, DatarefName enums
└── Cargo.toml
```

Both `autoortho-rs` and `autoortho-xpplugin` depend on this shared crate.

---

## Key Files

| File | Changes |
|------|---------|
| `autoortho-xpplugin/` | New crate — X-Plane plugin |
| `autoortho-shared/` | New crate — shared IPC types |
| `src/xplane/dataref.rs` | Keep for backward compat, add plugin mode that reads from shared mem |
| `src/xplane/udp.rs` | Keep for non-plugin mode |
| `PLAN.md` | Add plugin as future enhancement |

---

## Implementation Order

1. **[ ] Create `autoortho-shared/` crate** — define `FlightData`, `IpcMsg` types
2. **[ ] Create `autoortho-xpplugin/` crate** — basic plugin skeleton with XPluginStart/Stop/Enable/Disable/ReceiveMessage
3. **[ ] Implement dataref reads** — find and read 8 datarefs via XPLM, write to shared state
4. **[ ] Implement IPC** — UDP relay to existing autoortho-rs receiver (minimal change to autoortho-rs)
5. **[ ] Implement scenery management** — read/write scenery_packs.ini via XPLM file APIs
6. **[ ] Test plugin** — load in X-Plane, verify dataref values match UDP stream
7. **[ ] autoortho-rs: add plugin mode** — read from shared memory instead of UDP when plugin is detected
8. **[ ] Documentation** — update docs to mention plugin option

---

## Testing Plan

1. **Plugin loads**: Verify X-Plane loads the plugin without errors, shows in plugin manager
2. **Dataref accuracy**: Compare plugin dataref values vs UDP broadcast values (should be identical)
3. **Scenery ordering**: Install a scenery pack manually, verify plugin reorders scenery_packs.ini correctly
4. **IPC relay**: Verify autoortho-rs receives plugin messages via UDP (backward compatible with existing receiver)
5. **End-to-end**: Fly in X-Plane, verify autoortho tiles load correctly with plugin providing position data

---

## Compatibility

- X-Plane 11 and 12 compatible
- macOS, Windows, Linux
- Plugin and autoortho-rs can run independently (plugin mode is opt-in)

---

## Limitations

- Plugin is **optional** — autoortho-rs works standalone with UDP polling
- Plugin must be compiled separately for each platform (XPLM SDK linking)
- Plugin crashes = X-Plane crashes, so plugin code must be safe (no panics, no heavy computation)
- Cross-compiling for Windows from macOS/Linux requires cross-compiler toolchain

---

## Future Enhancements

- **Shared memory IPC** — replace UDP with shared memory for lower latency
- **Direct DDS serving** — plugin could use shared memory to hand DDS bytes directly to X-Plane without FUSE
- **Plugin configuration UI** — XPWidgets inside X-Plane to configure autoortho settings
- **Automatic plugin installation** — autoortho-rs could install the plugin on first run
