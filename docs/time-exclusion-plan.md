# Night Time Exclusion — Implementation Plan

## Overview

Automatically disable AutoOrtho satellite imagery during night hours in the simulator, redirecting DSF terrain reads to X-Plane's global scenery for better night visuals.

## Current Status: ✅ MOSTLY COMPLETE

### What's Implemented ✅

1. **TimeExclusion module** (`src/time_exclusion.rs`)
   - `TimeExclusion` struct with sun pitch calculation
   - `is_night()`, `is_day()`, `day_phase()` methods
   - Uses X-Plane's `sun_pitch_degrees` dataref

2. **Config** (`src/config.rs`)
   - `enable_night_exclusion`, `night_threshold`, `day_threshold`

3. **FUSE Filesystem integration** (`src/fuse/filesystem.rs`)
   - `night_exclusion: Arc<AtomicBool>` flag
   - Returns fallback DDS when active (solid color)

4. **Background task** (`src/main.rs`)
   - Polls sun_pitch from dataref tracker every 5 seconds
   - Updates exclusion flag

5. **UI**
   - Settings: toggle + threshold slider
   - Dashboard: status display

### What's NOT Implemented ❌

- **Decision preservation** — No preservation on disconnect
- **Active DSF protection** — LRU cache of recently served paths
- **Global scenery redirect** — Currently serves fallback DDS, not X-Plane's default DSFs
- **Default to exclusion** — Config option when sim time unavailable
- **Dashboard status** — Current state not shown in UI

---

## Summary

Core night exclusion is working. The filesystem returns fallback tiles during night. Remaining work is mostly polish/enhancements.
