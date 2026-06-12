# X-Plane Module

## Purpose

X-Plane integration: dataref tracking, SimBrief flight plan parsing, UDP communication.

## Ownership

- `dataref.rs` — `DatarefTracker`, RREF packet handling, position/altitude tracking
- `simbrief.rs` — `FlightPlan`, `FlightFix`, SimBrief XML parsing
- `udp.rs` — UDP socket for X-Plane communication
- `mod.rs` — `RrefCodec`, `FlightDataAverager`, `HeadingAverager`

## Local Contracts

- `RrefCodec::encode_request()` / `decode_response()` — RREF protocol
- `FlightDataAverager` — sliding window for smooth values
- `HeadingAverager` — circular average for 0–360° headings
- `DatarefTracker` is shared: used by `tiles::prefetch`, `webui`, `ui`

## Work Guidance

- `DatarefTracker` is the central position source — used by prefetch and web UI
- SimBrief parsing uses XML (quick-xml crate)
- UDP packets are fixed-size binary (little-endian)

## Verification

- `cargo test --lib xplane`
- Unit tests for RREF encode/decode and averaging

## Child DOX Index

None — flat module.
