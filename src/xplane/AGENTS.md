# X-Plane Module

## Purpose

X-Plane integration: flight data tracking, SimBrief flight plan parsing, UDP communication.

## Ownership

- `dataref.rs` — `FlightDataStore` (thread-safe snapshot + averaging), `FlightData` builder, `FlightAverages`
- `udp_loop.rs` — `run_tracker()`, `connect_and_track()`, `datarefs` constants, reconnection loop
- `udp.rs` — UDP socket utilities
- `simbrief.rs` — `FlightPlan`, `FlightFix`, SimBrief XML parsing
- `mod.rs` — `RrefCodec`, re-exports
- `averagers.rs` — `FlightDataAverager`, `HeadingAverager`
- `traits.rs` — `FlightDataTracker`, `FlightPlanSource` traits
- `codec.rs` — `RrefCodec`, RREF packet encode/decode
- `simbrief_adapter.rs` — `SimBriefAdapter`

## Local Contracts

- `FlightDataTracker` trait is the shared interface — implemented by `FlightDataStore`
- `FlightDataStore` holds 5 averagers internally, updated on every `update()` call
- `run_tracker()` in `udp_loop.rs` calls `tracker.update(lat, lon, alt_agl_m, ...)` with named fields
- `FlightData` uses builder pattern: `FlightData::new().lat(45.0).lon(90.0)...`
- `RrefCodec::encode_request()` / `decode_response()` — RREF protocol

## Work Guidance

- `FlightDataStore` is the central position source — used by prefetch and web UI
- Consumers use `Arc<dyn FlightDataTracker>` — no concrete type dependency
- SimBrief parsing uses XML (quick-xml crate)
- UDP packets are fixed-size binary (little-endian)
- `udp_loop.rs` handles reconnection on connection loss

## Verification

- `cargo test --lib xplane`
- Unit tests for RREF encode/decode, FlightDataStore update/averaging, builder pattern

## Child DOX Index

None — flat module.
