# Config

## Purpose

Persistent configuration for AutoOrtho: file loading, defaults, validation, serialization.

## Structure

Split into domain-specific sub-modules for **navigability**, not depth. Config is one of those modules where maintainers think about *specific domains* (cache, flight, network), not "the config module" as a whole. Each sub-module is a coherent unit: struct + defaults + validate + tests.

| File | Scope |
|------|-------|
| `mod.rs` | Root types (`AutoOrthoConfig`, `TileConfig`, `ConfigSnapshot`), load/save, re-exports |
| `cache.rs` | `CacheConfig` — DDS/chunk disk and memory cache sizes |
| `flight.rs` | `FlightConfig` — SimBrief, prefetch, route parameters |
| `network.rs` | `NetworkConfig` — X-Plane connection, rate limiting |
| `night.rs` | `NightConfig` — sun pitch thresholds |
| `season.rs` | `SeasonConfig` — per-season saturation |
| `ui.rs` | `UiConfig` — scale, window position, debug mode, log rotation |

### Why split instead of a god object?

- Each sub-config is ~70–140 lines — small enough to hold in your head
- Related fields stay together (all cache settings in one place, not scattered)
- Tests live next to the code they exercise
- `cargo test config::cache` tests only cache logic
- Adding a new config domain = adding one file, not bloating a 500+ line monster

### Why not go further?

- Sub-modules are **shallow** (struct + defaults + validate + tests) — deeper splitting would be over-modularizing
- `AutoOrthoConfig` composes them via `#[serde(flatten)]` — flat TOML format, no nesting
- Re-exports (`pub use cache::CacheConfig`) keep caller imports clean: `use crate::config::CacheConfig`

## Key Types

- **`AutoOrthoConfig`** — root config struct, all fields flattened. `load()` / `save()` / `from_file()`.
- **`ConfigSnapshot`** — cloned subset of config fields (tile, night, flight, cache). Exists to avoid holding `RwLock` across async boundaries.
- **Sub-configs** (`CacheConfig`, `FlightConfig`, etc.) — each has `Default`, `Serialize`/`Deserialize`, and `validate()`.

## Work Guidance

- **Adding a field**: add to the relevant sub-config struct + default fn + (if needed) validate fn + test
- **Adding a new domain**: create a new sub-file, add struct + defaults + validate, wire into `AutoOrthoConfig` with `#[serde(flatten)]`, add to this index
- **Env overrides**: `AUTOORTHO_<FIELD>` with `__` separator (e.g. `AUTOORTHO_TILE__PROVIDER=BI`). Currently manual in `apply_env_overrides()` — if this grows, consider a derive macro or the `config` crate (blocked by `#[serde(flatten)]` incompatibility)
- **Validation**: sub-config `validate()` methods are called from `AutoOrthoConfig::validate()`. Add validation at the sub-config level, not the root.

## Verification

- `cargo test --lib` runs all config tests
- `cargo clippy --all-features` catches unused imports, dead code

## Child DOX Index

None — config sub-modules are shallow enough to not warrant their own AGENTS.md files.
