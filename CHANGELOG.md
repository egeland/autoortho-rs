# Changelog

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
