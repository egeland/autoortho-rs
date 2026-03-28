# Plan: Medium Priority Issues

## Overview
These issues impact code quality, maintainability, and long-term sustainability. They should be addressed after high-priority issues.

---

## ✅ Issue 1: Multiple Tokio Runtimes

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/lib.rs`: Added `create_runtime()` function that creates a multi-threaded runtime
- `src/main.rs`: Creates runtime once before calling `ui::run()`
- `src/ui/mod.rs`: 
  - Added `RUNTIME` global `OnceLock` to store the shared runtime
  - Modified `run()` to store the runtime in the global
  - Modified `AutoOrthoApp::new()` to retrieve runtime from global
  - Updated tests to set up test runtime

### Benefits
- Single thread pool shared across all async components
- Reduced resource overhead
- Better thread utilization
- Named threads for debugging ("autoortho-worker")

---

## ✅ Issue 2: Lock Contention in FUSE Filesystem

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/fuse/filesystem.rs`: Replaced `std::sync::Mutex` with `parking_lot::RwLock` for `dds_cache`
- Updated all 8 constructor initializations
- Updated all cache operations to use `.read()` and `.write()` methods
- Removed `.expect()` calls since parking_lot RwLock doesn't poison

### Benefits
- Non-poisoning: locks are automatically released on panic
- Better fairness and performance than std Mutex
- Ready for future concurrent read optimizations

---

## ✅ Issue 3: No HTTP Client Sharing

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/tiles/provider.rs`: Added `HTTP_CLIENT` and `GOOGLE_CLIENT` static `OnceLock` clients
- Created `http_client()` and `google_http_client()` functions for lazy initialization
- Updated all 7 providers to use shared clients:
  - Google uses special User-Agent client
  - All others share the default client with TCP keepalive
- Providers now hold `&'static reqwest::Client` instead of owned clients

### Benefits
- Connection pooling across all tile providers
- TCP keepalive for connection reuse (60s)
- Reduced memory footprint (one client vs seven)
- Faster subsequent requests (no new connection overhead)

---

## ✅ Issue 4: WinFSP `block_on()` in Callbacks

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/fuse/mount_win.rs`:
  - Added `runtime` field to `AutoOrthoWinFsp` struct
  - Updated `new()` to accept runtime handle parameter
  - Updated `mount()` to use the passed runtime handle
  - Changed all `tokio::runtime::Handle::current().block_on()` to `self.runtime.block_on()`

### Benefits
- Uses the shared multi-threaded runtime handle instead of creating a new one
- No more `Handle::current()` calls which could fail or deadlock
- Consistent with the unified runtime architecture

---

## ✅ Issue 5: Silent Error Handling with `let _ =`

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/webui/routes.rs`: Added warning log for import_json failures
- `src/tiles/fetcher.rs`: Changed `set_fetching().ok()` to `set_fetching().is_err()` with debug logging

### Remaining Acceptable Silent Ignores
- UI message sends (oneshot channels, receiver may have dropped)
- Config saves (already handled by UI state)
- Shutdown signal sends (already shutdown)
- Test code (intentionally ignoring expected errors)

---

## Issue 6: Monolithic Message Enum

### Impact
Maintainability - difficult to navigate and extend

### Location
- `src/ui/mod.rs:52-157` (100+ variants)

### Problem
Single `Message` enum with 100+ variants creates long match statements.

### Solution
Split into sub-enums or use traits for modular message handling.

### Implementation Steps

1. **Group related messages:**
```rust
// Screen navigation
#[derive(Debug, Clone)]
pub enum NavigationMessage {
    GoToScreen(Screen),
    // ...
}

// Setup wizard
#[derive(Debug, Clone)]
pub enum SetupMessage {
    SetXPlanePath(String),
    SetCacheDir(String),
    // ...
}

// Runtime control
#[derive(Debug, Clone)]
pub enum RuntimeMessage {
    StartServices,
    StopServices,
    // ...
}
```

2. **Use enum inheritance pattern:**
```rust
#[derive(Debug, Clone)]
pub enum Message {
    Navigation(NavigationMessage),
    Setup(SetupMessage),
    Runtime(RuntimeMessage),
    // ... standalone messages
}

impl From<NavigationMessage> for Message {
    fn from(msg: NavigationMessage) -> Self {
        Message::Navigation(msg)
    }
}
```

3. **Update the message handler:**
```rust
impl Application for AutoOrthoApp {
    fn update(&self, message: Message) -> Task<Self::Message> {
        match message {
            Message::Navigation(msg) => self.update_navigation(msg),
            Message::Setup(msg) => self.update_setup(msg),
            Message::Runtime(msg) => self.update_runtime(msg),
            // ...
        }
    }
}
```

