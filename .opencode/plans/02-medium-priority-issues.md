# Plan: Medium Priority Issues

## Overview
These issues impact code quality, maintainability, and long-term sustainability. They should be addressed after high-priority issues.

---

## Issue 1: Multiple Tokio Runtimes

### Impact
Performance - Creates multiple thread pools

### Location
- `src/main.rs:41,54,63`
- `src/ui/mod.rs:159`

### Problem
The application creates `Runtime::new()` multiple times, each creating its own multi-threaded runtime with separate thread pools. This wastes resources and can cause contention.

### Solution
Create a single shared runtime that's used by all components.

### Implementation Steps

1. **Create a unified runtime in `lib.rs`:**
```rust
// src/lib.rs
use tokio::runtime::Builder;

pub fn create_runtime() -> tokio::runtime::Runtime {
    Builder::new_multi_thread()
        .enable_all()
        .thread_name("autoortho-worker")
        .build()
        .expect("Failed to create Tokio runtime")
}
```

2. **Update `main.rs` to use shared runtime:**
```rust
// Create runtime once at startup
let runtime = autoortho_lib::create_runtime();

// Use it for all async operations
runtime.block_on(async {
    // All async code here
});
```

3. **Update UI to accept runtime from main:**
```rust
// In ui/mod.rs, change:
runtime: Arc<tokio::runtime::Runtime>,

// Accept runtime as parameter instead of creating new one
```

4. **Consider using `#[tokio::main]` macro** for simpler async main:
```rust
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // All code is already async
}
```

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

## Issue 3: No HTTP Client Sharing

### Impact
Performance - no connection reuse between providers

### Location
- `src/tiles/provider.rs:178-291`

### Problem
Each tile provider creates its own `reqwest::Client`:
```rust
pub fn create(name: &str) -> Option<Arc<dyn TileProvider>> {
    match name.to_uppercase().as_str() {
        "GO2" | "GOOGLE" => Some(Arc::new(GoogleMapsProvider::new())),  // New client
        "BI" | "BING" => Some(Arc::new(BingMapsProvider::new(None))),    // New client
        // ...
    }
}
```

### Solution
Create a shared HTTP client with connection pooling.

### Implementation Steps

1. **Create a shared client:**
```rust
// src/tiles/provider.rs

use std::sync::OnceLock;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("Mozilla/5.0 ...")
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client")
    })
}
```

2. **Update provider constructors:**
```rust
impl GoogleMapsProvider {
    pub fn new() -> Self {
        Self {
            client: http_client().clone(),  // Reuse shared client
        }
    }
}
```

3. **Consider per-host connection pools:**
```rust
// For more control, use a connection pool per host:
use std::collections::HashMap;
use std::sync::RwLock;

struct ConnectionPool {
    clients: RwLock<HashMap<String, reqwest::Client>>,
}

impl ConnectionPool {
    pub fn get(&self, host: &str) -> reqwest::Client {
        // Check pool first
        // Create if not exists
    }
}
```

---

## Issue 4: WinFSP `block_on()` in Callbacks

### Impact
Potential deadlock if runtime is saturated

### Location
- `src/fuse/mount_win.rs:120,166,208`

### Problem
Synchronous WinFSP callbacks call `block_on()` to bridge to async code:
```rust
let attr = tokio::runtime::Handle::current().block_on(self.fs.get_attr(&path_str));
```

### Solution
Use a dedicated runtime for the FUSE filesystem operations.

### Implementation Steps

1. **Create dedicated runtime for FUSE:**
```rust
// In mount_win.rs
use tokio::runtime::Builder;

struct FuseRuntime {
    handle: tokio::runtime::Handle,
    _runtime: tokio::runtime::Runtime,  // Keep alive
}

impl FuseRuntime {
    fn new() -> Self {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create FUSE runtime");
        let handle = runtime.handle().clone();
        Self { handle, _runtime: runtime }
    }
    
    fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.handle.block_on(future)
    }
}
```

