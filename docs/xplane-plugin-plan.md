# X-Plane Plugin — Implementation Plan

## Overview

Build a thin X-Plane plugin in Rust that runs inside the simulator, replacing the UDP-based dataref polling with direct XPLM SDK calls and handling scenery pack management programmatically.

**Architecture:**
```
X-Plane (plugin runs inside here)
├── X-Plane Plugin (Rust / xplm crate)
│   ├── Direct dataref reads/writes
│   ├── Scenery pack ordering
│   └── IPC to autoortho-rs
└── autoortho-rs (separate process)
    ├── Tile fetching, DDS generation, FUSE mount
    └── Receives position/datarefs from plugin
```

## Current Status: NOT STARTED

This is a future enhancement plan. No implementation has been done yet.

## Why a Plugin?

### Replacing UDP Polling

Currently `xplane/dataref.rs` uses UDP broadcast to receive datarefs from X-Plane. Problems:
- UDP can be dropped, delayed, or blocked by firewalls
- Polling interval adds latency
- Requires X-Plane to have network access enabled

### Scenery Pack Management

Currently `scenery_packs.ini` ordering must be done manually. A plugin can automatically reorder entries.

### Sim Start Detection

Currently autoortho-rs guesses when X-Plane has fully loaded. A plugin can send reliable messages.

---

## Files to Create

| File | Description |
|------|-------------|
| `autoortho-xpplugin/` | New crate — X-Plane plugin |
| `autoortho-shared/` | New crate — shared IPC types |

## Implementation Order

1. Create `autoortho-shared/` crate — define `FlightData`, `IpcMsg` types
2. Create `autoortho-xpplugin/` crate — basic plugin skeleton
3. Implement dataref reads via XPLM
4. Implement IPC — UDP relay to existing receiver
5. Implement scenery management
6. Test plugin in X-Plane
7. autoortho-rs: add plugin mode

## Compatibility

- X-Plane 11 and 12 compatible
- macOS, Windows, Linux
- Plugin is optional — autoortho-rs works standalone with UDP polling
