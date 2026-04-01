# SimHeaven Compatibility — Implementation Plan

## Overview

Enable AutoOrtho to coexist with SimHeaven X-World scenery by managing overlay visibility in `scenery_packs.ini`.

When SimHeaven compatibility is enabled, AutoOrtho disables its road/label overlays (`yAutoOrtho_Overlays`) while keeping the ortho imagery packs (`z_ao_*`) enabled. This allows SimHeaven's roads/labels to show through while retaining AutoOrtho ortho imagery as a fallback.

## Current Status: ✅ COMPLETE

### Reference: Python Implementation

The Python version (`~/Programming/autoortho4xplane/autoortho/`) implements this feature:

- **Config**: `simheaven_compat = False` in `[autoortho]` section
- **UI**: Checkbox in settings with tooltip explaining the feature
- **Core Logic** (`config_ui_qt.py:5894`):
  1. Reads `scenery_packs.ini`
  2. Checks for required SimHeaven packages (XP11: `simHeaven_X-{region}`, XP12: `simHeaven_X-World_{region}`)
  3. Maps Kubilus region IDs to SimHeaven region names via `mappers.py`
  4. Disables AutoOrtho ortho packs (`z_ao_*`) when SimHeaven packages present
  5. Re-enables AutoOrtho ortho packs when compatibility disabled
  6. Shows warnings if required SimHeaven packages missing

### Region Mapping

| Kubilus Region | SimHeaven Region |
|---------------|------------------|
| `na` | America |
| `eur` | Europe |
| `asi` | Asia |
| `afr` | Africa |
| `aus_pac` | Australia-Oceania |
| `sa` | America |

## Implementation Plan

### Phase 1 — Core Module (`src/scenery/simheaven.rs`)

- [x] Create `SimHeavenRegion` enum with mapping from/to Kubilus regions
- [x] `check_simheaven_packages(xplane_dir, active_regions: &[Region]) -> Result<SimHeavenCheckResult>`:
- [x] Scan `scenery_packs.ini` for SimHeaven entries (both XP11 and XP12 patterns)
- [x] Return whether all active regions have SimHeaven packages
- [x] If any missing, return list of missing regions
- [x] `apply_simheaven_compat(xplane_dir, enabled, active_regions: &[Region]) -> Result<()>`:
- [x] If enabled: require ALL active regions to have SimHeaven packages
- [x] If all present: disable `yAutoOrtho_Overlays` in scenery_packs.ini
- [x] If missing any: return error (don't modify anything)
- [x] If disabled: enable `yAutoOrtho_Overlays` in scenery_packs.ini

### Phase 2 — Config Integration

- [x] Add `simheaven_compat: bool` to `Config` struct (default: `false`)
- [x] Add to TOML serialization/deserialization

### Phase 3 — UI Integration

- [x] Settings screen: Add checkbox with tooltip matching Python:
- [x] Apply on Settings save (via `refresh_scenery()` triggers `apply_simheaven_compat()`)
- [x] Show warning message box if SimHeaven enabled but packages not found

### Phase 4 — Testing

- [x] Unit tests for region mapping (Kubilus ↔ SimHeaven)
- [x] Unit tests for `check_simheaven_packages()` with mocked ini
- [x] Unit tests for `apply_simheaven_compat()` enable/disable
- [x] Integration test: SimHeaven packages missing → returns error without modifying

---

## Detailed Design

### SimHeaven Package Patterns

X-Plane 11: `Custom Scenery/simHeaven_X-{region}`
X-Plane 12: `Custom Scenery/simHeaven_X-World_{region}`

### SimHeavenCheckResult

```rust
pub struct SimHeavenCheckResult {
    pub all_present: bool,                      // All active AutoOrtho regions have SimHeaven
    pub missing_regions: Vec<KubilusRegion>,    // Regions missing SimHeaven packages
}
```

### Overlay Detection

AutoOrtho overlay to manage:
- `yAutoOrtho_Overlays` — Road/label overlays (this is what's toggled)

Note: The `z_ao_*` ortho imagery packs are NOT touched — they remain enabled so AutoOrtho provides ortho imagery where SimHeaven isn't installed. SimHeaven's ortho imagery simply overrides AutoOrtho's in regions where both exist.

### Compatibility Logic

```
# Python behavior:
if simheaven_compat:
    if SimHeaven packages found for all active AutoOrtho regions:
        disable yAutoOrtho_Overlays (roads/labels)
    else:
        show warning, skip modifications
else:
    enable yAutoOrtho_Overlays
```

---

## Files Created/Modified

| File | Action | Status |
|------|--------|--------|
| `src/scenery/simheaven.rs` | Created | ✅ Complete |
| `src/scenery/mod.rs` | Modified | ✅ Complete |
| `src/config.rs` | Modified | ✅ Complete |
| `src/ui/screens/settings.rs` | Modified | ✅ Complete |
| `src/ui/mod.rs` | Modified | ✅ Complete |
| `src/ui/handlers.rs` | Modified | ✅ Complete |
| `PLAN.md` | Modified | ✅ Complete |

## Implementation Notes

The implementation uses a `KubilusRegion` enum with the following mapping to SimHeaven regions:
- `NorthAmerica`, `SouthAmerica` → `America`
- `Europe` → `Europe`
- `Asia` → `Asia`
- `Africa` → `Africa`
- `AustraliaPacific` → `Australia-Oceania`

The `apply_simheaven_compat()` function is called in `ui/mod.rs` during scenery refresh when `simheaven_compat` is enabled.

---

## Dependencies

- No new crate dependencies required
- Reuses existing `scenery/packs_ini.rs` for reading/writing `scenery_packs.ini`
