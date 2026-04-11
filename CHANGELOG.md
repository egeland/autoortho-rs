# Changelog

## [0.6.22](https://github.com/egeland/autoortho-rs/compare/v0.6.21...v0.6.22) - 2026-04-11

### Added

- *(altitude-predictor)* optimize methods by removing unused parameters and adding early returns ([#149](https://github.com/egeland/autoortho-rs/pull/149))

## [0.6.21](https://github.com/egeland/autoortho-rs/compare/v0.6.20...v0.6.21) - 2026-04-10

### Added

- optimise some filesystem calls ([#147](https://github.com/egeland/autoortho-rs/pull/147))

## [0.6.20](https://github.com/egeland/autoortho-rs/compare/v0.6.19...v0.6.20) - 2026-04-10

### Added

- auto-release when builds work ([#143](https://github.com/egeland/autoortho-rs/pull/143))

### Fixed

- remove approval step now that 0 reviewers required ([#145](https://github.com/egeland/autoortho-rs/pull/145))

## [0.6.19](https://github.com/egeland/autoortho-rs/compare/v0.6.18...v0.6.19) - 2026-04-10

### Fixed

- run dist init to generate wix/main.wxs and fix MSI GUIDs
- enable MSI in cargo-dist to show in Downloads section ([#141](https://github.com/egeland/autoortho-rs/pull/141))

## [0.6.18](https://github.com/egeland/autoortho-rs/compare/v0.6.17...v0.6.18) - 2026-04-10

### Fixed

- correct path in wix/main.wxs to artifacts/autoortho.exe

## [0.6.17](https://github.com/egeland/autoortho-rs/compare/v0.6.16...v0.6.17) - 2026-04-10

### Fixed

- verify copy worked with Get-ChildItem ([#138](https://github.com/egeland/autoortho-rs/pull/138))

## [0.6.16](https://github.com/egeland/autoortho-rs/compare/v0.6.15...v0.6.16) - 2026-04-09

### Fixed

- use correct exe path target/distrib/autoortho-x86_64-pc-windows-msvc/ ([#136](https://github.com/egeland/autoortho-rs/pull/136))

## [0.6.15](https://github.com/egeland/autoortho-rs/compare/v0.6.14...v0.6.15) - 2026-04-09

### Fixed

- add debug listing to see where exe files are ([#134](https://github.com/egeland/autoortho-rs/pull/134))

## [0.6.14](https://github.com/egeland/autoortho-rs/compare/v0.6.13...v0.6.14) - 2026-04-09

### Fixed

- find exe dynamically and copy to target/distrib for MSI ([#132](https://github.com/egeland/autoortho-rs/pull/132))

## [0.6.13](https://github.com/egeland/autoortho-rs/compare/v0.6.12...v0.6.13) - 2026-04-09

### Fixed

- extract version from tag in MSI build step ([#130](https://github.com/egeland/autoortho-rs/pull/130))

## [0.6.12](https://github.com/egeland/autoortho-rs/compare/v0.6.11...v0.6.12) - 2026-04-09

### Fixed

- use full target name 'x86_64-pc-windows-msvc' in MSI build condition ([#128](https://github.com/egeland/autoortho-rs/pull/128))

## [0.6.11](https://github.com/egeland/autoortho-rs/compare/v0.6.10...v0.6.11) - 2026-04-09

### Other

- Fix/windows build 4 ([#126](https://github.com/egeland/autoortho-rs/pull/126))

## [0.6.10](https://github.com/egeland/autoortho-rs/compare/v0.6.9...v0.6.10) - 2026-04-09

### Fixed

- correct EULA accept command for WiX v7 ([#124](https://github.com/egeland/autoortho-rs/pull/124))

## [0.6.9](https://github.com/egeland/autoortho-rs/compare/v0.6.8...v0.6.9) - 2026-04-09

### Fixed

- use wix build (not compile), accept OSMF license for WiX v7 ([#122](https://github.com/egeland/autoortho-rs/pull/122))

## [0.6.8](https://github.com/egeland/autoortho-rs/compare/v0.6.7...v0.6.8) - 2026-04-09

### Other

- Fix/windows build 2 ([#120](https://github.com/egeland/autoortho-rs/pull/120))

## [0.6.7](https://github.com/egeland/autoortho-rs/compare/v0.6.6...v0.6.7) - 2026-04-08

### Fixed

- windows build ([#118](https://github.com/egeland/autoortho-rs/pull/118))

## [0.6.6](https://github.com/egeland/autoortho-rs/compare/v0.6.5...v0.6.6) - 2026-04-08

### Added

- auto-approve release PR before merging

### Fixed

- use prs_created and fromJSON to correctly access PR number

### Other

- add --nocapture to WiX build for debug output

## [0.6.5](https://github.com/egeland/autoortho-rs/compare/v0.6.4...v0.6.5) - 2026-04-08

### Added

- dynamically set WiX version from release workflow

### Fixed

- update WiX version and add Compressed=no to bypass candle error

## [0.6.4](https://github.com/egeland/autoortho-rs/compare/v0.6.3...v0.6.4) - 2026-04-08

### Fixed

- use RELEASE_TOKEN to trigger release.yml on tag push

## [0.6.3](https://github.com/egeland/autoortho-rs/compare/v0.6.2...v0.6.3) - 2026-04-08

### Added

- add auto release to CI, part 4 ([#113](https://github.com/egeland/autoortho-rs/pull/113))

## [0.6.2](https://github.com/egeland/autoortho-rs/compare/v0.6.1...v0.6.2) - 2026-04-08

### Added

- add auto release to CI, part 3 ([#111](https://github.com/egeland/autoortho-rs/pull/111))
- add auto release to CI, part 2 ([#109](https://github.com/egeland/autoortho-rs/pull/109))
- add auto release to CI ([#108](https://github.com/egeland/autoortho-rs/pull/108))
- bump version attempt 3 ([#107](https://github.com/egeland/autoortho-rs/pull/107))
- bump version attempt 2 ([#106](https://github.com/egeland/autoortho-rs/pull/106))
- minor update ([#105](https://github.com/egeland/autoortho-rs/pull/105))
- add request rate limiting for tile providers ([#102](https://github.com/egeland/autoortho-rs/pull/102))

### Fixed

- fix mut and duplicated code ([#100](https://github.com/egeland/autoortho-rs/pull/100))

### Other

- bump tokio-tungstenite from 0.28.0 to 0.29.0 ([#85](https://github.com/egeland/autoortho-rs/pull/85))
- bump actions/upload-artifact from 6 to 7 ([#83](https://github.com/egeland/autoortho-rs/pull/83))
- bump libc from 0.2.183 to 0.2.184 ([#84](https://github.com/egeland/autoortho-rs/pull/84))
- bump actions/download-artifact from 7 to 8 ([#82](https://github.com/egeland/autoortho-rs/pull/82))
- bump tokio from 1.50.0 to 1.51.0 ([#86](https://github.com/egeland/autoortho-rs/pull/86))
- more workflow cleanup ([#104](https://github.com/egeland/autoortho-rs/pull/104))
- fix workflows ([#103](https://github.com/egeland/autoortho-rs/pull/103))
- optimize fetcher to return Arc<Vec<u8>> instead of cloning ([#101](https://github.com/egeland/autoortho-rs/pull/101))
- bump zip from 8.4.0 to 8.5.0 ([#87](https://github.com/egeland/autoortho-rs/pull/87))
- release v0.6.1 ([#99](https://github.com/egeland/autoortho-rs/pull/99))

## [0.6.1](https://github.com/egeland/autoortho-rs/releases/tag/v0.6.1) - 2026-04-06

### Added

- enable Windows FUSE support using winfsp ([#81](https://github.com/egeland/autoortho-rs/pull/81))
- add WiX MSI installer support ([#71](https://github.com/egeland/autoortho-rs/pull/71))
- add Inno Setup Windows installer workflow ([#57](https://github.com/egeland/autoortho-rs/pull/57))
- add Windows MSI installer support ([#54](https://github.com/egeland/autoortho-rs/pull/54))
- add LRU disk cache eviction to DdsCache ([#53](https://github.com/egeland/autoortho-rs/pull/53))
- add release-plz for fully automated releases ([#51](https://github.com/egeland/autoortho-rs/pull/51))
- auto-merge release-please PRs ([#20](https://github.com/egeland/autoortho-rs/pull/20))
- add SimHeaven X-World compatibility support ([#11](https://github.com/egeland/autoortho-rs/pull/11))
- Implement WebSocket for live position updates
- Implement fallback system and fix security/memory issues
- Add Windows FUSE support using winfsp
- Add cross-platform FUSE support using unifuse
- implement dynamic zoom with altitude-based rules and upserving
- add cache viewer web UI to visualize cached DDS tiles
- add custom map tile provider support with per-cell overrides
- add 7z extraction and seasonal adjustment UI
- add criterion benchmarks for performance testing
- implement SimBrief route prefetch settings and UI
- wire night exclusion into FUSE filesystem
- add SimBrief route settings and Prefetch Route button placeholder
- two-column flight plan with TOC/TOD highlighting
- add expandable flight plan details on Dashboard
- add SimBrief integration — config, Settings UI, Dashboard fetch
- replace hand-rolled BCn compression with texpresso
- simplify path config — derive mount and install from X-Plane folder
- disable Start button when scenery_packs.ini not found
- update Scenery Install tooltip and warn if scenery_packs.ini missing
- improve path labels and add hover tooltips
- wire persistent DDS disk cache with Settings UI
- upgrade fuser 0.14 → 0.17
- switch reqwest to pure Rust TLS (rustls)
- add automated versioning and release binaries

### Fixed

- fix mut and duplicated code ([#100](https://github.com/egeland/autoortho-rs/pull/100))
- fix fuse on win ([#97](https://github.com/egeland/autoortho-rs/pull/97))
- make release-plz depend on cross-platform tests passing ([#94](https://github.com/egeland/autoortho-rs/pull/94))
- fix release workflow ([#93](https://github.com/egeland/autoortho-rs/pull/93))
- fix windows fuse issues ([#92](https://github.com/egeland/autoortho-rs/pull/92))
- use winfsp directly on Windows instead of unifuse ([#91](https://github.com/egeland/autoortho-rs/pull/91))
- unify FUSE mounting across all platforms using unifuse ([#90](https://github.com/egeland/autoortho-rs/pull/90))
- always checkout main, get version from Cargo.toml ([#73](https://github.com/egeland/autoortho-rs/pull/73))
- update installer workflow with Inno Setup ([#69](https://github.com/egeland/autoortho-rs/pull/69))
- use PowerShell filtering ([#66](https://github.com/egeland/autoortho-rs/pull/66))
- add GH_TOKEN to download step ([#65](https://github.com/egeland/autoortho-rs/pull/65))
- simplify using gh release view ([#63](https://github.com/egeland/autoortho-rs/pull/63))
- fix installer workflow ([#61](https://github.com/egeland/autoortho-rs/pull/61))
- fix download and NSIS script paths ([#59](https://github.com/egeland/autoortho-rs/pull/59))
- use NSIS instead of Inno Setup (more reliable) ([#58](https://github.com/egeland/autoortho-rs/pull/58))
- configure release-plz to skip crates.io publish ([#52](https://github.com/egeland/autoortho-rs/pull/52))
- regenerate release.yml to match cargo-dist dispatch-releases ([#50](https://github.com/egeland/autoortho-rs/pull/50))
- delete existing release before cargo-dist creates new one ([#47](https://github.com/egeland/autoortho-rs/pull/47))
- skip GitHub Release creation in release-please ([#45](https://github.com/egeland/autoortho-rs/pull/45))
- fix Windows compilation errors in main.rs ([#43](https://github.com/egeland/autoortho-rs/pull/43))
- use cargo-dist expanded workflow with dispatch-releases ([#41](https://github.com/egeland/autoortho-rs/pull/41))
- use cargo-dist dispatch-releases mode for automated builds ([#39](https://github.com/egeland/autoortho-rs/pull/39))
- recreate tag via GitHub API to trigger release build ([#37](https://github.com/egeland/autoortho-rs/pull/37))
- use repository_dispatch to trigger release build ([#35](https://github.com/egeland/autoortho-rs/pull/35))
- add gate job so workflow_dispatch works with reusable workflow ([#33](https://github.com/egeland/autoortho-rs/pull/33))
- use workflow_dispatch to trigger release build ([#30](https://github.com/egeland/autoortho-rs/pull/30))
- re-push tag to trigger release.yml instead of workflow_dispatch ([#28](https://github.com/egeland/autoortho-rs/pull/28))
- remove commit message filter from version.yml ([#26](https://github.com/egeland/autoortho-rs/pull/26))
- remove required input from release.yml workflow_dispatch ([#24](https://github.com/egeland/autoortho-rs/pull/24))
- trigger cargo-dist release build after release-please creates tag ([#22](https://github.com/egeland/autoortho-rs/pull/22))
- use RELEASE_TOKEN for release-please permissions ([#18](https://github.com/egeland/autoortho-rs/pull/18))
- correct version.yml workflow_run condition paths ([#17](https://github.com/egeland/autoortho-rs/pull/17))
- resolve Windows build errors (texpresso, winfsp types) ([#16](https://github.com/egeland/autoortho-rs/pull/16))
- correct winfsp 0.12 API for Windows build ([#15](https://github.com/egeland/autoortho-rs/pull/15))
- rewrite mount_win.rs for winfsp 0.12 API compatibility ([#14](https://github.com/egeland/autoortho-rs/pull/14))
- use stable Rust toolchain in cross-platform CI ([#13](https://github.com/egeland/autoortho-rs/pull/13))
- wire DDS in-memory cache size from config ([#12](https://github.com/egeland/autoortho-rs/pull/12))
- Address silent errors, WinFSP runtime, and duplicate providers
- Disable FUSE on Windows due to unifuse/winFSP incompatibility
- make FUSE dependency optional for Windows builds
- install macfuse on macOS builds
- add actions permission to release-please workflow
- add FUSE_NO_PKG_CONFIG for macOS builds
- make build job depend on check and test
- remove obsolete fuse feature flag from CI workflows
- update Leaflet to 1.9.4 and add XSS protection
- skip coverage check for cells with custom map override
- increase flight plan waypoint text size from 12 to 13
- show field elevation for airports in flight plan display
- use ICAO airport codes for SimBrief route preview, rename to User ID Number
- move scenery_packs.ini warning under X-Plane Folder input
- rename "Temp Downloads" to "Scenery Downloads"
- match Scenery path label sizes to Settings screen
- reduce config save log from info to debug
- combine release-please and binary builds into single workflow

### Other

- release v0.6.1 ([#98](https://github.com/egeland/autoortho-rs/pull/98))
- release v0.6.0 ([#96](https://github.com/egeland/autoortho-rs/pull/96))
- release v0.6.0 ([#95](https://github.com/egeland/autoortho-rs/pull/95))
- bump hyper from 1.8.1 to 1.9.0 ([#88](https://github.com/egeland/autoortho-rs/pull/88))
- extract hardcoded port 5847 to WEB_UI_PORT constant ([#79](https://github.com/egeland/autoortho-rs/pull/79))
- release v0.5.8 ([#77](https://github.com/egeland/autoortho-rs/pull/77))
- extract SimBrief prefetch logic to dedicated function ([#78](https://github.com/egeland/autoortho-rs/pull/78))
- replace manual CLI parsing with clap ([#76](https://github.com/egeland/autoortho-rs/pull/76))
- extract common initialization to AppContext ([#74](https://github.com/egeland/autoortho-rs/pull/74))
- add file listing to diagnose wix path issue ([#72](https://github.com/egeland/autoortho-rs/pull/72))
- add verbose debug output ([#67](https://github.com/egeland/autoortho-rs/pull/67))
- remove release-please, use cargo-dist alone for releases ([#48](https://github.com/egeland/autoortho-rs/pull/48))
- *(main)* release 0.5.8 ([#49](https://github.com/egeland/autoortho-rs/pull/49))
- *(main)* release 0.5.7 ([#46](https://github.com/egeland/autoortho-rs/pull/46))
- *(main)* release 0.5.6 ([#44](https://github.com/egeland/autoortho-rs/pull/44))
- *(main)* release 0.5.5 ([#42](https://github.com/egeland/autoortho-rs/pull/42))
- *(main)* release 0.5.4 ([#40](https://github.com/egeland/autoortho-rs/pull/40))
- *(main)* release 0.5.3 ([#38](https://github.com/egeland/autoortho-rs/pull/38))
- *(main)* release 0.5.2 ([#36](https://github.com/egeland/autoortho-rs/pull/36))
- *(main)* release 0.5.1 ([#34](https://github.com/egeland/autoortho-rs/pull/34))
- *(main)* release 0.5.0 ([#32](https://github.com/egeland/autoortho-rs/pull/32))
- *(main)* release 0.4.5 ([#31](https://github.com/egeland/autoortho-rs/pull/31))
- *(main)* release 0.4.4 ([#29](https://github.com/egeland/autoortho-rs/pull/29))
- *(main)* release 0.4.3 ([#27](https://github.com/egeland/autoortho-rs/pull/27))
- *(main)* release 0.4.2 ([#25](https://github.com/egeland/autoortho-rs/pull/25))
- *(main)* release 0.4.1 ([#23](https://github.com/egeland/autoortho-rs/pull/23))
- *(main)* release 0.4.0 ([#21](https://github.com/egeland/autoortho-rs/pull/21))
- *(main)* release 0.3.0 ([#19](https://github.com/egeland/autoortho-rs/pull/19))
- bump cargo-dist from 0.30.4 to 0.31.0 ([#7](https://github.com/egeland/autoortho-rs/pull/7))
- bump actions/checkout from 4 to 6 ([#4](https://github.com/egeland/autoortho-rs/pull/4))
- bump dtolnay/rust-toolchain from 1.85.0 to 1.100.0 ([#3](https://github.com/egeland/autoortho-rs/pull/3))
- bump libloading from 0.8.9 to 0.9.0 ([#8](https://github.com/egeland/autoortho-rs/pull/8))
- bump criterion from 0.5.1 to 0.8.2 ([#9](https://github.com/egeland/autoortho-rs/pull/9))
- bump tokio-tungstenite from 0.26.2 to 0.28.0 ([#10](https://github.com/egeland/autoortho-rs/pull/10))
- Simbrief dynamic zoom ([#5](https://github.com/egeland/autoortho-rs/pull/5))
- Add AGENTS.md with development workflow documentation
- Fix version.yml: wait for tests to pass before bumping
- Fix cross-platform workflow: remove unnecessary release build
- Optimize workflow: ci.yml now runs only on PRs
- Add enhanced CI/CD workflows
- Apply workflow best practices fixes
- Optimize GitHub workflows
- Add release-please workflow for auto version bumps
- Improve GitHub workflows with best practices
- Clean up GitHub workflows for consistency
- Set up cargo-dist for native installers
- Add Release workflow for GitHub releases
- Add cargo-dist plan step to CI workflow
- Add cargo-dist configuration for native installers
- Add Yandex and Apple Maps tile providers
- Add plans for missing tile providers and native installers
- Extract UI message handlers to separate module
- Config validation and UI handler extraction
- Standardize on parking_lot mutexes
- Fetch User-Agent at build time from Chrome releases
- Add input validation for parsed numeric values
- Force HTTPS for Bing and NAIP providers
- Update PLAN.md and apply cargo fmt
- Add builder pattern for DdsFileSystem
- R1 code quality fixes: cache eviction, clones, deduplication, dead code
- Add user guide
- Add configuration reference guide
- Fix duplicate entry in plan file
- Update README and add installation guide
- Consolidate multiple Tokio runtimes into one
- Share HTTP clients across tile providers
- Replace Mutex with RwLock in DdsFileSystem
- Combine CI and release into single workflow
- comment out winfsp until fully implemented
- fix formatting to pass CI
- simplify and update PLAN.md to reflect actual state
- update PLAN.md to reflect completed custom map integration
- cleanups
- remove optional fuse feature flag
- add plan to hide roads
- update PLAN.md and SimBrief plan with current progress
- update PLAN.md — mark SimBrief config+UI complete
- update PLAN.md with current session progress
- move scenery paths from Scenery screen to Settings
- *(main)* release 0.1.0
- opt into Node.js 24 for GitHub Actions
- Initial commit

## [0.6.0](https://github.com/egeland/autoortho-rs/releases/tag/v0.6.0) - 2026-04-06

### Added

- enable Windows FUSE support using winfsp ([#81](https://github.com/egeland/autoortho-rs/pull/81))
- add WiX MSI installer support ([#71](https://github.com/egeland/autoortho-rs/pull/71))
- add Inno Setup Windows installer workflow ([#57](https://github.com/egeland/autoortho-rs/pull/57))
- add Windows MSI installer support ([#54](https://github.com/egeland/autoortho-rs/pull/54))
- add LRU disk cache eviction to DdsCache ([#53](https://github.com/egeland/autoortho-rs/pull/53))
- add release-plz for fully automated releases ([#51](https://github.com/egeland/autoortho-rs/pull/51))
- auto-merge release-please PRs ([#20](https://github.com/egeland/autoortho-rs/pull/20))
- add SimHeaven X-World compatibility support ([#11](https://github.com/egeland/autoortho-rs/pull/11))
- Implement WebSocket for live position updates
- Implement fallback system and fix security/memory issues
- Add Windows FUSE support using winfsp
- Add cross-platform FUSE support using unifuse
- implement dynamic zoom with altitude-based rules and upserving
- add cache viewer web UI to visualize cached DDS tiles
- add custom map tile provider support with per-cell overrides
- add 7z extraction and seasonal adjustment UI
- add criterion benchmarks for performance testing
- implement SimBrief route prefetch settings and UI
- wire night exclusion into FUSE filesystem
- add SimBrief route settings and Prefetch Route button placeholder
- two-column flight plan with TOC/TOD highlighting
- add expandable flight plan details on Dashboard
- add SimBrief integration — config, Settings UI, Dashboard fetch
- replace hand-rolled BCn compression with texpresso
- simplify path config — derive mount and install from X-Plane folder
- disable Start button when scenery_packs.ini not found
- update Scenery Install tooltip and warn if scenery_packs.ini missing
- improve path labels and add hover tooltips
- wire persistent DDS disk cache with Settings UI
- upgrade fuser 0.14 → 0.17
- switch reqwest to pure Rust TLS (rustls)
- add automated versioning and release binaries

### Fixed

- fix fuse on win ([#97](https://github.com/egeland/autoortho-rs/pull/97))
- make release-plz depend on cross-platform tests passing ([#94](https://github.com/egeland/autoortho-rs/pull/94))
- fix release workflow ([#93](https://github.com/egeland/autoortho-rs/pull/93))
- fix windows fuse issues ([#92](https://github.com/egeland/autoortho-rs/pull/92))
- use winfsp directly on Windows instead of unifuse ([#91](https://github.com/egeland/autoortho-rs/pull/91))
- unify FUSE mounting across all platforms using unifuse ([#90](https://github.com/egeland/autoortho-rs/pull/90))
- always checkout main, get version from Cargo.toml ([#73](https://github.com/egeland/autoortho-rs/pull/73))
- update installer workflow with Inno Setup ([#69](https://github.com/egeland/autoortho-rs/pull/69))
- use PowerShell filtering ([#66](https://github.com/egeland/autoortho-rs/pull/66))
- add GH_TOKEN to download step ([#65](https://github.com/egeland/autoortho-rs/pull/65))
- simplify using gh release view ([#63](https://github.com/egeland/autoortho-rs/pull/63))
- fix installer workflow ([#61](https://github.com/egeland/autoortho-rs/pull/61))
- fix download and NSIS script paths ([#59](https://github.com/egeland/autoortho-rs/pull/59))
- use NSIS instead of Inno Setup (more reliable) ([#58](https://github.com/egeland/autoortho-rs/pull/58))
- configure release-plz to skip crates.io publish ([#52](https://github.com/egeland/autoortho-rs/pull/52))
- regenerate release.yml to match cargo-dist dispatch-releases ([#50](https://github.com/egeland/autoortho-rs/pull/50))
- delete existing release before cargo-dist creates new one ([#47](https://github.com/egeland/autoortho-rs/pull/47))
- skip GitHub Release creation in release-please ([#45](https://github.com/egeland/autoortho-rs/pull/45))
- fix Windows compilation errors in main.rs ([#43](https://github.com/egeland/autoortho-rs/pull/43))
- use cargo-dist expanded workflow with dispatch-releases ([#41](https://github.com/egeland/autoortho-rs/pull/41))
- use cargo-dist dispatch-releases mode for automated builds ([#39](https://github.com/egeland/autoortho-rs/pull/39))
- recreate tag via GitHub API to trigger release build ([#37](https://github.com/egeland/autoortho-rs/pull/37))
- use repository_dispatch to trigger release build ([#35](https://github.com/egeland/autoortho-rs/pull/35))
- add gate job so workflow_dispatch works with reusable workflow ([#33](https://github.com/egeland/autoortho-rs/pull/33))
- use workflow_dispatch to trigger release build ([#30](https://github.com/egeland/autoortho-rs/pull/30))
- re-push tag to trigger release.yml instead of workflow_dispatch ([#28](https://github.com/egeland/autoortho-rs/pull/28))
- remove commit message filter from version.yml ([#26](https://github.com/egeland/autoortho-rs/pull/26))
- remove required input from release.yml workflow_dispatch ([#24](https://github.com/egeland/autoortho-rs/pull/24))
- trigger cargo-dist release build after release-please creates tag ([#22](https://github.com/egeland/autoortho-rs/pull/22))
- use RELEASE_TOKEN for release-please permissions ([#18](https://github.com/egeland/autoortho-rs/pull/18))
- correct version.yml workflow_run condition paths ([#17](https://github.com/egeland/autoortho-rs/pull/17))
- resolve Windows build errors (texpresso, winfsp types) ([#16](https://github.com/egeland/autoortho-rs/pull/16))
- correct winfsp 0.12 API for Windows build ([#15](https://github.com/egeland/autoortho-rs/pull/15))
- rewrite mount_win.rs for winfsp 0.12 API compatibility ([#14](https://github.com/egeland/autoortho-rs/pull/14))
- use stable Rust toolchain in cross-platform CI ([#13](https://github.com/egeland/autoortho-rs/pull/13))
- wire DDS in-memory cache size from config ([#12](https://github.com/egeland/autoortho-rs/pull/12))
- Address silent errors, WinFSP runtime, and duplicate providers
- Disable FUSE on Windows due to unifuse/winFSP incompatibility
- make FUSE dependency optional for Windows builds
- install macfuse on macOS builds
- add actions permission to release-please workflow
- add FUSE_NO_PKG_CONFIG for macOS builds
- make build job depend on check and test
- remove obsolete fuse feature flag from CI workflows
- update Leaflet to 1.9.4 and add XSS protection
- skip coverage check for cells with custom map override
- increase flight plan waypoint text size from 12 to 13
- show field elevation for airports in flight plan display
- use ICAO airport codes for SimBrief route preview, rename to User ID Number
- move scenery_packs.ini warning under X-Plane Folder input
- rename "Temp Downloads" to "Scenery Downloads"
- match Scenery path label sizes to Settings screen
- reduce config save log from info to debug
- combine release-please and binary builds into single workflow

### Other

- release v0.6.0 ([#95](https://github.com/egeland/autoortho-rs/pull/95))
- bump hyper from 1.8.1 to 1.9.0 ([#88](https://github.com/egeland/autoortho-rs/pull/88))
- extract hardcoded port 5847 to WEB_UI_PORT constant ([#79](https://github.com/egeland/autoortho-rs/pull/79))
- release v0.5.8 ([#77](https://github.com/egeland/autoortho-rs/pull/77))
- extract SimBrief prefetch logic to dedicated function ([#78](https://github.com/egeland/autoortho-rs/pull/78))
- replace manual CLI parsing with clap ([#76](https://github.com/egeland/autoortho-rs/pull/76))
- extract common initialization to AppContext ([#74](https://github.com/egeland/autoortho-rs/pull/74))
- add file listing to diagnose wix path issue ([#72](https://github.com/egeland/autoortho-rs/pull/72))
- add verbose debug output ([#67](https://github.com/egeland/autoortho-rs/pull/67))
- remove release-please, use cargo-dist alone for releases ([#48](https://github.com/egeland/autoortho-rs/pull/48))
- *(main)* release 0.5.8 ([#49](https://github.com/egeland/autoortho-rs/pull/49))
- *(main)* release 0.5.7 ([#46](https://github.com/egeland/autoortho-rs/pull/46))
- *(main)* release 0.5.6 ([#44](https://github.com/egeland/autoortho-rs/pull/44))
- *(main)* release 0.5.5 ([#42](https://github.com/egeland/autoortho-rs/pull/42))
- *(main)* release 0.5.4 ([#40](https://github.com/egeland/autoortho-rs/pull/40))
- *(main)* release 0.5.3 ([#38](https://github.com/egeland/autoortho-rs/pull/38))
- *(main)* release 0.5.2 ([#36](https://github.com/egeland/autoortho-rs/pull/36))
- *(main)* release 0.5.1 ([#34](https://github.com/egeland/autoortho-rs/pull/34))
- *(main)* release 0.5.0 ([#32](https://github.com/egeland/autoortho-rs/pull/32))
- *(main)* release 0.4.5 ([#31](https://github.com/egeland/autoortho-rs/pull/31))
- *(main)* release 0.4.4 ([#29](https://github.com/egeland/autoortho-rs/pull/29))
- *(main)* release 0.4.3 ([#27](https://github.com/egeland/autoortho-rs/pull/27))
- *(main)* release 0.4.2 ([#25](https://github.com/egeland/autoortho-rs/pull/25))
- *(main)* release 0.4.1 ([#23](https://github.com/egeland/autoortho-rs/pull/23))
- *(main)* release 0.4.0 ([#21](https://github.com/egeland/autoortho-rs/pull/21))
- *(main)* release 0.3.0 ([#19](https://github.com/egeland/autoortho-rs/pull/19))
- bump cargo-dist from 0.30.4 to 0.31.0 ([#7](https://github.com/egeland/autoortho-rs/pull/7))
- bump actions/checkout from 4 to 6 ([#4](https://github.com/egeland/autoortho-rs/pull/4))
- bump dtolnay/rust-toolchain from 1.85.0 to 1.100.0 ([#3](https://github.com/egeland/autoortho-rs/pull/3))
- bump libloading from 0.8.9 to 0.9.0 ([#8](https://github.com/egeland/autoortho-rs/pull/8))
- bump criterion from 0.5.1 to 0.8.2 ([#9](https://github.com/egeland/autoortho-rs/pull/9))
- bump tokio-tungstenite from 0.26.2 to 0.28.0 ([#10](https://github.com/egeland/autoortho-rs/pull/10))
- Simbrief dynamic zoom ([#5](https://github.com/egeland/autoortho-rs/pull/5))
- Add AGENTS.md with development workflow documentation
- Fix version.yml: wait for tests to pass before bumping
- Fix cross-platform workflow: remove unnecessary release build
- Optimize workflow: ci.yml now runs only on PRs
- Add enhanced CI/CD workflows
- Apply workflow best practices fixes
- Optimize GitHub workflows
- Add release-please workflow for auto version bumps
- Improve GitHub workflows with best practices
- Clean up GitHub workflows for consistency
- Set up cargo-dist for native installers
- Add Release workflow for GitHub releases
- Add cargo-dist plan step to CI workflow
- Add cargo-dist configuration for native installers
- Add Yandex and Apple Maps tile providers
- Add plans for missing tile providers and native installers
- Extract UI message handlers to separate module
- Config validation and UI handler extraction
- Standardize on parking_lot mutexes
- Fetch User-Agent at build time from Chrome releases
- Add input validation for parsed numeric values
- Force HTTPS for Bing and NAIP providers
- Update PLAN.md and apply cargo fmt
- Add builder pattern for DdsFileSystem
- R1 code quality fixes: cache eviction, clones, deduplication, dead code
- Add user guide
- Add configuration reference guide
- Fix duplicate entry in plan file
- Update README and add installation guide
- Consolidate multiple Tokio runtimes into one
- Share HTTP clients across tile providers
- Replace Mutex with RwLock in DdsFileSystem
- Combine CI and release into single workflow
- comment out winfsp until fully implemented
- fix formatting to pass CI
- simplify and update PLAN.md to reflect actual state
- update PLAN.md to reflect completed custom map integration
- cleanups
- remove optional fuse feature flag
- add plan to hide roads
- update PLAN.md and SimBrief plan with current progress
- update PLAN.md — mark SimBrief config+UI complete
- update PLAN.md with current session progress
- move scenery paths from Scenery screen to Settings
- *(main)* release 0.1.0
- opt into Node.js 24 for GitHub Actions
- Initial commit

## [0.5.8](https://github.com/egeland/autoortho-rs/compare/v0.5.7...v0.5.8) (2026-03-30)


### Bug Fixes

* delete existing release before cargo-dist creates new one ([#47](https://github.com/egeland/autoortho-rs/issues/47)) ([037f5f0](https://github.com/egeland/autoortho-rs/commit/037f5f0bac222d6c05f9a4872d4c632ce12577c3))

## [0.5.7](https://github.com/egeland/autoortho-rs/compare/v0.5.6...v0.5.7) (2026-03-30)


### Bug Fixes

* skip GitHub Release creation in release-please ([#45](https://github.com/egeland/autoortho-rs/issues/45)) ([849efb2](https://github.com/egeland/autoortho-rs/commit/849efb240a1b6ace8624d5540e528c170a153464))

## [0.5.6](https://github.com/egeland/autoortho-rs/compare/v0.5.5...v0.5.6) (2026-03-30)


### Bug Fixes

* fix Windows compilation errors in main.rs ([#43](https://github.com/egeland/autoortho-rs/issues/43)) ([89a43e7](https://github.com/egeland/autoortho-rs/commit/89a43e76918cd75246e47376b2d1f91f6d6ecb1c))

## [0.5.5](https://github.com/egeland/autoortho-rs/compare/v0.5.4...v0.5.5) (2026-03-30)


### Bug Fixes

* use cargo-dist expanded workflow with dispatch-releases ([#41](https://github.com/egeland/autoortho-rs/issues/41)) ([60c6db8](https://github.com/egeland/autoortho-rs/commit/60c6db8475a5b9620afd20d6a25e384ed58c1dde))

## [0.5.4](https://github.com/egeland/autoortho-rs/compare/v0.5.3...v0.5.4) (2026-03-30)


### Bug Fixes

* use cargo-dist dispatch-releases mode for automated builds ([#39](https://github.com/egeland/autoortho-rs/issues/39)) ([7d6b333](https://github.com/egeland/autoortho-rs/commit/7d6b33367914b493f4c78169d6a0ee6a69ee6777))

## [0.5.3](https://github.com/egeland/autoortho-rs/compare/v0.5.2...v0.5.3) (2026-03-30)


### Bug Fixes

* recreate tag via GitHub API to trigger release build ([#37](https://github.com/egeland/autoortho-rs/issues/37)) ([e1b47d1](https://github.com/egeland/autoortho-rs/commit/e1b47d1886ba5f9fbe6c151b1415bff25aa7a43e))

## [0.5.2](https://github.com/egeland/autoortho-rs/compare/v0.5.1...v0.5.2) (2026-03-30)


### Bug Fixes

* use repository_dispatch to trigger release build ([#35](https://github.com/egeland/autoortho-rs/issues/35)) ([284eceb](https://github.com/egeland/autoortho-rs/commit/284eceba3b128ab6c11fac0973ea7b658e8e014f))

## [0.5.1](https://github.com/egeland/autoortho-rs/compare/v0.5.0...v0.5.1) (2026-03-30)


### Bug Fixes

* add gate job so workflow_dispatch works with reusable workflow ([#33](https://github.com/egeland/autoortho-rs/issues/33)) ([d72dcd2](https://github.com/egeland/autoortho-rs/commit/d72dcd2a47f88b94f4e856201ba4b4383e2d3e1a))

## [0.5.0](https://github.com/egeland/autoortho-rs/compare/v0.4.5...v0.5.0) (2026-03-30)


### Features

* add 7z extraction and seasonal adjustment UI ([d8fae31](https://github.com/egeland/autoortho-rs/commit/d8fae31c9c0f9556d7d7b4dc256fe5a1597e1e0c))
* add automated versioning and release binaries ([980283b](https://github.com/egeland/autoortho-rs/commit/980283b7b851e7d91ef0b899ca2afaf705685111))
* add cache viewer web UI to visualize cached DDS tiles ([6167415](https://github.com/egeland/autoortho-rs/commit/6167415da30110536e2beddbc07ec19a08628dca))
* add criterion benchmarks for performance testing ([c65ee40](https://github.com/egeland/autoortho-rs/commit/c65ee40162858db9e3ca862701fde99cfee8e991))
* Add cross-platform FUSE support using unifuse ([7bae563](https://github.com/egeland/autoortho-rs/commit/7bae563d85a4cc9bf6114b0836aac41e89d550c3))
* add custom map tile provider support with per-cell overrides ([415ff1f](https://github.com/egeland/autoortho-rs/commit/415ff1f04959cde2e8472aa796892b482935556c))
* add expandable flight plan details on Dashboard ([d1334d3](https://github.com/egeland/autoortho-rs/commit/d1334d33d79a58b2cfd61c2247c84bcd469bb344))
* add SimBrief integration — config, Settings UI, Dashboard fetch ([b3ff7ed](https://github.com/egeland/autoortho-rs/commit/b3ff7edba9424004114ae669218a6803fe234406))
* add SimBrief route settings and Prefetch Route button placeholder ([aa5a22b](https://github.com/egeland/autoortho-rs/commit/aa5a22b2631ae358cdbd80706fcdcd2faf4bd859))
* add SimHeaven X-World compatibility support ([#11](https://github.com/egeland/autoortho-rs/issues/11)) ([e7bdd44](https://github.com/egeland/autoortho-rs/commit/e7bdd4411b7db0fcc8b1a16bec60a9a3026da1d4)), closes [#6](https://github.com/egeland/autoortho-rs/issues/6)
* Add Windows FUSE support using winfsp ([3be466d](https://github.com/egeland/autoortho-rs/commit/3be466d27430013fd1f8969cab589bdad667169a))
* auto-merge release-please PRs ([#20](https://github.com/egeland/autoortho-rs/issues/20)) ([0d41246](https://github.com/egeland/autoortho-rs/commit/0d41246fb405236fcfebc5612f21f8ee72367859))
* disable Start button when scenery_packs.ini not found ([de52b35](https://github.com/egeland/autoortho-rs/commit/de52b356831c273a0c1842a0fb221f6dbcfa006d))
* implement dynamic zoom with altitude-based rules and upserving ([71ec855](https://github.com/egeland/autoortho-rs/commit/71ec85566fc5ec98eb4ab1da3791273ead1586a7))
* Implement fallback system and fix security/memory issues ([dba5bff](https://github.com/egeland/autoortho-rs/commit/dba5bff79ea0029c580fe495ab6e87f9f76a81fc))
* implement SimBrief route prefetch settings and UI ([0c4bb77](https://github.com/egeland/autoortho-rs/commit/0c4bb77d26f0d7044ca7743374602927a13c7634))
* Implement WebSocket for live position updates ([4b8c49b](https://github.com/egeland/autoortho-rs/commit/4b8c49b619e2e5aa3c0e05ff66da80835bafd0ef))
* improve path labels and add hover tooltips ([819a223](https://github.com/egeland/autoortho-rs/commit/819a223e6b488c9137e2476838817a3380db2b9e))
* replace hand-rolled BCn compression with texpresso ([6531e9d](https://github.com/egeland/autoortho-rs/commit/6531e9dc7ae1a05a0ca5d166e74a9f190fafcdb0))
* simplify path config — derive mount and install from X-Plane folder ([884ec95](https://github.com/egeland/autoortho-rs/commit/884ec954b2f2d671fc3111d0a399da60b1ac5c7b))
* switch reqwest to pure Rust TLS (rustls) ([dac508a](https://github.com/egeland/autoortho-rs/commit/dac508acae38b2cefff9063ab7ea5f8befc77175))
* two-column flight plan with TOC/TOD highlighting ([c8c4bd3](https://github.com/egeland/autoortho-rs/commit/c8c4bd392586d329ba895cf4562b6780b1960412))
* update Scenery Install tooltip and warn if scenery_packs.ini missing ([a5f0e91](https://github.com/egeland/autoortho-rs/commit/a5f0e919a49fb12a72b9d99798aaaaa2329b7a5b))
* upgrade fuser 0.14 → 0.17 ([a2325d6](https://github.com/egeland/autoortho-rs/commit/a2325d64342506ac8ddeef68696fac2e8c35b0e6))
* wire night exclusion into FUSE filesystem ([e1de229](https://github.com/egeland/autoortho-rs/commit/e1de2299bf67d3da7bcc9d75558ab539428f8a7c))
* wire persistent DDS disk cache with Settings UI ([e7763bd](https://github.com/egeland/autoortho-rs/commit/e7763bddb68872c15d1f834da4032dda1215690e))


### Bug Fixes

* add actions permission to release-please workflow ([50ac364](https://github.com/egeland/autoortho-rs/commit/50ac364136f85fae39c8184fd6360a2c66a51cfc))
* add FUSE_NO_PKG_CONFIG for macOS builds ([f9bc619](https://github.com/egeland/autoortho-rs/commit/f9bc6197878454386ad406e94cb8e8b197477129))
* Address silent errors, WinFSP runtime, and duplicate providers ([68b12ce](https://github.com/egeland/autoortho-rs/commit/68b12ce67a36f043cd2acb83862f5b6ecdcb0b20))
* combine release-please and binary builds into single workflow ([2f825bd](https://github.com/egeland/autoortho-rs/commit/2f825bd75cafb0612178a01a72b5ebb8bff35984))
* correct version.yml workflow_run condition paths ([#17](https://github.com/egeland/autoortho-rs/issues/17)) ([e739d28](https://github.com/egeland/autoortho-rs/commit/e739d28dcc9c6f9ba84ca41d5e32a16b8bd828e5))
* correct winfsp 0.12 API for Windows build ([#15](https://github.com/egeland/autoortho-rs/issues/15)) ([98a1363](https://github.com/egeland/autoortho-rs/commit/98a1363b60b405a57b80e66c5651bc5f3794226c))
* Disable FUSE on Windows due to unifuse/winFSP incompatibility ([e03a8ee](https://github.com/egeland/autoortho-rs/commit/e03a8eecdb6fc0497823a752ef5b0f8816b3587a))
* increase flight plan waypoint text size from 12 to 13 ([7f52f6b](https://github.com/egeland/autoortho-rs/commit/7f52f6b2ed95aaf79b9ab41ebda4751f98073e2b))
* install macfuse on macOS builds ([9415707](https://github.com/egeland/autoortho-rs/commit/9415707983c7805fcceca2cc128ff3e61b2ce5fa))
* make build job depend on check and test ([ea5570e](https://github.com/egeland/autoortho-rs/commit/ea5570ee6d48980341e8aba002bf4a3a4e4510c2))
* make FUSE dependency optional for Windows builds ([ec61584](https://github.com/egeland/autoortho-rs/commit/ec6158473d0c3cdbdfd9325d8fdddb605c96cfef))
* match Scenery path label sizes to Settings screen ([3531e31](https://github.com/egeland/autoortho-rs/commit/3531e31360ca191b33e3fa15f1397d07896b283b))
* move scenery_packs.ini warning under X-Plane Folder input ([49bcb51](https://github.com/egeland/autoortho-rs/commit/49bcb51026b0f7db14754363e88c16283aaf4986))
* re-push tag to trigger release.yml instead of workflow_dispatch ([#28](https://github.com/egeland/autoortho-rs/issues/28)) ([3af69d6](https://github.com/egeland/autoortho-rs/commit/3af69d6095405d34f1518737a5be29c52e1cf615))
* reduce config save log from info to debug ([8d19cf7](https://github.com/egeland/autoortho-rs/commit/8d19cf7eab7b3cf00ecf8ee7190919dd695b13ce))
* remove commit message filter from version.yml ([#26](https://github.com/egeland/autoortho-rs/issues/26)) ([e22c50d](https://github.com/egeland/autoortho-rs/commit/e22c50dc1b6fafd874c7276c2bd38818745e04ff))
* remove obsolete fuse feature flag from CI workflows ([872c097](https://github.com/egeland/autoortho-rs/commit/872c0978172ebb5d14cc294a00106bc49a90a581))
* remove required input from release.yml workflow_dispatch ([#24](https://github.com/egeland/autoortho-rs/issues/24)) ([aa3421d](https://github.com/egeland/autoortho-rs/commit/aa3421db97dfa0248f3e19b4057aae592bde8e79))
* rename "Temp Downloads" to "Scenery Downloads" ([1e13177](https://github.com/egeland/autoortho-rs/commit/1e13177eea4dbe4ad453a0f53e0887b3455b5d72))
* resolve Windows build errors (texpresso, winfsp types) ([#16](https://github.com/egeland/autoortho-rs/issues/16)) ([6d54c15](https://github.com/egeland/autoortho-rs/commit/6d54c154288feae36433f180823c058cc1d238aa))
* rewrite mount_win.rs for winfsp 0.12 API compatibility ([#14](https://github.com/egeland/autoortho-rs/issues/14)) ([c535dc5](https://github.com/egeland/autoortho-rs/commit/c535dc53f7933fcbe823093283b8aa9a02107f21))
* show field elevation for airports in flight plan display ([e928d7c](https://github.com/egeland/autoortho-rs/commit/e928d7c64da56efe63e163e08e6ac7c97d8c17c5))
* skip coverage check for cells with custom map override ([da054ea](https://github.com/egeland/autoortho-rs/commit/da054ea54ab68d03ea2ee83ab6dd0bfd6190407c))
* trigger cargo-dist release build after release-please creates tag ([#22](https://github.com/egeland/autoortho-rs/issues/22)) ([10e92cf](https://github.com/egeland/autoortho-rs/commit/10e92cf384e9ac26ed9978d8d6c19ed4959ba279))
* update Leaflet to 1.9.4 and add XSS protection ([0f49220](https://github.com/egeland/autoortho-rs/commit/0f49220d8a6f92b3ec5a1bf91edee1fb25b76feb))
* use ICAO airport codes for SimBrief route preview, rename to User ID Number ([6a7dd47](https://github.com/egeland/autoortho-rs/commit/6a7dd47db69c4d51f82e15edac3f8292c9e3ac74))
* use RELEASE_TOKEN for release-please permissions ([#18](https://github.com/egeland/autoortho-rs/issues/18)) ([3ad52ac](https://github.com/egeland/autoortho-rs/commit/3ad52aca9c8d7929ce2f12504abe2fc1bd2b75c8))
* use stable Rust toolchain in cross-platform CI ([#13](https://github.com/egeland/autoortho-rs/issues/13)) ([f132930](https://github.com/egeland/autoortho-rs/commit/f13293055d75df9246904633e725b6ed05338b8a))
* use workflow_dispatch to trigger release build ([#30](https://github.com/egeland/autoortho-rs/issues/30)) ([b5617cf](https://github.com/egeland/autoortho-rs/commit/b5617cfa863aae1e889efc4f5e886cef86b9bec2))
* wire DDS in-memory cache size from config ([#12](https://github.com/egeland/autoortho-rs/issues/12)) ([e3df78a](https://github.com/egeland/autoortho-rs/commit/e3df78afa65dd0bbde41904df43defb48c3536f1))


### Performance Improvements

* Consolidate multiple Tokio runtimes into one ([eabaffb](https://github.com/egeland/autoortho-rs/commit/eabaffbc411af18da87218028d13b344ebb2a885))
* Replace Mutex with RwLock in DdsFileSystem ([d8ec852](https://github.com/egeland/autoortho-rs/commit/d8ec85217e1d23b8f6c6fa240869fe5ab2f512c4))
* Share HTTP clients across tile providers ([6c61afe](https://github.com/egeland/autoortho-rs/commit/6c61afe5d8e88387712f0d9c4dcf616f0ffc224c))

## [0.4.5](https://github.com/egeland/autoortho-rs/compare/v0.4.4...v0.4.5) (2026-03-30)


### Bug Fixes

* use workflow_dispatch to trigger release build ([#30](https://github.com/egeland/autoortho-rs/issues/30)) ([b5617cf](https://github.com/egeland/autoortho-rs/commit/b5617cfa863aae1e889efc4f5e886cef86b9bec2))

## [0.4.4](https://github.com/egeland/autoortho-rs/compare/v0.4.3...v0.4.4) (2026-03-30)


### Bug Fixes

* re-push tag to trigger release.yml instead of workflow_dispatch ([#28](https://github.com/egeland/autoortho-rs/issues/28)) ([3af69d6](https://github.com/egeland/autoortho-rs/commit/3af69d6095405d34f1518737a5be29c52e1cf615))

## [0.4.3](https://github.com/egeland/autoortho-rs/compare/v0.4.2...v0.4.3) (2026-03-30)


### Bug Fixes

* remove commit message filter from version.yml ([#26](https://github.com/egeland/autoortho-rs/issues/26)) ([e22c50d](https://github.com/egeland/autoortho-rs/commit/e22c50dc1b6fafd874c7276c2bd38818745e04ff))

## [0.4.2](https://github.com/egeland/autoortho-rs/compare/v0.4.1...v0.4.2) (2026-03-30)


### Bug Fixes

* remove required input from release.yml workflow_dispatch ([#24](https://github.com/egeland/autoortho-rs/issues/24)) ([aa3421d](https://github.com/egeland/autoortho-rs/commit/aa3421db97dfa0248f3e19b4057aae592bde8e79))

## [0.4.1](https://github.com/egeland/autoortho-rs/compare/v0.4.0...v0.4.1) (2026-03-30)


### Bug Fixes

* trigger cargo-dist release build after release-please creates tag ([#22](https://github.com/egeland/autoortho-rs/issues/22)) ([10e92cf](https://github.com/egeland/autoortho-rs/commit/10e92cf384e9ac26ed9978d8d6c19ed4959ba279))

## [0.4.0](https://github.com/egeland/autoortho-rs/compare/v0.3.0...v0.4.0) (2026-03-30)


### Features

* auto-merge release-please PRs ([#20](https://github.com/egeland/autoortho-rs/issues/20)) ([0d41246](https://github.com/egeland/autoortho-rs/commit/0d41246fb405236fcfebc5612f21f8ee72367859))

## [0.3.0](https://github.com/egeland/autoortho-rs/compare/v0.2.0...v0.3.0) (2026-03-30)


### Features

* add SimHeaven X-World compatibility support ([#11](https://github.com/egeland/autoortho-rs/issues/11)) ([e7bdd44](https://github.com/egeland/autoortho-rs/commit/e7bdd4411b7db0fcc8b1a16bec60a9a3026da1d4)), closes [#6](https://github.com/egeland/autoortho-rs/issues/6)
* Add Windows FUSE support using winfsp ([3be466d](https://github.com/egeland/autoortho-rs/commit/3be466d27430013fd1f8969cab589bdad667169a))
* Implement fallback system and fix security/memory issues ([dba5bff](https://github.com/egeland/autoortho-rs/commit/dba5bff79ea0029c580fe495ab6e87f9f76a81fc))
* Implement WebSocket for live position updates ([4b8c49b](https://github.com/egeland/autoortho-rs/commit/4b8c49b619e2e5aa3c0e05ff66da80835bafd0ef))


### Bug Fixes

* Address silent errors, WinFSP runtime, and duplicate providers ([68b12ce](https://github.com/egeland/autoortho-rs/commit/68b12ce67a36f043cd2acb83862f5b6ecdcb0b20))
* correct version.yml workflow_run condition paths ([#17](https://github.com/egeland/autoortho-rs/issues/17)) ([e739d28](https://github.com/egeland/autoortho-rs/commit/e739d28dcc9c6f9ba84ca41d5e32a16b8bd828e5))
* correct winfsp 0.12 API for Windows build ([#15](https://github.com/egeland/autoortho-rs/issues/15)) ([98a1363](https://github.com/egeland/autoortho-rs/commit/98a1363b60b405a57b80e66c5651bc5f3794226c))
* Disable FUSE on Windows due to unifuse/winFSP incompatibility ([e03a8ee](https://github.com/egeland/autoortho-rs/commit/e03a8eecdb6fc0497823a752ef5b0f8816b3587a))
* resolve Windows build errors (texpresso, winfsp types) ([#16](https://github.com/egeland/autoortho-rs/issues/16)) ([6d54c15](https://github.com/egeland/autoortho-rs/commit/6d54c154288feae36433f180823c058cc1d238aa))
* rewrite mount_win.rs for winfsp 0.12 API compatibility ([#14](https://github.com/egeland/autoortho-rs/issues/14)) ([c535dc5](https://github.com/egeland/autoortho-rs/commit/c535dc53f7933fcbe823093283b8aa9a02107f21))
* use RELEASE_TOKEN for release-please permissions ([#18](https://github.com/egeland/autoortho-rs/issues/18)) ([3ad52ac](https://github.com/egeland/autoortho-rs/commit/3ad52aca9c8d7929ce2f12504abe2fc1bd2b75c8))
* use stable Rust toolchain in cross-platform CI ([#13](https://github.com/egeland/autoortho-rs/issues/13)) ([f132930](https://github.com/egeland/autoortho-rs/commit/f13293055d75df9246904633e725b6ed05338b8a))
* wire DDS in-memory cache size from config ([#12](https://github.com/egeland/autoortho-rs/issues/12)) ([e3df78a](https://github.com/egeland/autoortho-rs/commit/e3df78afa65dd0bbde41904df43defb48c3536f1))


### Performance Improvements

* Consolidate multiple Tokio runtimes into one ([eabaffb](https://github.com/egeland/autoortho-rs/commit/eabaffbc411af18da87218028d13b344ebb2a885))
* Replace Mutex with RwLock in DdsFileSystem ([d8ec852](https://github.com/egeland/autoortho-rs/commit/d8ec85217e1d23b8f6c6fa240869fe5ab2f512c4))
* Share HTTP clients across tile providers ([6c61afe](https://github.com/egeland/autoortho-rs/commit/6c61afe5d8e88387712f0d9c4dcf616f0ffc224c))

## [0.2.0](https://github.com/egeland/autoortho-rs/compare/v0.1.0...v0.2.0) (2026-03-28)


### Features

* add 7z extraction and seasonal adjustment UI ([d8fae31](https://github.com/egeland/autoortho-rs/commit/d8fae31c9c0f9556d7d7b4dc256fe5a1597e1e0c))
* add cache viewer web UI to visualize cached DDS tiles ([6167415](https://github.com/egeland/autoortho-rs/commit/6167415da30110536e2beddbc07ec19a08628dca))
* add criterion benchmarks for performance testing ([c65ee40](https://github.com/egeland/autoortho-rs/commit/c65ee40162858db9e3ca862701fde99cfee8e991))
* Add cross-platform FUSE support using unifuse ([7bae563](https://github.com/egeland/autoortho-rs/commit/7bae563d85a4cc9bf6114b0836aac41e89d550c3))
* add custom map tile provider support with per-cell overrides ([415ff1f](https://github.com/egeland/autoortho-rs/commit/415ff1f04959cde2e8472aa796892b482935556c))
* add expandable flight plan details on Dashboard ([d1334d3](https://github.com/egeland/autoortho-rs/commit/d1334d33d79a58b2cfd61c2247c84bcd469bb344))
* add SimBrief integration — config, Settings UI, Dashboard fetch ([b3ff7ed](https://github.com/egeland/autoortho-rs/commit/b3ff7edba9424004114ae669218a6803fe234406))
* add SimBrief route settings and Prefetch Route button placeholder ([aa5a22b](https://github.com/egeland/autoortho-rs/commit/aa5a22b2631ae358cdbd80706fcdcd2faf4bd859))
* disable Start button when scenery_packs.ini not found ([de52b35](https://github.com/egeland/autoortho-rs/commit/de52b356831c273a0c1842a0fb221f6dbcfa006d))
* implement dynamic zoom with altitude-based rules and upserving ([71ec855](https://github.com/egeland/autoortho-rs/commit/71ec85566fc5ec98eb4ab1da3791273ead1586a7))
* implement SimBrief route prefetch settings and UI ([0c4bb77](https://github.com/egeland/autoortho-rs/commit/0c4bb77d26f0d7044ca7743374602927a13c7634))
* improve path labels and add hover tooltips ([819a223](https://github.com/egeland/autoortho-rs/commit/819a223e6b488c9137e2476838817a3380db2b9e))
* replace hand-rolled BCn compression with texpresso ([6531e9d](https://github.com/egeland/autoortho-rs/commit/6531e9dc7ae1a05a0ca5d166e74a9f190fafcdb0))
* simplify path config — derive mount and install from X-Plane folder ([884ec95](https://github.com/egeland/autoortho-rs/commit/884ec954b2f2d671fc3111d0a399da60b1ac5c7b))
* switch reqwest to pure Rust TLS (rustls) ([dac508a](https://github.com/egeland/autoortho-rs/commit/dac508acae38b2cefff9063ab7ea5f8befc77175))
* two-column flight plan with TOC/TOD highlighting ([c8c4bd3](https://github.com/egeland/autoortho-rs/commit/c8c4bd392586d329ba895cf4562b6780b1960412))
* update Scenery Install tooltip and warn if scenery_packs.ini missing ([a5f0e91](https://github.com/egeland/autoortho-rs/commit/a5f0e919a49fb12a72b9d99798aaaaa2329b7a5b))
* upgrade fuser 0.14 → 0.17 ([a2325d6](https://github.com/egeland/autoortho-rs/commit/a2325d64342506ac8ddeef68696fac2e8c35b0e6))
* wire night exclusion into FUSE filesystem ([e1de229](https://github.com/egeland/autoortho-rs/commit/e1de2299bf67d3da7bcc9d75558ab539428f8a7c))
* wire persistent DDS disk cache with Settings UI ([e7763bd](https://github.com/egeland/autoortho-rs/commit/e7763bddb68872c15d1f834da4032dda1215690e))


### Bug Fixes

* add actions permission to release-please workflow ([50ac364](https://github.com/egeland/autoortho-rs/commit/50ac364136f85fae39c8184fd6360a2c66a51cfc))
* add FUSE_NO_PKG_CONFIG for macOS builds ([f9bc619](https://github.com/egeland/autoortho-rs/commit/f9bc6197878454386ad406e94cb8e8b197477129))
* combine release-please and binary builds into single workflow ([2f825bd](https://github.com/egeland/autoortho-rs/commit/2f825bd75cafb0612178a01a72b5ebb8bff35984))
* increase flight plan waypoint text size from 12 to 13 ([7f52f6b](https://github.com/egeland/autoortho-rs/commit/7f52f6b2ed95aaf79b9ab41ebda4751f98073e2b))
* install macfuse on macOS builds ([9415707](https://github.com/egeland/autoortho-rs/commit/9415707983c7805fcceca2cc128ff3e61b2ce5fa))
* make build job depend on check and test ([ea5570e](https://github.com/egeland/autoortho-rs/commit/ea5570ee6d48980341e8aba002bf4a3a4e4510c2))
* make FUSE dependency optional for Windows builds ([ec61584](https://github.com/egeland/autoortho-rs/commit/ec6158473d0c3cdbdfd9325d8fdddb605c96cfef))
* match Scenery path label sizes to Settings screen ([3531e31](https://github.com/egeland/autoortho-rs/commit/3531e31360ca191b33e3fa15f1397d07896b283b))
* move scenery_packs.ini warning under X-Plane Folder input ([49bcb51](https://github.com/egeland/autoortho-rs/commit/49bcb51026b0f7db14754363e88c16283aaf4986))
* reduce config save log from info to debug ([8d19cf7](https://github.com/egeland/autoortho-rs/commit/8d19cf7eab7b3cf00ecf8ee7190919dd695b13ce))
* remove obsolete fuse feature flag from CI workflows ([872c097](https://github.com/egeland/autoortho-rs/commit/872c0978172ebb5d14cc294a00106bc49a90a581))
* rename "Temp Downloads" to "Scenery Downloads" ([1e13177](https://github.com/egeland/autoortho-rs/commit/1e13177eea4dbe4ad453a0f53e0887b3455b5d72))
* show field elevation for airports in flight plan display ([e928d7c](https://github.com/egeland/autoortho-rs/commit/e928d7c64da56efe63e163e08e6ac7c97d8c17c5))
* skip coverage check for cells with custom map override ([da054ea](https://github.com/egeland/autoortho-rs/commit/da054ea54ab68d03ea2ee83ab6dd0bfd6190407c))
* update Leaflet to 1.9.4 and add XSS protection ([0f49220](https://github.com/egeland/autoortho-rs/commit/0f49220d8a6f92b3ec5a1bf91edee1fb25b76feb))
* use ICAO airport codes for SimBrief route preview, rename to User ID Number ([6a7dd47](https://github.com/egeland/autoortho-rs/commit/6a7dd47db69c4d51f82e15edac3f8292c9e3ac74))

## 0.1.0 (2026-03-27)


### Features

* add automated versioning and release binaries ([980283b](https://github.com/egeland/autoortho-rs/commit/980283b7b851e7d91ef0b899ca2afaf705685111))
