# WebUI Module

## Purpose

Browser-based dashboard: flight tracking map, stats, REST API, WebSocket, custom map editor.

## Ownership

- `mod.rs` — `WebState`, `PositionUpdate`, server startup, `WEB_UI_PORT`
- `routes.rs` — Axum route handlers, router definition
- `custommap.rs` — `CustomMapStore`, custom map cell CRUD

## Local Contracts

- Port: `WEB_UI_PORT = 5847`
- `WebState` holds: `StatsStore`, `FlightDataTracker` (trait object), `CustomMapStore`, `AutoOrthoConfig`
- WebSocket channel: `broadcast::Sender<PositionUpdate>`
- Custom map stored in OS config dir: `autoortho/custom_map.json`

## Work Guidance

- Routes defined in `routes.rs` — add new endpoints there
- `CustomMapStore` is loaded once at startup, persisted on mutation
- `PositionUpdate` broadcast to all connected WebSocket clients
- Shutdown watch sender must OUTLIVE the server: dropping it closes the
  channel and axum's graceful shutdown kills the server instantly (web UI
  dead on arrival — hit mount mode once, fixed in `run_with_mount`)

## Verification

- `cargo test --lib webui`
- Server starts on random port test
- Restart-after-shutdown test

## Child DOX Index

None — flat module.