2. **Store runtime in struct:**
```rust
struct AutoOrthoWinFsp {
    fs: Arc<DdsFileSystem>,
    runtime: Arc<FuseRuntime>,
    // ...
}
```

3. **Use runtime for all async calls:**
```rust
// Instead of:
let attr = tokio::runtime::Handle::current().block_on(...);

// Use:
let attr = self.runtime.block_on(...);
```

---

## Issue 5: Silent Error Handling with `let _ =`

### Impact
Debugging difficulty - errors go unnoticed

### Location
- `src/ui/mod.rs` - multiple locations (lines 531, 549, 579, 665, 804, etc.)
- `src/main.rs:491`

### Problem
```rust
let _ = shutdown_tx_clone.send(());
// Error silently ignored

chunk.set_fetching().ok();
// State transition failure silently ignored
```

### Solution
Log meaningful errors, or return them properly.

### Implementation Steps

1. **Create a logging macro for expected-but-not-fatal errors:**
```rust
macro_rules! log_ignore {
    ($expr:expr, $msg:expr) => {
        if let Err(e) = $expr {
            debug!("{}: {}", $msg, e);
        }
    };
    ($expr:expr) => {
        log_ignore!($expr, "Operation failed")
    };
}

// Usage:
log_ignore!(shutdown_tx_clone.send(()), "Failed to signal shutdown");
```

2. **For UI message sends, use warn! level:**
```rust
if let Err(e) = self.tx.send(message) {
    warn!("UI message send failed: {}", e);
}
```

3. **For state transitions, use debug!:**
```rust
// In chunk.rs
pub fn set_fetching(&mut self) -> Result<(), ChunkError> {
    if self.state != ChunkState::Missing {
        return Err(ChunkError::InvalidStateTransition);
    }
    self.state = ChunkState::Fetching;
    Ok(())
}

// In fetcher.rs, update callers:
if chunk.set_fetching().is_err() {
    debug!("Chunk already being fetched: {}", key);
    return Ok(None);  // Or continue to next iteration
}
```

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

## Issue 9: Duplicated Provider Lists

### Impact
Maintenance - easy to get out of sync

### Location
- `src/ui/screens/setup.rs:7`
- `src/tiles/provider.rs` (PROVIDER_IDS constant)

### Solution
Define provider list in one canonical location.

### Implementation Steps

1. **Move to tiles/provider.rs:**
```rust
// src/tiles/provider.rs

/// All supported tile provider IDs
pub const PROVIDER_IDS: &[&str] = &["ARC", "BI", "GO2", "NAIP", "USGS", "EOX", "FIREFLY"];

/// Provider metadata (name, URL pattern, etc.)
pub const PROVIDER_INFO: &[(ProviderId, ProviderMetadata)] = &[
    ("ARC", ProviderMetadata { name: "ArcGIS", url_pattern: "...", .. }),
    // ...
];
```

2. **Export from lib.rs:**
```rust
pub use tiles::provider::{PROVIDER_IDS, PROVIDER_INFO};
```

3. **Update all consumers:**
```rust
// In setup.rs, remove local PROVIDERS constant
// Use: crate::PROVIDER_IDS

// In settings.rs, use crate::PROVIDER_INFO
```

---

## Summary

| Issue | Impact | Effort | Priority | Status |
|-------|--------|--------|----------|--------|
| Multiple Tokio Runtimes | Performance | Medium | P1 | Pending |
| Lock Contention | Performance | Low | P1 | ✅ Done |
| No HTTP Client Sharing | Performance | Low | P2 | Pending |
| WinFSP block_on() | Deadlock risk | Medium | P2 | Pending |
| Silent Error Handling | Debugging | Low | P2 | Pending |
| Monolithic Message Enum | Maintainability | High | P2 | Pending |
| AppState Too Large | Maintainability | Medium | P2 | Pending |
| Long View Functions | Maintainability | Medium | P3 | Pending |
| Duplicated Provider Lists | Maintainability | Low | P3 | Pending |

**Note:** Some of these (Message enum, AppState, View functions) are significant refactors that would benefit from being done together with the Phase 8c UI improvements in PLAN.md.
