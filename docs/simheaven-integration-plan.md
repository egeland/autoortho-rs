# SimHeaven Integration Plan (SUPERSEDED)

> **This plan has been superseded by [simheaven-compat-plan.md](simheaven-compat-plan.md)**

## Overview

This plan details how to implement SimHeaven compatibility mode in the Rust AutoOrtho project, mirroring the functionality in the Python version.

## Background

SimHeaven provides X-World scenery packages that include their own orthophoto overlays. When using SimHeaven scenery, AutoOrtho's built-in overlays (`yAutoOrtho_Overlays`) can conflict with SimHeaven's overlays. The SimHeaven compatibility mode disables AutoOrtho's overlays to allow SimHeaven's overlays to be used instead.

## Python Version Behavior

The Python version implements SimHeaven compatibility as follows:

1. **Configuration Setting**: A `simheaven_compat` boolean (default: `false`) in the config
2. **UI Checkbox**: A "SimHeaven compatibility mode" checkbox in Settings
3. **Logic**: When enabled:
   - Checks if required SimHeaven packages are present in `scenery_packs.ini`
   - Disables AutoOrtho's `yAutoOrtho_Overlays` entry in `scenery_packs.ini`
   - When disabled: Re-enables the AutoOrtho overlays entry

### Region Mapping

AutoOrtho region IDs are mapped to SimHeaven region names:

| AutoOrtho Region | SimHeaven Region |
|-----------------|------------------|
| `na`            | `America`        |
| `eur`           | `Europe`         |
| `asi`           | `Asia`           |
| `afr`           | `Africa`         |
| `aus_pac`       | `Australia-Oceania` |
| `sa`            | `America`        |

### SimHeaven Package Patterns

The Python version checks for these SimHeaven patterns in `scenery_packs.ini`:
- **XP11**: `Custom Scenery/simHeaven_X-{region_id}`
- **XP12**: `Custom Scenery/simHeaven_X-World_{region_id}`

---

## Implementation Plan

### Step 1: Add Region Mapper Module

**File**: `src/scenery/simheaven.rs` (new file)

Create a module to handle SimHeaven-specific logic:

```rust
// Region mapping from AutoOrtho IDs to SimHeaven names
pub fn map_region_to_simheaven(region_id: &str) -> Option<&str> {
    match region_id {
        "na" | "sa" => Some("America"),
        "eur" => Some("Europe"),
        "asi" => Some("Asia"),
        "afr" => Some("Africa"),
        "aus_pac" => Some("Australia-Oceania"),
        _ => None,
    }
}

// Check if a line contains a SimHeaven overlay entry
pub fn is_simheaven_overlay(line: &str) -> bool {
    line.contains("simHeaven_X-") || line.contains("simHeaven_X-World_")
}

// Check if a line is the AutoOrtho overlay entry
pub fn is_autoortho_overlay(line: &str) -> bool {
    line.contains("yAutoOrtho_Overlays")
}
```

### Step 2: Add `simheaven_compat` to Configuration

**File**: `src/config.rs`

Add the new configuration option:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoOrthoConfig {
    // ... existing fields ...
    
    /// Enable SimHeaven compatibility mode (disables AutoOrtho overlays)
    #[serde(default)]
    pub simheaven_compat: bool,
}
```

### Step 3: Add `apply_simheaven_compat` Function

**File**: `src/scenery/packs_ini.rs` or `src/scenery/simheaven.rs`

Implement the core logic to enable/disable AutoOrtho overlays:

```rust
/// Apply SimHeaven compatibility mode to scenery_packs.ini
///
/// When enabling SimHeaven compat: Disable AutoOrtho overlays
/// When disabling SimHeaven compat: Enable AutoOrtho overlays
///
/// Returns:
/// - Ok(true) if modifications were made
/// - Ok(false) if no modifications needed
/// - Err if SimHeaven packages are missing or other errors
pub fn apply_simheaven_compat(
    xplane_dir: &Path,
    enabled: bool,
    installed_regions: &[String],
) -> Result<CompatResult, CompatError> {
    // 1. Read scenery_packs.ini
    // 2. Check if required SimHeaven packages are present for installed regions
    // 3. If SimHeaven packages missing: return error with list of missing packages
    // 4. Find and modify AutoOrtho overlay entry
    // 5. Write back the modified file
}

