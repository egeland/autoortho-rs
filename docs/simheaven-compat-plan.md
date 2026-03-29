# SimHeaven Compatibility — Implementation Plan

## Overview

Enable AutoOrtho to coexist with SimHeaven X-World scenery by managing overlay visibility in `scenery_packs.ini`.

When SimHeaven compatibility is enabled, AutoOrtho disables its road/label overlays (`yAutoOrtho_Overlays`) while keeping the ortho imagery packs (`z_ao_*`) enabled. This allows SimHeaven's roads/labels to show through while retaining AutoOrtho ortho imagery as a fallback.

## Current Status: 📋 PLANNED

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

- [ ] Create `SimHeavenRegion` enum with mapping from/to Kubilus regions
- [ ] `check_simheaven_packages(xplane_dir, active_regions: &[Region]) -> Result<SimHeavenCheckResult>`:
  - Scan `scenery_packs.ini` for SimHeaven entries (both XP11 and XP12 patterns)
  - Return whether all active regions have SimHeaven packages
  - If any missing, return list of missing regions
- [ ] `apply_simheaven_compat(xplane_dir, enabled, active_regions: &[Region]) -> Result<()>`:
  - If enabled: require ALL active regions to have SimHeaven packages
    - If all present: disable `yAutoOrtho_Overlays` in scenery_packs.ini
    - If missing any: return error (don't modify anything)
  - If disabled: enable `yAutoOrtho_Overlays` in scenery_packs.ini

### Phase 2 — Config Integration

- [ ] Add `simheaven_compat: bool` to `Config` struct (default: `false`)
- [ ] Add to TOML serialization/deserialization

### Phase 3 — UI Integration

- [ ] Settings screen: Add checkbox with tooltip matching Python:
  - "Enable this if you are using SimHeaven scenery.\nThis will disable AutoOrtho Overlays to use the SimHeaven overlay instead. This is done by changing values within scenery_packs.ini.\nUse with caution, this may cause issues with other scenery packs."
- [ ] Apply on Settings save (via `refresh_scenery()` triggers `apply_simheaven_compat()`)
- [ ] Show warning message box if SimHeaven enabled but packages not found

### Phase 4 — Testing

- [ ] Unit tests for region mapping (Kubilus ↔ SimHeaven)
- [ ] Unit tests for `check_simheaven_packages()` with mocked ini
- [ ] Unit tests for `apply_simheaven_compat()` enable/disable
- [ ] Integration test: SimHeaven packages missing → returns error without modifying

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

## Files to Create/Modify

| File | Action |
|------|--------|
| `src/scenery/simheaven.rs` | **Create** — Core SimHeaven detection and compatibility logic |
| `src/scenery/mod.rs` | **Modify** — Add `pub mod simheaven;` |
| `src/config.rs` | **Modify** — Add `simheaven_compat` field |
| `src/ui/screens/settings.rs` | **Modify** — Add checkbox and status display |
| `PLAN.md` | **Modify** — Add section note |

---

## Dependencies

- No new crate dependencies required
- Reuses existing `scenery/packs_ini.rs` for reading/writing `scenery_packs.ini`
