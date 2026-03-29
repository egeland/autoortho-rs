# SimBrief Integration — Implementation Plan

## Overview

Integrate SimBrief flight plan data to enhance Dynamic Zoom and Prefetching.

## Current Status: ✅ MOSTLY COMPLETE

### What's Implemented ✅

1. **Config** (`src/config.rs`)
   - `simbrief_user_id` field
   - `route_consideration_radius_nm`, `route_deviation_threshold_nm`, `route_prefetch_radius_nm`
   - `prefetch_route_percent`, `prefetch_airports`, `airport_radius_nm`, `near_airport_zoom`

2. **SimBrief Client** (`src/xplane/simbrief.rs`)
   - Full OFP parsing with `FlightPlan`, `FlightFix`, `PrefetchPoint`
   - SID/STAR gap handling (interpolates altitude through procedural procedures)
   - On-route detection and deviation checking
   - Airport field elevation lookup

3. **UI** 
   - Settings: User ID input with tooltips
   - Dashboard: Fetch button, route preview, expandable waypoint list

4. **Prefetcher** (`src/tiles/prefetch.rs`)
   - `prefetch_route()` method using SimBrief PrefetchPoints
   - Route config: percent_ahead, waypoint_radius_nm, airport_radius_nm

5. **Main Integration** (`src/main.rs`)
   - Auto-fetch on startup if user_id configured
   - Background prefetch loop with SimBrief route

### What's NOT Implemented ❌

- **Dynamic Zoom SimBrief mode** — `DynamicZoom` doesn't use flight plan waypoints for zoom selection. Currently uses only dataref altitude.
- **"Use Flight Data" toggle** — Not wired up. SimBrief is always used when loaded.

---

## Summary

SimBrief integration is functionally complete for prefetching. The DynamicZoom module doesn't yet use SimBrief waypoints for zoom level selection, which would be a nice enhancement.