pub struct CompatResult {
    pub modified: bool,
    pub disabled_overlays: Vec<String>,
    pub enabled_overlays: Vec<String>,
}

pub enum CompatError {
    MissingSimHeavenPackages(Vec<String>),
    IoError(std::io::Error),
    NoOverlayFound,
}
```

### Step 4: Add UI Controls

**File**: `src/ui/screens/settings.rs`

Add a checkbox for SimHeaven compatibility mode in the Settings screen:

```rust
// In the settings view function
let simheaven_compat_row = row![
    text("SimHeaven compatibility mode").width(Length::Fill),
    toggler(state.config.simheaven_compat)
        .on_toggle(Message::SetSimheavenCompat)
]
.spacing(8);

// Tooltip text:
// "Enable this if you are using SimHeaven scenery.
//  This will disable AutoOrtho Overlays to use the SimHeaven
//  overlay instead. This is done by modifying scenery_packs.ini."
```

**File**: `src/ui/mod.rs`

Add the new message variant:

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // ... existing messages ...
    SetSimheavenCompat(bool),
}
```

Add handler in the `update` function:

```rust
Message::SetSimheavenCompat(enabled) => {
    handlers::set_simheaven_compat(&mut self.state, enabled);
}
```

### Step 5: Add Handler

**File**: `src/ui/handlers.rs`

```rust
pub fn set_simheaven_compat(state: &mut AppState, enabled: bool) {
    state.config.simheaven_compat = enabled;
    state.config_changed = true;
}
```

### Step 6: Integrate with Scenery Refresh

**File**: `src/ui/screens/scenery.rs` or `src/ui/mod.rs`

When refreshing scenery or applying settings, call `apply_simheaven_compat` if the setting is enabled:

```rust
// After scenery refresh or when simheaven_compat setting changes
if state.config.simheaven_compat {
    let result = crate::scenery::simheaven::apply_simheaven_compat(
        Path::new(&state.config.xplane_path),
        true,
        &state.installed_regions,
    );
    
    match result {
        Ok(CompatResult { modified: true, .. }) => {
            log::info!("SimHeaven compatibility mode applied");
        }
        Err(CompatError::MissingSimHeavenPackages(packages)) => {
            state.error_message = Some(format!(
                "Missing SimHeaven packages: {}",
                packages.join(", ")
            ));
        }
        // ... handle other cases ...
    }
}
```

---

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `src/scenery/simheaven.rs` | Create | Region mapping and SimHeaven detection logic |
| `src/scenery/mod.rs` | Modify | Add `pub mod simheaven;` |
| `src/config.rs` | Modify | Add `simheaven_compat` field |
| `src/ui/mod.rs` | Modify | Add `SetSimheavenCompat` message and handler |
| `src/ui/handlers.rs` | Modify | Add `set_simheaven_compat` function |
| `src/ui/screens/settings.rs` | Modify | Add SimHeaven compatibility checkbox |
| `src/scenery/packs_ini.rs` | Modify | Optionally add helper functions |

---

## Testing Plan

1. **Unit Tests**:
   - Test `map_region_to_simheaven` with all region IDs
   - Test `is_simheaven_overlay` with various package names
   - Test `apply_simheaven_compat` with mock `scenery_packs.ini` content

2. **Integration Tests**:
   - Test enabling SimHeaven compat with SimHeaven packages present
   - Test enabling SimHeaven compat with missing SimHeaven packages (should error)
   - Test disabling SimHeaven compat re-enables AutoOrtho overlays

---

## Open Questions

1. Should we auto-detect if SimHeaven is installed and prompt the user to enable compat mode?
2. Should we handle both XP11 and XP12 SimHeaven naming patterns?
3. Should the apply happen automatically on settings change, or require a manual "Apply" button?