---

## Issue 7: AppState Has Too Many Responsibilities

### Impact
Maintainability - unclear ownership boundaries

### Location
- `src/ui/state.rs`

### Problem
`AppState` mixes configuration, runtime status, UI state, and download progress.

### Solution
Split into focused state structs.

### Implementation Steps

1. **Define focused state types:**
```rust
// Config state (persisted)
#[derive(Debug, Clone)]
pub struct ConfigState {
    pub xplane_path: String,
    pub cache_dir: String,
    pub tile_provider: String,
    // ... other config fields
}

// Runtime services (not persisted)
#[derive(Debug, Clone)]
pub struct ServiceState {
    pub web_server: ServiceStatus,
    pub xplane_tracker: ServiceStatus,
    pub web_url: Option<String>,
}

// Download progress
#[derive(Debug, Clone)]
pub struct DownloadState {
    pub active_downloads: HashMap<String, DownloadProgress>,
    pub available_regions: Vec<SceneryRegionInfo>,
}
```

2. **Compose in AppState:**
```rust
pub struct AppState {
    pub config: ConfigState,
    pub services: ServiceState,
    pub downloads: DownloadState,
    pub ui: UiState,  // Screen, error messages, etc.
}
```

3. **Update serialization** to handle the split.

---

## Issue 8: Long View Functions

### Impact
Maintainability - hard to read and modify

### Location
- `src/ui/screens/settings.rs:588 lines`
- `src/ui/screens/dashboard.rs:240 lines`

### Solution
Extract sub-components and widgets.

### Implementation Steps

1. **Extract card/section components:**
```rust
// src/ui/screens/helpers.rs

pub fn settings_card<'a, T: 'a>(
    title: &'a str,
    content: Element<'a, Message>,
) -> Container<'a, Message> {
    container(
        column![
            text(title).size(16).weight(FontWeight::Bold),
            content,
        ]
        .spacing(8)
        .padding(16)
    )
    .style(container::rounded_box)
}

pub fn settings_row<'a>(
    label: &'a str,
    value: Element<'a, Message>,
) -> Row<'a, Message> {
    row![
        text(label).width(Length::FillPortion(1)),
        value.width(Length::FillPortion(2)),
    ]
}
```

2. **Extract reusable widget modules:**
```
src/ui/widgets/
├── src/lib.rs
├── src/toggle.rs      // Styled toggle switch
├── src/slider.rs      // Slider with label
├── src/path_input.rs  // Path input with browse button
└── src/status_badge.rs // Status indicator
```

3. **Break up settings view:**
```rust
// settings.rs
mod path_settings;
mod cache_settings;
mod network_settings;
mod ui_settings;

impl SettingsScreen {
    fn view(&self) -> Element<Message> {
        column![
            path_settings::view(&self.state),
            cache_settings::view(&self.state),
            network_settings::view(&self.state),
            ui_settings::view(&self.state),
        ]
        .scrollable()
        .into()
    }
}
```

---

## ✅ Issue 9: Duplicated Provider Lists

**Status: IMPLEMENTED** ✅

### Changes Made
- `src/ui/screens/setup.rs`: Removed local `PROVIDERS` constant and imported `PROVIDER_IDS` from `crate::tiles::provider`

### Provider Lists Now Unified
- `PROVIDER_IDS` defined once in `src/tiles/provider.rs`
- Used by: `setup.rs`, `settings.rs`, `developer.rs`

---

## Summary

| Issue | Impact | Effort | Priority | Status |
|-------|--------|--------|----------|--------|
| Multiple Tokio Runtimes | Performance | Medium | P1 | ✅ Done |
| Lock Contention | Performance | Low | P1 | ✅ Done |
| HTTP Client Sharing | Performance | Low | P2 | ✅ Done |
| WinFSP block_on() | Deadlock risk | Medium | P2 | ✅ Done |
| Silent Error Handling | Debugging | Low | P2 | ✅ Done |
| Duplicated Provider Lists | Maintainability | Low | P3 | ✅ Done |
| Monolithic Message Enum | Maintainability | High | P2 | Pending |
| AppState Too Large | Maintainability | Medium | P2 | Pending |
| Long View Functions | Maintainability | Medium | P3 | Pending |
| Duplicated Provider Lists | Maintainability | Low | P3 | Pending |

**Note:** Some of these (Message enum, AppState, View functions) are significant refactors that would benefit from being done together with the Phase 8c UI improvements in PLAN.md.
