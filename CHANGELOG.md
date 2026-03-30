# Changelog

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
