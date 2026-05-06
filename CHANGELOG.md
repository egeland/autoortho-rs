# Changelog

## [0.8.2](https://github.com/egeland/autoortho-rs/compare/v0.8.1...v0.8.2) (2026-05-06)


### Bug Fixes

* clarify VERSION constant documentation ([769741e](https://github.com/egeland/autoortho-rs/commit/769741e75dd2eb37dacef86881cf2a4f65fb0a1b))
* correct tag filter pattern in release workflow to trigger on version tags ([#308](https://github.com/egeland/autoortho-rs/issues/308)) ([7b56bf0](https://github.com/egeland/autoortho-rs/commit/7b56bf062c15f0a9f4ab10ced7a22b3225ae91b3))
* download Dokan DLL directly from GitHub releases ([#312](https://github.com/egeland/autoortho-rs/issues/312)) ([ab3b824](https://github.com/egeland/autoortho-rs/commit/ab3b8246b447816ad121ad018da1b70e41cbb7f1))
* download Dokan DLL directly from GitHub releases ([#313](https://github.com/egeland/autoortho-rs/issues/313)) ([20a27f7](https://github.com/egeland/autoortho-rs/commit/20a27f7933996040a752c947b81a1eeab8b1e16a))
* set DOKAN_DIR in Windows install step to resolve DLL injection failure ([#309](https://github.com/egeland/autoortho-rs/issues/309)) ([d83a68b](https://github.com/egeland/autoortho-rs/commit/d83a68bfa90e19d9d0af21c23fc85c3ce6cf3c1c))
* streamline VERSION constant documentation ([#314](https://github.com/egeland/autoortho-rs/issues/314)) ([f1e82a9](https://github.com/egeland/autoortho-rs/commit/f1e82a9f339f34a870489083ce0a9de8731c52c8))
* update to trigger workflow ([#315](https://github.com/egeland/autoortho-rs/issues/315)) ([fa20570](https://github.com/egeland/autoortho-rs/commit/fa205705842347f3a2690762eba2f16b1d243d89))
* use lib path for Dokan DLL and correct DLL name ([#311](https://github.com/egeland/autoortho-rs/issues/311)) ([0b3d63b](https://github.com/egeland/autoortho-rs/commit/0b3d63bc025e3b28f2a61b8d0a907966aa9f0b1d))

## [0.8.1](https://github.com/egeland/autoortho-rs/compare/v0.8.0...v0.8.1) (2026-05-05)


### Bug Fixes

* add bootstrap-sha to release-please config to limit commit history parsing ([#300](https://github.com/egeland/autoortho-rs/issues/300)) ([c0c0f8f](https://github.com/egeland/autoortho-rs/commit/c0c0f8f20f3185fb8855c1e11a27a57930612729))
* handle tag determination in release workflow for manual dispatch ([#297](https://github.com/egeland/autoortho-rs/issues/297)) ([6be08e9](https://github.com/egeland/autoortho-rs/commit/6be08e95bd6b48127f3eeca9811dcc5c8c4a7846))
* remove duplicate run key in release-please workflow ([#305](https://github.com/egeland/autoortho-rs/issues/305)) ([043cb51](https://github.com/egeland/autoortho-rs/commit/043cb51ccb633d94396800a130d0a02f36db1742))
* remove redundant comment ([#299](https://github.com/egeland/autoortho-rs/issues/299)) ([44e5bd6](https://github.com/egeland/autoortho-rs/commit/44e5bd6a947a518b00b8272839eed9e091d6f462))
* simplify release-please workflow to avoid shell parsing errors ([#302](https://github.com/egeland/autoortho-rs/issues/302)) ([4883939](https://github.com/egeland/autoortho-rs/commit/4883939974566462f83771ec70082f84539ed758))
* simplify release-please workflow to avoid shell parsing errors ([#302](https://github.com/egeland/autoortho-rs/issues/302)) ([#304](https://github.com/egeland/autoortho-rs/issues/304)) ([45c2c6d](https://github.com/egeland/autoortho-rs/commit/45c2c6d93cd7ec845682f44c606107f93d623fde))
* use --squash for auto-merge in release-please ([#307](https://github.com/egeland/autoortho-rs/issues/307)) ([92f95c8](https://github.com/egeland/autoortho-rs/commit/92f95c83d5e6a7bb40ab6bece1b290fb0cacbd19))


### Miscellaneous

* remove redundant comments in config.rs ([#298](https://github.com/egeland/autoortho-rs/issues/298)) ([6f0e4d4](https://github.com/egeland/autoortho-rs/commit/6f0e4d4c6efe67c48c6ce35a29c077574e2e2220))

## [0.8.0](https://github.com/egeland/autoortho-rs/compare/v0.7.1...v0.8.0) (2026-05-05)


### ⚠ BREAKING CHANGES

* Requires dokan >= 0.3 from crates.io

### Features

* add 7z extraction and seasonal adjustment UI ([d8fae31](https://github.com/egeland/autoortho-rs/commit/d8fae31c9c0f9556d7d7b4dc256fe5a1597e1e0c))
* add automated versioning and release binaries ([980283b](https://github.com/egeland/autoortho-rs/commit/980283b7b851e7d91ef0b899ca2afaf705685111))
* add cache viewer web UI to visualize cached DDS tiles ([6167415](https://github.com/egeland/autoortho-rs/commit/6167415da30110536e2beddbc07ec19a08628dca))
* add criterion benchmarks for performance testing ([c65ee40](https://github.com/egeland/autoortho-rs/commit/c65ee40162858db9e3ca862701fde99cfee8e991))
* Add cross-platform FUSE support using unifuse ([7bae563](https://github.com/egeland/autoortho-rs/commit/7bae563d85a4cc9bf6114b0836aac41e89d550c3))
* add custom map tile provider support with per-cell overrides ([415ff1f](https://github.com/egeland/autoortho-rs/commit/415ff1f04959cde2e8472aa796892b482935556c))
* add debug mode setting with UI toggle ([#247](https://github.com/egeland/autoortho-rs/issues/247)) ([4c60335](https://github.com/egeland/autoortho-rs/commit/4c603359eafeaba62b1f6ea3652daf632eb95883))
* add expandable flight plan details on Dashboard ([d1334d3](https://github.com/egeland/autoortho-rs/commit/d1334d33d79a58b2cfd61c2247c84bcd469bb344))
* add Inno Setup Windows installer workflow ([#57](https://github.com/egeland/autoortho-rs/issues/57)) ([e669c43](https://github.com/egeland/autoortho-rs/commit/e669c435f52fdb7dde93acc7be83ca4da2865985))
* add LRU disk cache eviction to DdsCache ([#53](https://github.com/egeland/autoortho-rs/issues/53)) ([3852285](https://github.com/egeland/autoortho-rs/commit/385228566a9455d860e3cfddf6404fd76a32e503))
* add release-plz for fully automated releases ([#51](https://github.com/egeland/autoortho-rs/issues/51)) ([e748169](https://github.com/egeland/autoortho-rs/commit/e748169eebdfc88fc1c3d4bbab8745e8f9dd8876))
* add request rate limiting for tile providers ([#102](https://github.com/egeland/autoortho-rs/issues/102)) ([ee066e8](https://github.com/egeland/autoortho-rs/commit/ee066e8c3d7a8d1806a609ce6076bf9aef927ae1))
* add SimBrief integration — config, Settings UI, Dashboard fetch ([b3ff7ed](https://github.com/egeland/autoortho-rs/commit/b3ff7edba9424004114ae669218a6803fe234406))
* add SimBrief route settings and Prefetch Route button placeholder ([aa5a22b](https://github.com/egeland/autoortho-rs/commit/aa5a22b2631ae358cdbd80706fcdcd2faf4bd859))
* add SimHeaven X-World compatibility support ([#11](https://github.com/egeland/autoortho-rs/issues/11)) ([e7bdd44](https://github.com/egeland/autoortho-rs/commit/e7bdd4411b7db0fcc8b1a16bec60a9a3026da1d4)), closes [#6](https://github.com/egeland/autoortho-rs/issues/6)
* add TileCoord newtype struct and update PLAN.md ([#153](https://github.com/egeland/autoortho-rs/issues/153)) ([865dde3](https://github.com/egeland/autoortho-rs/commit/865dde35a5383bd43f60edeb64971d184359d633))
* add VERSION constant and crates.io badges ([#288](https://github.com/egeland/autoortho-rs/issues/288)) ([f966632](https://github.com/egeland/autoortho-rs/commit/f9666321e2ef4cb00aa9aaa21caa40791d2af209))
* Add Windows FUSE support using winfsp ([3be466d](https://github.com/egeland/autoortho-rs/commit/3be466d27430013fd1f8969cab589bdad667169a))
* add Windows MSI installer support ([#54](https://github.com/egeland/autoortho-rs/issues/54)) ([a162b75](https://github.com/egeland/autoortho-rs/commit/a162b753aa60b79a17f4b420ad8b120855f74b65))
* add WiX MSI installer support ([#71](https://github.com/egeland/autoortho-rs/issues/71)) ([0724889](https://github.com/egeland/autoortho-rs/commit/07248894c772ea7c2ca4619925773956506976bd))
* **altitude-predictor:** optimize methods by removing unused parameters and adding early returns ([#149](https://github.com/egeland/autoortho-rs/issues/149)) ([67e4c7a](https://github.com/egeland/autoortho-rs/commit/67e4c7a20806ecb9bb1a4c971d81ca97625b9f5f))
* auto-approve release PR before merging ([dc74020](https://github.com/egeland/autoortho-rs/commit/dc74020151c347aded77d48be13699a31956b3c2))
* auto-merge release-please PRs ([#20](https://github.com/egeland/autoortho-rs/issues/20)) ([0d41246](https://github.com/egeland/autoortho-rs/commit/0d41246fb405236fcfebc5612f21f8ee72367859))
* auto-release when builds work ([#143](https://github.com/egeland/autoortho-rs/issues/143)) ([e1bfd14](https://github.com/egeland/autoortho-rs/commit/e1bfd1492921d42d7e405de8c5bde7a2b2db0317))
* bump version attempt 2 ([#106](https://github.com/egeland/autoortho-rs/issues/106)) ([d8fa53c](https://github.com/egeland/autoortho-rs/commit/d8fa53c09e2c561c31ffb3cf86ea58f2cc97164b))
* create reusable Windows build workflow with artifact support ([#256](https://github.com/egeland/autoortho-rs/issues/256)) ([2e9713e](https://github.com/egeland/autoortho-rs/commit/2e9713ed2226c65d349c926b7b4de834163c8c3f))
* disable Start button when scenery_packs.ini not found ([de52b35](https://github.com/egeland/autoortho-rs/commit/de52b356831c273a0c1842a0fb221f6dbcfa006d))
* dynamically set WiX version from release workflow ([3f3026a](https://github.com/egeland/autoortho-rs/commit/3f3026a610db881d4a86a65465c5ab88ce0c5e41))
* enable auto-merge for release-please PRs ([#282](https://github.com/egeland/autoortho-rs/issues/282)) ([1cdd825](https://github.com/egeland/autoortho-rs/commit/1cdd8259573811cb54cbcc077bafaa48e2797d4a))
* enable Prefetch Route button in dashboard ([#249](https://github.com/egeland/autoortho-rs/issues/249)) ([85fa031](https://github.com/egeland/autoortho-rs/commit/85fa031a6c7faefa758a22baae9afd5ecaaab781))
* enable Windows FUSE support using winfsp ([#81](https://github.com/egeland/autoortho-rs/issues/81)) ([29050f7](https://github.com/egeland/autoortho-rs/commit/29050f73d5b61b7ad88dd0fc2f1d9e7136e8d35b))
* implement dynamic zoom with altitude-based rules and upserving ([71ec855](https://github.com/egeland/autoortho-rs/commit/71ec85566fc5ec98eb4ab1da3791273ead1586a7))
* Implement fallback system and fix security/memory issues ([dba5bff](https://github.com/egeland/autoortho-rs/commit/dba5bff79ea0029c580fe495ab6e87f9f76a81fc))
* implement SimBrief route prefetch settings and UI ([0c4bb77](https://github.com/egeland/autoortho-rs/commit/0c4bb77d26f0d7044ca7743374602927a13c7634))
* Implement WebSocket for live position updates ([4b8c49b](https://github.com/egeland/autoortho-rs/commit/4b8c49b619e2e5aa3c0e05ff66da80835bafd0ef))
* improve path labels and add hover tooltips ([819a223](https://github.com/egeland/autoortho-rs/commit/819a223e6b488c9137e2476838817a3380db2b9e))
* Make GUI the default launch mode ([#189](https://github.com/egeland/autoortho-rs/issues/189)) ([4dc26eb](https://github.com/egeland/autoortho-rs/commit/4dc26eb3b47dc3878942e4c6cc9178f017d30de6))
* minor update ([#105](https://github.com/egeland/autoortho-rs/issues/105)) ([ecb8e03](https://github.com/egeland/autoortho-rs/commit/ecb8e03fddd4e334e3fb27d1cb4f8da004479648))
* optimise some filesystem calls ([#147](https://github.com/egeland/autoortho-rs/issues/147)) ([f909b37](https://github.com/egeland/autoortho-rs/commit/f909b37b04fb42adb9a74d372bd8176d180dc487))
* replace hand-rolled BCn compression with texpresso ([6531e9d](https://github.com/egeland/autoortho-rs/commit/6531e9dc7ae1a05a0ca5d166e74a9f190fafcdb0))
* Replace Windows WinFsp with Dokan for FUSE ([#266](https://github.com/egeland/autoortho-rs/issues/266)) ([ef7e48a](https://github.com/egeland/autoortho-rs/commit/ef7e48ae8314bfde843eedc71138320a5e9d5994))
* simplify path config — derive mount and install from X-Plane folder ([884ec95](https://github.com/egeland/autoortho-rs/commit/884ec954b2f2d671fc3111d0a399da60b1ac5c7b))
* switch reqwest to pure Rust TLS (rustls) ([dac508a](https://github.com/egeland/autoortho-rs/commit/dac508acae38b2cefff9063ab7ea5f8befc77175))
* test release-please auto-merge workflow ([#284](https://github.com/egeland/autoortho-rs/issues/284)) ([0602563](https://github.com/egeland/autoortho-rs/commit/06025637dc6eec114e9109b6f668c0d4386fbe42))
* two-column flight plan with TOC/TOD highlighting ([c8c4bd3](https://github.com/egeland/autoortho-rs/commit/c8c4bd392586d329ba895cf4562b6780b1960412))
* update Scenery Install tooltip and warn if scenery_packs.ini missing ([a5f0e91](https://github.com/egeland/autoortho-rs/commit/a5f0e919a49fb12a72b9d99798aaaaa2329b7a5b))
* upgrade fuser 0.14 → 0.17 ([a2325d6](https://github.com/egeland/autoortho-rs/commit/a2325d64342506ac8ddeef68696fac2e8c35b0e6))
* wire night exclusion into FUSE filesystem ([e1de229](https://github.com/egeland/autoortho-rs/commit/e1de2299bf67d3da7bcc9d75558ab539428f8a7c))
* wire persistent DDS disk cache with Settings UI ([e7763bd](https://github.com/egeland/autoortho-rs/commit/e7763bddb68872c15d1f834da4032dda1215690e))


### Bug Fixes

* add actions permission to release-please workflow ([50ac364](https://github.com/egeland/autoortho-rs/commit/50ac364136f85fae39c8184fd6360a2c66a51cfc))
* add debug listing to see where exe files are ([#134](https://github.com/egeland/autoortho-rs/issues/134)) ([46bffd9](https://github.com/egeland/autoortho-rs/commit/46bffd90f51a00c5967077affb9e695a8e9afa14))
* add FUSE_NO_PKG_CONFIG for macOS builds ([f9bc619](https://github.com/egeland/autoortho-rs/commit/f9bc6197878454386ad406e94cb8e8b197477129))
* add gate job so workflow_dispatch works with reusable workflow ([#33](https://github.com/egeland/autoortho-rs/issues/33)) ([d72dcd2](https://github.com/egeland/autoortho-rs/commit/d72dcd2a47f88b94f4e856201ba4b4383e2d3e1a))
* add GH_TOKEN to download step ([#65](https://github.com/egeland/autoortho-rs/issues/65)) ([1656083](https://github.com/egeland/autoortho-rs/commit/165608385b9e1e2bb609dc6f36a00971b1063dd5))
* add skip-github-release to release-please workflow ([#280](https://github.com/egeland/autoortho-rs/issues/280)) ([ba6f60d](https://github.com/egeland/autoortho-rs/commit/ba6f60dd4ea4e5972350b2669828c918733c3ad2))
* add stale mount cleanup before FUSE mount (P1) ([#251](https://github.com/egeland/autoortho-rs/issues/251)) ([f5f007d](https://github.com/egeland/autoortho-rs/commit/f5f007d95fffc1da7ec39358d43dc996087611ab))
* Address silent errors, WinFSP runtime, and duplicate providers ([68b12ce](https://github.com/egeland/autoortho-rs/commit/68b12ce67a36f043cd2acb83862f5b6ecdcb0b20))
* always checkout main, get version from Cargo.toml ([#73](https://github.com/egeland/autoortho-rs/issues/73)) ([d8dccb5](https://github.com/egeland/autoortho-rs/commit/d8dccb56129abc85cebb00a7802664f89c82d41c))
* always install WinFSP in release workflow ([#169](https://github.com/egeland/autoortho-rs/issues/169)) ([aef4d74](https://github.com/egeland/autoortho-rs/commit/aef4d74be9ac674ca6cf71472dd2bcfda64c29f6))
* bundle winfsp-x64.dll in WiX MSI installer ([#158](https://github.com/egeland/autoortho-rs/issues/158)) ([67ade8b](https://github.com/egeland/autoortho-rs/commit/67ade8b39a5a3e65f7782127850e47e568c54e4a))
* **ci:** add WinFsp bin to PATH for Windows DLL resolution ([#243](https://github.com/egeland/autoortho-rs/issues/243)) ([9b621d5](https://github.com/egeland/autoortho-rs/commit/9b621d5d89ceec945d972df2914361c74e0f70ed))
* **ci:** stage winfsp DLL to dist profile dir, not release ([#194](https://github.com/egeland/autoortho-rs/issues/194)) ([353e5ef](https://github.com/egeland/autoortho-rs/commit/353e5efb393fb03d89ed6de7c33ff54a7417d823))
* combine release-please and binary builds into single workflow ([2f825bd](https://github.com/egeland/autoortho-rs/commit/2f825bd75cafb0612178a01a72b5ebb8bff35984))
* configure release-plz to skip crates.io publish ([#52](https://github.com/egeland/autoortho-rs/issues/52)) ([8025acf](https://github.com/egeland/autoortho-rs/commit/8025acfc1a06d276c88d9fe855b4ff289b6fccdf))
* connect to map, and actually use FUSE ([#239](https://github.com/egeland/autoortho-rs/issues/239)) ([d1cf145](https://github.com/egeland/autoortho-rs/commit/d1cf1453de3d8fa90312a79a4cad6f82749fa8c2))
* connect to map, and actually use FUSE, formatted ([#241](https://github.com/egeland/autoortho-rs/issues/241)) ([bdead7f](https://github.com/egeland/autoortho-rs/commit/bdead7f4f780ed4b27a76917d98668599aa53f01))
* correct deny.toml syntax ([#183](https://github.com/egeland/autoortho-rs/issues/183)) ([08b3424](https://github.com/egeland/autoortho-rs/commit/08b3424c0370ec9d295e4e60a1a134f116a65990))
* correct EULA accept command for WiX v7 ([#124](https://github.com/egeland/autoortho-rs/issues/124)) ([31419a1](https://github.com/egeland/autoortho-rs/commit/31419a16de826d02dcec64f7c2c011c8d561beea))
* correct Inno Setup installation command and PATH handling ([#215](https://github.com/egeland/autoortho-rs/issues/215)) ([5fbbf6a](https://github.com/egeland/autoortho-rs/commit/5fbbf6a78155d2ebb771a8ffa1a4df5e5340cbe7))
* correct matrix.targets condition for WinFSP in release.yml ([#187](https://github.com/egeland/autoortho-rs/issues/187)) ([8a4ab34](https://github.com/egeland/autoortho-rs/commit/8a4ab3472ffc4f9e6448413e8a85e831209481d3))
* correct path in wix/main.wxs to artifacts/autoortho.exe ([d945079](https://github.com/egeland/autoortho-rs/commit/d9450794e78fe68b3757c2febda18fcec8891fbb))
* correct version.yml workflow_run condition paths ([#17](https://github.com/egeland/autoortho-rs/issues/17)) ([e739d28](https://github.com/egeland/autoortho-rs/commit/e739d28dcc9c6f9ba84ca41d5e32a16b8bd828e5))
* correct winfsp 0.12 API for Windows build ([#15](https://github.com/egeland/autoortho-rs/issues/15)) ([98a1363](https://github.com/egeland/autoortho-rs/commit/98a1363b60b405a57b80e66c5651bc5f3794226c))
* delete existing release before cargo-dist creates new one ([#47](https://github.com/egeland/autoortho-rs/issues/47)) ([037f5f0](https://github.com/egeland/autoortho-rs/commit/037f5f0bac222d6c05f9a4872d4c632ce12577c3))
* Disable FUSE on Windows due to unifuse/winFSP incompatibility ([e03a8ee](https://github.com/egeland/autoortho-rs/commit/e03a8eecdb6fc0497823a752ef5b0f8816b3587a))
* enable MSI in cargo-dist to show in Downloads section ([#141](https://github.com/egeland/autoortho-rs/issues/141)) ([f372279](https://github.com/egeland/autoortho-rs/commit/f372279c111ccff59e4a58876d4015c6a5744c9c))
* enable winfsp system feature and fix CI ([#161](https://github.com/egeland/autoortho-rs/issues/161)) ([a23a5b8](https://github.com/egeland/autoortho-rs/commit/a23a5b82681bb89150c8a4fd507a481155044dcb))
* extract version from tag in MSI build step ([#130](https://github.com/egeland/autoortho-rs/issues/130)) ([492a722](https://github.com/egeland/autoortho-rs/commit/492a72265a9ad194e9afee8033f88cbb37f41fd8))
* find exe dynamically and copy to target/distrib for MSI ([#132](https://github.com/egeland/autoortho-rs/issues/132)) ([f2336e6](https://github.com/egeland/autoortho-rs/commit/f2336e606c636eb4e463e79f48df141e770eedb0))
* fix download and NSIS script paths ([#59](https://github.com/egeland/autoortho-rs/issues/59)) ([f6a665c](https://github.com/egeland/autoortho-rs/commit/f6a665c9bde52a869c1b5105a462989e8957252f))
* fix installer workflow ([#61](https://github.com/egeland/autoortho-rs/issues/61)) ([ae33bd8](https://github.com/egeland/autoortho-rs/commit/ae33bd8478063dd09e841a403a32a8e9433ec2b5))
* fix Windows compilation errors in main.rs ([#43](https://github.com/egeland/autoortho-rs/issues/43)) ([89a43e7](https://github.com/egeland/autoortho-rs/commit/89a43e76918cd75246e47376b2d1f91f6d6ecb1c))
* fix WinFSP install condition - check for 'pc-windows-msvc' target ([#167](https://github.com/egeland/autoortho-rs/issues/167)) ([a9cec58](https://github.com/egeland/autoortho-rs/commit/a9cec58cd2d4aee975891d40db9827c3870ca067))
* fix WinFSP install condition in release workflow ([#165](https://github.com/egeland/autoortho-rs/issues/165)) ([a01571d](https://github.com/egeland/autoortho-rs/commit/a01571da8b56d2e155eab5467627ce340498f3c4))
* force Node.js 24 for Inno Setup Action on Windows ([#213](https://github.com/egeland/autoortho-rs/issues/213)) ([a323ef1](https://github.com/egeland/autoortho-rs/commit/a323ef189a963bccd3f52e7239eafce865303b00))
* increase flight plan waypoint text size from 12 to 13 ([7f52f6b](https://github.com/egeland/autoortho-rs/commit/7f52f6b2ed95aaf79b9ab41ebda4751f98073e2b))
* install macfuse on macOS builds ([9415707](https://github.com/egeland/autoortho-rs/commit/9415707983c7805fcceca2cc128ff3e61b2ce5fa))
* install WinFSP in release workflow for Windows build ([#163](https://github.com/egeland/autoortho-rs/issues/163)) ([9347161](https://github.com/egeland/autoortho-rs/commit/934716173d271d2f851fb71b92936158ab185c3e))
* make build job depend on check and test ([ea5570e](https://github.com/egeland/autoortho-rs/commit/ea5570ee6d48980341e8aba002bf4a3a4e4510c2))
* make FUSE dependency optional for Windows builds ([ec61584](https://github.com/egeland/autoortho-rs/commit/ec6158473d0c3cdbdfd9325d8fdddb605c96cfef))
* make FUSE mount conditional to fix Windows build ([#244](https://github.com/egeland/autoortho-rs/issues/244)) ([b76d482](https://github.com/egeland/autoortho-rs/commit/b76d482d9964d70dac5bd5e70fa772b72a0ccc3b))
* make release-plz depend on cross-platform tests passing ([#94](https://github.com/egeland/autoortho-rs/issues/94)) ([eb272cd](https://github.com/egeland/autoortho-rs/commit/eb272cd97bc20922b01d2e9980d51d82d1079f10))
* match Scenery path label sizes to Settings screen ([3531e31](https://github.com/egeland/autoortho-rs/commit/3531e31360ca191b33e3fa15f1397d07896b283b))
* move scenery_packs.ini warning under X-Plane Folder input ([49bcb51](https://github.com/egeland/autoortho-rs/commit/49bcb51026b0f7db14754363e88c16283aaf4986))
* properly escape gh pr merge command in release-please ([#291](https://github.com/egeland/autoortho-rs/issues/291)) ([8657614](https://github.com/egeland/autoortho-rs/commit/86576149a7ff4cc6837531d87dd757221958c112))
* re-push tag to trigger release.yml instead of workflow_dispatch ([#28](https://github.com/egeland/autoortho-rs/issues/28)) ([3af69d6](https://github.com/egeland/autoortho-rs/commit/3af69d6095405d34f1518737a5be29c52e1cf615))
* recreate tag via GitHub API to trigger release build ([#37](https://github.com/egeland/autoortho-rs/issues/37)) ([e1b47d1](https://github.com/egeland/autoortho-rs/commit/e1b47d1886ba5f9fbe6c151b1415bff25aa7a43e))
* reduce config save log from info to debug ([8d19cf7](https://github.com/egeland/autoortho-rs/commit/8d19cf7eab7b3cf00ecf8ee7190919dd695b13ce))
* regenerate release.yml to match cargo-dist dispatch-releases ([#50](https://github.com/egeland/autoortho-rs/issues/50)) ([e8e9e3b](https://github.com/egeland/autoortho-rs/commit/e8e9e3b67ab9bead931f839d223c00392c2e130d))
* remove approval step now that 0 reviewers required ([#145](https://github.com/egeland/autoortho-rs/issues/145)) ([f1867da](https://github.com/egeland/autoortho-rs/commit/f1867da54bc860616e261aacf6bb0717f2c703a1))
* remove commit message filter from version.yml ([#26](https://github.com/egeland/autoortho-rs/issues/26)) ([e22c50d](https://github.com/egeland/autoortho-rs/commit/e22c50dc1b6fafd874c7276c2bd38818745e04ff))
* remove junk ([#190](https://github.com/egeland/autoortho-rs/issues/190)) ([767465c](https://github.com/egeland/autoortho-rs/commit/767465cd3b51166454c4ad3fef62cb6e675b936e))
* remove malformed brace from release-please workflow ([#290](https://github.com/egeland/autoortho-rs/issues/290)) ([c2e79d2](https://github.com/egeland/autoortho-rs/commit/c2e79d2ab9148e85571f8176a0c0e38f576eb9b4))
* remove obsolete fuse feature flag from CI workflows ([872c097](https://github.com/egeland/autoortho-rs/commit/872c0978172ebb5d14cc294a00106bc49a90a581))
* remove quotes from gh pr merge to prevent shell escaping issue ([189e78e](https://github.com/egeland/autoortho-rs/commit/189e78e460f77701eb7998471bf7c888077cac12))
* remove required input from release.yml workflow_dispatch ([#24](https://github.com/egeland/autoortho-rs/issues/24)) ([aa3421d](https://github.com/egeland/autoortho-rs/commit/aa3421db97dfa0248f3e19b4057aae592bde8e79))
* rename "Temp Downloads" to "Scenery Downloads" ([1e13177](https://github.com/egeland/autoortho-rs/commit/1e13177eea4dbe4ad453a0f53e0887b3455b5d72))
* resolve Inno Setup installation failure ([#217](https://github.com/egeland/autoortho-rs/issues/217)) ([93dda2e](https://github.com/egeland/autoortho-rs/commit/93dda2e22ebedc0106c2b0fe6a5e392ed2ec988a))
* resolve Windows build errors (texpresso, winfsp types) ([#16](https://github.com/egeland/autoortho-rs/issues/16)) ([6d54c15](https://github.com/egeland/autoortho-rs/commit/6d54c154288feae36433f180823c058cc1d238aa))
* resolve WiX path issue for Windows installer build ([#198](https://github.com/egeland/autoortho-rs/issues/198)) ([34e5016](https://github.com/egeland/autoortho-rs/commit/34e5016a615f96ca4712da4e6bf0562f636675a6))
* rewrite mount_win.rs for winfsp 0.12 API compatibility ([#14](https://github.com/egeland/autoortho-rs/issues/14)) ([c535dc5](https://github.com/egeland/autoortho-rs/commit/c535dc53f7933fcbe823093283b8aa9a02107f21))
* run dist init to generate wix/main.wxs and fix MSI GUIDs ([7ba2b47](https://github.com/egeland/autoortho-rs/commit/7ba2b474791e36092b042d10caece6f665e189a5))
* save PR number to file before using in gh pr merge ([#296](https://github.com/egeland/autoortho-rs/issues/296)) ([3635071](https://github.com/egeland/autoortho-rs/commit/36350719889479aefa645c1b74ab9cac158b7ab0))
* set WINFSP_DIR env var for winfsp-sys build ([#181](https://github.com/egeland/autoortho-rs/issues/181)) ([d87021a](https://github.com/egeland/autoortho-rs/commit/d87021a80c5673eb7cf3e23ecdc086f115acc519))
* set WINFSP_DIR env var for winfsp-sys build in ci ([#185](https://github.com/egeland/autoortho-rs/issues/185)) ([21311f4](https://github.com/egeland/autoortho-rs/commit/21311f45f73135aa86b3f9fbb372039b2f704cbe))
* show field elevation for airports in flight plan display ([e928d7c](https://github.com/egeland/autoortho-rs/commit/e928d7c64da56efe63e163e08e6ac7c97d8c17c5))
* simplify using gh release view ([#63](https://github.com/egeland/autoortho-rs/issues/63)) ([c681237](https://github.com/egeland/autoortho-rs/commit/c68123778b2620b98c528343e2318451ae69bf87))
* skip coverage check for cells with custom map override ([da054ea](https://github.com/egeland/autoortho-rs/commit/da054ea54ab68d03ea2ee83ab6dd0bfd6190407c))
* skip GitHub Release creation in release-please ([#45](https://github.com/egeland/autoortho-rs/issues/45)) ([849efb2](https://github.com/egeland/autoortho-rs/commit/849efb240a1b6ace8624d5540e528c170a153464))
* trigger cargo-dist release build after release-please creates tag ([#22](https://github.com/egeland/autoortho-rs/issues/22)) ([10e92cf](https://github.com/egeland/autoortho-rs/commit/10e92cf384e9ac26ed9978d8d6c19ed4959ba279))
* try fixing windows builds ([#202](https://github.com/egeland/autoortho-rs/issues/202)) ([736023b](https://github.com/egeland/autoortho-rs/commit/736023b06f9326b39194aab416af5bfcd5497e56))
* try geminis workflow files, from web gemini ([#229](https://github.com/egeland/autoortho-rs/issues/229)) ([9502d95](https://github.com/egeland/autoortho-rs/commit/9502d9587807e84d969d5bf771450a434a62245e))
* try geminis workflow files, from web gemini, optimise release ([#231](https://github.com/egeland/autoortho-rs/issues/231)) ([d28bffe](https://github.com/egeland/autoortho-rs/commit/d28bffee79744c2377f914f21e87e6034dfe64cf))
* unify FUSE mounting across all platforms using unifuse ([#90](https://github.com/egeland/autoortho-rs/issues/90)) ([0ac8b43](https://github.com/egeland/autoortho-rs/commit/0ac8b43e76122bfd0a7e6b803aa0cf5bec5a273f))
* unzip without the root dir ([#236](https://github.com/egeland/autoortho-rs/issues/236)) ([747aa56](https://github.com/egeland/autoortho-rs/commit/747aa566715c424441a49f9c915d6ea992a4cb0b))
* update Inno Setup download URL to v6.7.1 ([#209](https://github.com/egeland/autoortho-rs/issues/209)) ([3bd87fd](https://github.com/egeland/autoortho-rs/commit/3bd87fd5473242782413b2da5a2cf8279f7877c1))
* update installer workflow with Inno Setup ([#69](https://github.com/egeland/autoortho-rs/issues/69)) ([857bb72](https://github.com/egeland/autoortho-rs/commit/857bb726df417610f4347ae35c394d4e6dfe30fc))
* update Leaflet to 1.9.4 and add XSS protection ([0f49220](https://github.com/egeland/autoortho-rs/commit/0f49220d8a6f92b3ec5a1bf91edee1fb25b76feb))
* update WiX version and add Compressed=no to bypass candle error ([264e1a5](https://github.com/egeland/autoortho-rs/commit/264e1a53778d70c97fb69428af992b03a138ed83))
* use cargo-dist dispatch-releases mode for automated builds ([#39](https://github.com/egeland/autoortho-rs/issues/39)) ([7d6b333](https://github.com/egeland/autoortho-rs/commit/7d6b33367914b493f4c78169d6a0ee6a69ee6777))
* use cargo-dist expanded workflow with dispatch-releases ([#41](https://github.com/egeland/autoortho-rs/issues/41)) ([60c6db8](https://github.com/egeland/autoortho-rs/commit/60c6db8475a5b9620afd20d6a25e384ed58c1dde))
* use chocolatey for Dokan install in release.yml ([#285](https://github.com/egeland/autoortho-rs/issues/285)) ([9c3df4d](https://github.com/egeland/autoortho-rs/commit/9c3df4dc2c96cb9c6eb6e00c2fad4004791d42a8))
* use correct exe path target/distrib/autoortho-x86_64-pc-windows-msvc/ ([#136](https://github.com/egeland/autoortho-rs/issues/136)) ([1bc5cc1](https://github.com/egeland/autoortho-rs/commit/1bc5cc1ce4ebb8a0f7894488c5a354421cf0e9a1))
* use correct Inno Setup silent install flags to prevent hanging ([#211](https://github.com/egeland/autoortho-rs/issues/211)) ([7378348](https://github.com/egeland/autoortho-rs/commit/7378348fef14238a8a0449da6bbcbde9bd194b0e))
* use crates.io dokan API (FileSystemMounter) instead of Drive ([fca561c](https://github.com/egeland/autoortho-rs/commit/fca561c3fd2b0781e2de7f9ac181534b2498db6e))
* use crates.io dokan instead of git ([#272](https://github.com/egeland/autoortho-rs/issues/272)) ([d5c3cdd](https://github.com/egeland/autoortho-rs/commit/d5c3cdd4d20193da5ef9c9cb3687dca46dc55d36))
* use direct MSI install for WinFSP ([#175](https://github.com/egeland/autoortho-rs/issues/175)) ([4d05284](https://github.com/egeland/autoortho-rs/commit/4d0528446ddbbf1cff1e1ccc7d6514c91ce65d0a))
* use dokan crates.io API (Drive builder) instead of old FileSystemMounter ([230fd75](https://github.com/egeland/autoortho-rs/commit/230fd751dafe259c29f08a6c8b185ed2ac059820))
* use full target name 'x86_64-pc-windows-msvc' in MSI build condition ([#128](https://github.com/egeland/autoortho-rs/issues/128)) ([801a490](https://github.com/egeland/autoortho-rs/commit/801a49098f04885cae2a95095fd23fc7ebccca45))
* use ICAO airport codes for SimBrief route preview, rename to User ID Number ([6a7dd47](https://github.com/egeland/autoortho-rs/commit/6a7dd47db69c4d51f82e15edac3f8292c9e3ac74))
* use matrix.targets check for WinFSP install in release workflow ([#173](https://github.com/egeland/autoortho-rs/issues/173)) ([3cffbc5](https://github.com/egeland/autoortho-rs/commit/3cffbc55a09b65ec941ae3ab969a3b8c7d963e8d))
* use NSIS instead of Inno Setup (more reliable) ([#58](https://github.com/egeland/autoortho-rs/issues/58)) ([dda0534](https://github.com/egeland/autoortho-rs/commit/dda0534ac422312522a8e223530cc8ce5a28b4d6))
* use PowerShell filtering ([#66](https://github.com/egeland/autoortho-rs/issues/66)) ([d863d77](https://github.com/egeland/autoortho-rs/commit/d863d770710b4440839330200c9f29e527d06801))
* use prs_created and fromJSON to correctly access PR number ([fbd6610](https://github.com/egeland/autoortho-rs/commit/fbd661080bc22f31d5a53022588301d9adb635fd))
* use reference type for mount_point ([#277](https://github.com/egeland/autoortho-rs/issues/277)) ([74f35c9](https://github.com/egeland/autoortho-rs/commit/74f35c910ebd240fd9f85bcc153c7e6320c84798))
* use RELEASE_TOKEN for release-please permissions ([#18](https://github.com/egeland/autoortho-rs/issues/18)) ([3ad52ac](https://github.com/egeland/autoortho-rs/commit/3ad52aca9c8d7929ce2f12504abe2fc1bd2b75c8))
* use RELEASE_TOKEN to trigger release.yml on tag push ([a191620](https://github.com/egeland/autoortho-rs/commit/a1916200f3118193b31de65a82502cdbe6af99b3))
* use repository_dispatch to trigger release build ([#35](https://github.com/egeland/autoortho-rs/issues/35)) ([284eceb](https://github.com/egeland/autoortho-rs/commit/284eceba3b128ab6c11fac0973ea7b658e8e014f))
* use runner.os to check Windows ([#171](https://github.com/egeland/autoortho-rs/issues/171)) ([9b8a4cd](https://github.com/egeland/autoortho-rs/commit/9b8a4cd1e4c327e31987150deda70394cb6ea731))
* use secrets.GITHUB_TOKEN ([#64](https://github.com/egeland/autoortho-rs/issues/64)) ([58cd5be](https://github.com/egeland/autoortho-rs/commit/58cd5bec4362094ed3d6f2c07b9b9ddd99b67f50))
* use stable Rust toolchain in cross-platform CI ([#13](https://github.com/egeland/autoortho-rs/issues/13)) ([f132930](https://github.com/egeland/autoortho-rs/commit/f13293055d75df9246904633e725b6ed05338b8a))
* use winfsp directly on Windows instead of unifuse ([#91](https://github.com/egeland/autoortho-rs/issues/91)) ([634d87a](https://github.com/egeland/autoortho-rs/commit/634d87a1cf4960746e369140eb48c012c6a6c83b))
* use wix build (not compile), accept OSMF license for WiX v7 ([#122](https://github.com/egeland/autoortho-rs/issues/122)) ([65f8a42](https://github.com/egeland/autoortho-rs/commit/65f8a42fd7059c10c71d666caa182ebe02700e0d))
* use workflow_dispatch to trigger release build ([#30](https://github.com/egeland/autoortho-rs/issues/30)) ([b5617cf](https://github.com/egeland/autoortho-rs/commit/b5617cfa863aae1e889efc4f5e886cef86b9bec2))
* verify copy worked with Get-ChildItem ([#138](https://github.com/egeland/autoortho-rs/issues/138)) ([d40d872](https://github.com/egeland/autoortho-rs/commit/d40d872494850b8db37f451d2eb5ba33a2de6012))
* windows build ([#118](https://github.com/egeland/autoortho-rs/issues/118)) ([d3656b9](https://github.com/egeland/autoortho-rs/commit/d3656b960aa1ccdb9c816fcbe60d81815b16b87c))
* **windows:** bundle winfsp-x64.dll in zip and MSI, graceful init error ([#192](https://github.com/egeland/autoortho-rs/issues/192)) ([0b8dab3](https://github.com/egeland/autoortho-rs/commit/0b8dab3e354b25894434d6fd319949665e7ab9be))
* **windows:** Handle stale WinFsp mounts with retry logic ([#258](https://github.com/egeland/autoortho-rs/issues/258)) ([7a46489](https://github.com/egeland/autoortho-rs/commit/7a46489a7b994046dad99cfefb3811bcedde661e))
* wire DDS in-memory cache size from config ([#12](https://github.com/egeland/autoortho-rs/issues/12)) ([e3df78a](https://github.com/egeland/autoortho-rs/commit/e3df78afa65dd0bbde41904df43defb48c3536f1))
* workflow fix attempt ([#207](https://github.com/egeland/autoortho-rs/issues/207)) ([7eaafa8](https://github.com/egeland/autoortho-rs/commit/7eaafa8e6145db033f0bd3f2a9f4aea12db74fe4))
* workflow fix attempt 3 ([#221](https://github.com/egeland/autoortho-rs/issues/221)) ([1cff66a](https://github.com/egeland/autoortho-rs/commit/1cff66aa57bc1e1a3445e8d86f3766e25d1b094d))
* workflow skip tests on releases ([#234](https://github.com/egeland/autoortho-rs/issues/234)) ([04c9ebc](https://github.com/egeland/autoortho-rs/commit/04c9ebc97547a10ecc900f5f9297b18ea43b36ec))
* **workflow:** restore valid release.yml syntax ([#204](https://github.com/egeland/autoortho-rs/issues/204)) ([48339ab](https://github.com/egeland/autoortho-rs/commit/48339ab08401634ac00714d40249238410b63565))
* **workflow:** separate Inno Setup step from run block ([#205](https://github.com/egeland/autoortho-rs/issues/205)) ([e9c046e](https://github.com/egeland/autoortho-rs/commit/e9c046e031fbf3579b3af9f72a716561ce696b34))


### Performance Improvements

* Consolidate multiple Tokio runtimes into one ([eabaffb](https://github.com/egeland/autoortho-rs/commit/eabaffbc411af18da87218028d13b344ebb2a885))
* optimize fetcher to return Arc&lt;Vec&lt;u8&gt;&gt; instead of cloning ([#101](https://github.com/egeland/autoortho-rs/issues/101)) ([9e481cf](https://github.com/egeland/autoortho-rs/commit/9e481cf2c084ab908870b6925be16c8264659e51)), closes [#87](https://github.com/egeland/autoortho-rs/issues/87)
* Replace Mutex with RwLock in DdsFileSystem ([d8ec852](https://github.com/egeland/autoortho-rs/commit/d8ec85217e1d23b8f6c6fa240869fe5ab2f512c4))
* Share HTTP clients across tile providers ([6c61afe](https://github.com/egeland/autoortho-rs/commit/6c61afe5d8e88387712f0d9c4dcf616f0ffc224c))


### Documentation

* Add configuration reference guide ([3250eb6](https://github.com/egeland/autoortho-rs/commit/3250eb67bfbc1bbecd90be9f40d045b5f058877e))
* Add user guide ([562f464](https://github.com/egeland/autoortho-rs/commit/562f464eda5ca08ab492d199fe481b2e86064ebf))
* Fix duplicate entry in plan file ([400bc1e](https://github.com/egeland/autoortho-rs/commit/400bc1e076a1673107c9bbd31702f62d5f2971d9))
* simplify and update PLAN.md to reflect actual state ([67882a8](https://github.com/egeland/autoortho-rs/commit/67882a834dbf53f669dc692c547d6f91d3d2b953))
* update PLAN.md — mark SimBrief config+UI complete ([39f796c](https://github.com/egeland/autoortho-rs/commit/39f796c72f62995e965e7e9d6390f3a434454a65))
* update PLAN.md and SimBrief plan with current progress ([2a4a722](https://github.com/egeland/autoortho-rs/commit/2a4a722d1125551eb7698cacb15b5acf3e9ac282))
* update PLAN.md to reflect completed custom map integration ([17fc1d7](https://github.com/egeland/autoortho-rs/commit/17fc1d78fea31c461efef6f8eff876e57d71151d))
* update PLAN.md with current session progress ([7311ca2](https://github.com/egeland/autoortho-rs/commit/7311ca23493ece205f4b3a67f0567955e2e21bd4))
* Update README and add installation guide ([1dd28d2](https://github.com/egeland/autoortho-rs/commit/1dd28d2ac99001bbb8069228550c40629375b81b))


### Miscellaneous

* add --nocapture to WiX build for debug output ([21d26c1](https://github.com/egeland/autoortho-rs/commit/21d26c120b871bfd74fdd7bd21ea37443428f905))
* bump manifest to 0.7.1 to clear stuck release-please state ([#286](https://github.com/egeland/autoortho-rs/issues/286)) ([9d3d4c5](https://github.com/egeland/autoortho-rs/commit/9d3d4c5edd9b27ca7489dc7c9e85c26bdc65f31f))
* comment out winfsp until fully implemented ([ae8b323](https://github.com/egeland/autoortho-rs/commit/ae8b3237b6f0875c64e2ab824d951e6438a86255))
* fix workflows ([#103](https://github.com/egeland/autoortho-rs/issues/103)) ([f264661](https://github.com/egeland/autoortho-rs/commit/f264661a06a0da0c88f360ae03921401710aa8cd))
* include agents files ([#255](https://github.com/egeland/autoortho-rs/issues/255)) ([8b1ca42](https://github.com/egeland/autoortho-rs/commit/8b1ca42c026a8d1fd4799b56fcdefcbd94b33814))
* **main:** release 0.1.0 ([9e195c3](https://github.com/egeland/autoortho-rs/commit/9e195c3f92714f22527c8403c76acbf278d456ca))
* **main:** release 0.1.0 ([b8ac591](https://github.com/egeland/autoortho-rs/commit/b8ac59179a4942f30b686ac33a0aeacbdd9345f3))
* **main:** release 0.2.0 ([4d8faeb](https://github.com/egeland/autoortho-rs/commit/4d8faeb424180e77cdaba4e6507dff5d1af05b61))
* **main:** release 0.2.0 ([e9e49e8](https://github.com/egeland/autoortho-rs/commit/e9e49e8530cf6af4ad2392a466f1d229875a5565))
* **main:** release 0.3.0 ([#19](https://github.com/egeland/autoortho-rs/issues/19)) ([af9f16a](https://github.com/egeland/autoortho-rs/commit/af9f16a279b7c194f038de4e83c2927fc932d611))
* **main:** release 0.4.0 ([#21](https://github.com/egeland/autoortho-rs/issues/21)) ([0d8bb4c](https://github.com/egeland/autoortho-rs/commit/0d8bb4c34bb723626f9db70032cbd64127d14ccf))
* **main:** release 0.4.1 ([#23](https://github.com/egeland/autoortho-rs/issues/23)) ([10076a3](https://github.com/egeland/autoortho-rs/commit/10076a399a7d0d03cdb8ba30ed38d60e551523a7))
* **main:** release 0.4.2 ([#25](https://github.com/egeland/autoortho-rs/issues/25)) ([884b0fc](https://github.com/egeland/autoortho-rs/commit/884b0fc08c284189b0a276908731d07428a35847))
* **main:** release 0.4.3 ([#27](https://github.com/egeland/autoortho-rs/issues/27)) ([3c2ca59](https://github.com/egeland/autoortho-rs/commit/3c2ca5992242f5dd509c0141dffc5c117f8ac1c8))
* **main:** release 0.4.4 ([#29](https://github.com/egeland/autoortho-rs/issues/29)) ([0379aa1](https://github.com/egeland/autoortho-rs/commit/0379aa1852ab25722bd8c6f3e120fbbfb7a8031c))
* **main:** release 0.4.5 ([#31](https://github.com/egeland/autoortho-rs/issues/31)) ([0a17c83](https://github.com/egeland/autoortho-rs/commit/0a17c831fc90d6f3858f1471611c98118fa307bc))
* **main:** release 0.5.0 ([#32](https://github.com/egeland/autoortho-rs/issues/32)) ([5c297b2](https://github.com/egeland/autoortho-rs/commit/5c297b28f59c39ad911e218e0ddfb71257dee961))
* **main:** release 0.5.1 ([#34](https://github.com/egeland/autoortho-rs/issues/34)) ([d878bba](https://github.com/egeland/autoortho-rs/commit/d878bba47569f184d8f468138a12dbd6880c91a4))
* **main:** release 0.5.2 ([#36](https://github.com/egeland/autoortho-rs/issues/36)) ([24b1d2c](https://github.com/egeland/autoortho-rs/commit/24b1d2c33c14ab34fc781b11052efaaac4d4fa6d))
* **main:** release 0.5.3 ([#38](https://github.com/egeland/autoortho-rs/issues/38)) ([3f0906f](https://github.com/egeland/autoortho-rs/commit/3f0906f3891edcc87b6c9211b9ca505e7e96861d))
* **main:** release 0.5.4 ([#40](https://github.com/egeland/autoortho-rs/issues/40)) ([f970e3f](https://github.com/egeland/autoortho-rs/commit/f970e3f759b0bb9eaf4c1b1d3c0575abd8cfc74e))
* **main:** release 0.5.5 ([#42](https://github.com/egeland/autoortho-rs/issues/42)) ([bdcd50a](https://github.com/egeland/autoortho-rs/commit/bdcd50accd8602c9df64294907ee03a3cd81b33e))
* **main:** release 0.5.6 ([#44](https://github.com/egeland/autoortho-rs/issues/44)) ([4cdc05f](https://github.com/egeland/autoortho-rs/commit/4cdc05fee04e73e4f0ad12459bc262cd348d0ae6))
* **main:** release 0.5.7 ([#46](https://github.com/egeland/autoortho-rs/issues/46)) ([5ea1a4f](https://github.com/egeland/autoortho-rs/commit/5ea1a4ffc56acfa84337d7ebf1a67c1b9d6ed880))
* **main:** release 0.5.8 ([#49](https://github.com/egeland/autoortho-rs/issues/49)) ([cf9d63d](https://github.com/egeland/autoortho-rs/commit/cf9d63d43f4100007f0d51f8c6d69d718af02c95))
* **main:** release 0.7.0 ([#281](https://github.com/egeland/autoortho-rs/issues/281)) ([2636a12](https://github.com/egeland/autoortho-rs/commit/2636a12e391a06e4392767c8fe4f7738c508ee1b))
* more workflow cleanup ([#104](https://github.com/egeland/autoortho-rs/issues/104)) ([ec48bc9](https://github.com/egeland/autoortho-rs/commit/ec48bc92c2e99bfaf13688fa7331667e5bc33e00))
* release v0.5.8 ([#77](https://github.com/egeland/autoortho-rs/issues/77)) ([420ce2f](https://github.com/egeland/autoortho-rs/commit/420ce2fd7d425d9f4279e050e377bb6d28df27de))
* release v0.6.0 ([#95](https://github.com/egeland/autoortho-rs/issues/95)) ([f7129bc](https://github.com/egeland/autoortho-rs/commit/f7129bca4ca24fe70c556423e90abb0914e98910))
* release v0.6.0 ([#96](https://github.com/egeland/autoortho-rs/issues/96)) ([8f397a3](https://github.com/egeland/autoortho-rs/commit/8f397a30ea75f091f2b300ca95606dbaf697d707))
* release v0.6.1 ([#98](https://github.com/egeland/autoortho-rs/issues/98)) ([2c3e4ef](https://github.com/egeland/autoortho-rs/commit/2c3e4ef6e493306381bc0de24391e4dea8bdb602))
* release v0.6.1 ([#99](https://github.com/egeland/autoortho-rs/issues/99)) ([c9699dd](https://github.com/egeland/autoortho-rs/commit/c9699dd6f8677be81f57c0ede3b8bf5ed211efe8))
* release v0.6.10 ([#125](https://github.com/egeland/autoortho-rs/issues/125)) ([0c6a86d](https://github.com/egeland/autoortho-rs/commit/0c6a86dc7aa107f57279ee60c21d933b7ac6e2ed))
* release v0.6.11 ([#127](https://github.com/egeland/autoortho-rs/issues/127)) ([650c261](https://github.com/egeland/autoortho-rs/commit/650c26140cb1db4139bf5d58042f2de5357481a0))
* release v0.6.12 ([#129](https://github.com/egeland/autoortho-rs/issues/129)) ([0b3ea22](https://github.com/egeland/autoortho-rs/commit/0b3ea220fa80006b71aff2688aadd8867d06b2e9))
* release v0.6.13 ([#131](https://github.com/egeland/autoortho-rs/issues/131)) ([02f1ba1](https://github.com/egeland/autoortho-rs/commit/02f1ba171f3d1466b405d9a7fa51fc805a60225a))
* release v0.6.14 ([#133](https://github.com/egeland/autoortho-rs/issues/133)) ([4d5139e](https://github.com/egeland/autoortho-rs/commit/4d5139e2569e00d01e3e8976945246d6cb811c14))
* release v0.6.15 ([#135](https://github.com/egeland/autoortho-rs/issues/135)) ([2996fd7](https://github.com/egeland/autoortho-rs/commit/2996fd740c61fb969fb743d607e91327f8a791a0))
* release v0.6.16 ([#137](https://github.com/egeland/autoortho-rs/issues/137)) ([461c3bc](https://github.com/egeland/autoortho-rs/commit/461c3bc24d2450d11b76833e3930860ffc822734))
* release v0.6.17 ([#139](https://github.com/egeland/autoortho-rs/issues/139)) ([9d80b71](https://github.com/egeland/autoortho-rs/commit/9d80b713c3e60f228ff9b59de11ed042aa03bda2))
* release v0.6.18 ([#140](https://github.com/egeland/autoortho-rs/issues/140)) ([e3e7006](https://github.com/egeland/autoortho-rs/commit/e3e7006ad945a3b30251b7e9f3dc68d91eba7839))
* release v0.6.19 ([#142](https://github.com/egeland/autoortho-rs/issues/142)) ([d20a240](https://github.com/egeland/autoortho-rs/commit/d20a2408b1b075360bbf694b2552b7fed2abb4f9))
* release v0.6.2 ([#112](https://github.com/egeland/autoortho-rs/issues/112)) ([31e9888](https://github.com/egeland/autoortho-rs/commit/31e98884591b848c8e1f3c7230693137d9d32b2b))
* release v0.6.20 ([#146](https://github.com/egeland/autoortho-rs/issues/146)) ([9562bd6](https://github.com/egeland/autoortho-rs/commit/9562bd617799e9eeff26f2429788b8f8f9a00286))
* release v0.6.21 ([#148](https://github.com/egeland/autoortho-rs/issues/148)) ([649bb7b](https://github.com/egeland/autoortho-rs/commit/649bb7b7f4d779a341de567c211fe289defbdcea))
* release v0.6.22 ([#150](https://github.com/egeland/autoortho-rs/issues/150)) ([6a1e8a2](https://github.com/egeland/autoortho-rs/commit/6a1e8a2b1d3c8b8f6695d4666581c39dfb8c6283))
* release v0.6.23 ([#152](https://github.com/egeland/autoortho-rs/issues/152)) ([99535b0](https://github.com/egeland/autoortho-rs/commit/99535b0a9cd4cbeb7da05304aec21406409b2e83))
* release v0.6.24 ([#154](https://github.com/egeland/autoortho-rs/issues/154)) ([4115c1d](https://github.com/egeland/autoortho-rs/commit/4115c1dc0d88943faea7dff606b6d3bba97f4f4c))
* release v0.6.25 ([#159](https://github.com/egeland/autoortho-rs/issues/159)) ([05ae509](https://github.com/egeland/autoortho-rs/commit/05ae509b8b265086fa5dbd60839bb70eb3f54eb3))
* release v0.6.26 ([#162](https://github.com/egeland/autoortho-rs/issues/162)) ([37ebe9b](https://github.com/egeland/autoortho-rs/commit/37ebe9bdc1ce9270b1184c41fe36deddd2cd3b5b))
* release v0.6.27 ([#164](https://github.com/egeland/autoortho-rs/issues/164)) ([9a4203b](https://github.com/egeland/autoortho-rs/commit/9a4203bada5e64581f17917c829aee290e09d32f))
* release v0.6.28 ([#166](https://github.com/egeland/autoortho-rs/issues/166)) ([a81ff7d](https://github.com/egeland/autoortho-rs/commit/a81ff7d84b98d94ab9e5bc8461a3b3adfc45d6c6))
* release v0.6.29 ([#168](https://github.com/egeland/autoortho-rs/issues/168)) ([33cd2a3](https://github.com/egeland/autoortho-rs/commit/33cd2a373b98fab88bf1c1f348ffc9ff024da300))
* release v0.6.3 ([#114](https://github.com/egeland/autoortho-rs/issues/114)) ([592bdd3](https://github.com/egeland/autoortho-rs/commit/592bdd38798761699caec5b7279e16be5a785c1f))
* release v0.6.30 ([#170](https://github.com/egeland/autoortho-rs/issues/170)) ([1fb1b56](https://github.com/egeland/autoortho-rs/commit/1fb1b5657eebd2fd5eb7e1baacca9f7417f57fea))
* release v0.6.31 ([#172](https://github.com/egeland/autoortho-rs/issues/172)) ([009ab75](https://github.com/egeland/autoortho-rs/commit/009ab75db1be67b38f0925539d1d96529942f973))
* release v0.6.32 ([#174](https://github.com/egeland/autoortho-rs/issues/174)) ([c02c618](https://github.com/egeland/autoortho-rs/commit/c02c61889144cf9c12be6795494badfe8109a1eb))
* release v0.6.33 ([#176](https://github.com/egeland/autoortho-rs/issues/176)) ([6eef1c8](https://github.com/egeland/autoortho-rs/commit/6eef1c83f520c8a513d1ef9739495fee1d83bf9e))
* release v0.6.34 ([#178](https://github.com/egeland/autoortho-rs/issues/178)) ([b4c3018](https://github.com/egeland/autoortho-rs/commit/b4c30183d3b3fb26ea0bbd9ecf301f60f139c49f))
* release v0.6.35 ([#180](https://github.com/egeland/autoortho-rs/issues/180)) ([8d90af8](https://github.com/egeland/autoortho-rs/commit/8d90af8c31861eb0a159836ec8c05c5ac8b65f37))
* release v0.6.36 ([#182](https://github.com/egeland/autoortho-rs/issues/182)) ([6996697](https://github.com/egeland/autoortho-rs/commit/6996697ee7ceb487fb011f29154952c25a1fbcae))
* release v0.6.37 ([#184](https://github.com/egeland/autoortho-rs/issues/184)) ([ce2705e](https://github.com/egeland/autoortho-rs/commit/ce2705e916e7f6e6f34705cc6de82957fface482))
* release v0.6.38 ([#186](https://github.com/egeland/autoortho-rs/issues/186)) ([0ebbf98](https://github.com/egeland/autoortho-rs/commit/0ebbf988782f829df6e955079aad6609d68e643e))
* release v0.6.39 ([#188](https://github.com/egeland/autoortho-rs/issues/188)) ([ff54209](https://github.com/egeland/autoortho-rs/commit/ff542093ea29571a4150d0be5225a9a86e4d0b77))
* release v0.6.4 ([#115](https://github.com/egeland/autoortho-rs/issues/115)) ([69adda5](https://github.com/egeland/autoortho-rs/commit/69adda5f885edafab0daa1330bdf00803c4f8b3d))
* release v0.6.40 ([#191](https://github.com/egeland/autoortho-rs/issues/191)) ([83167a7](https://github.com/egeland/autoortho-rs/commit/83167a7c4d38233a2dfd9068320d888ef1696cae))
* release v0.6.41 ([#193](https://github.com/egeland/autoortho-rs/issues/193)) ([3a23d2f](https://github.com/egeland/autoortho-rs/commit/3a23d2fa1c9eafe0a77e85f4ccc1a23e7b290818))
* release v0.6.42 ([#195](https://github.com/egeland/autoortho-rs/issues/195)) ([cc57de7](https://github.com/egeland/autoortho-rs/commit/cc57de7124366a8552e82d384143641f1a81735e))
* release v0.6.43 ([#197](https://github.com/egeland/autoortho-rs/issues/197)) ([6759598](https://github.com/egeland/autoortho-rs/commit/6759598f7354a2273ca6c6cfe511fbb898d51a62))
* release v0.6.44 ([#199](https://github.com/egeland/autoortho-rs/issues/199)) ([11a926d](https://github.com/egeland/autoortho-rs/commit/11a926d74b6a2a30dfc1abdd355489a15e74cebf))
* release v0.6.45 ([#201](https://github.com/egeland/autoortho-rs/issues/201)) ([5cbbdfd](https://github.com/egeland/autoortho-rs/commit/5cbbdfdaa834a0c9ffab00b0e7ab6063aa627c03))
* release v0.6.46 ([#203](https://github.com/egeland/autoortho-rs/issues/203)) ([f3b9cda](https://github.com/egeland/autoortho-rs/commit/f3b9cda492fd2567efcfed3720e9b297b7d8d8ba))
* release v0.6.47 ([#206](https://github.com/egeland/autoortho-rs/issues/206)) ([2908e8c](https://github.com/egeland/autoortho-rs/commit/2908e8c32d220f0b8655b1140aa5475d1223e3ff))
* release v0.6.48 ([#208](https://github.com/egeland/autoortho-rs/issues/208)) ([1218533](https://github.com/egeland/autoortho-rs/commit/1218533c6a1007814b7b1ddd7141477f3ea7c9d4))
* release v0.6.49 ([#210](https://github.com/egeland/autoortho-rs/issues/210)) ([5407a49](https://github.com/egeland/autoortho-rs/commit/5407a49f5cfd12f4cc6721b7842765cd523e0278))
* release v0.6.5 ([#116](https://github.com/egeland/autoortho-rs/issues/116)) ([a718381](https://github.com/egeland/autoortho-rs/commit/a71838188e300bfef81c6738fd2d8d77b0e08e65))
* release v0.6.50 ([#212](https://github.com/egeland/autoortho-rs/issues/212)) ([b582385](https://github.com/egeland/autoortho-rs/commit/b582385fd0681a3f94c20c309de4d281da3a8337))
* release v0.6.51 ([#214](https://github.com/egeland/autoortho-rs/issues/214)) ([fedab78](https://github.com/egeland/autoortho-rs/commit/fedab78bccb5151b472b60130c847b2fdce5b4c0))
* release v0.6.52 ([#216](https://github.com/egeland/autoortho-rs/issues/216)) ([052c619](https://github.com/egeland/autoortho-rs/commit/052c6193fc33c39aab74e3ec4bbcef22b9ed2889))
* release v0.6.53 ([#218](https://github.com/egeland/autoortho-rs/issues/218)) ([f4db7ac](https://github.com/egeland/autoortho-rs/commit/f4db7accebdb2b9a3937a7a8c32b72aa089fed17))
* release v0.6.54 ([#220](https://github.com/egeland/autoortho-rs/issues/220)) ([bb97f58](https://github.com/egeland/autoortho-rs/commit/bb97f580e6fec749a761ddaa4f50a4654592243f))
* release v0.6.55 ([#228](https://github.com/egeland/autoortho-rs/issues/228)) ([461d42b](https://github.com/egeland/autoortho-rs/commit/461d42b3dbd0f2f664eb3ddc34573b6479656e9b))
* release v0.6.56 ([#230](https://github.com/egeland/autoortho-rs/issues/230)) ([19086f5](https://github.com/egeland/autoortho-rs/commit/19086f51dfa396f02122aeb5bc9d8f6d365de518))
* release v0.6.57 ([#233](https://github.com/egeland/autoortho-rs/issues/233)) ([9fd5bab](https://github.com/egeland/autoortho-rs/commit/9fd5bab2a9f4575a8702d99d2ff8ce6c58a45ade))
* release v0.6.58 ([#237](https://github.com/egeland/autoortho-rs/issues/237)) ([b4b51ff](https://github.com/egeland/autoortho-rs/commit/b4b51ff807cb9d5070ddff8015ab13bca4ad4954))
* release v0.6.59 ([#246](https://github.com/egeland/autoortho-rs/issues/246)) ([0d1521c](https://github.com/egeland/autoortho-rs/commit/0d1521c4839604f9a06c5528318bad738d89f848))
* release v0.6.6 ([#117](https://github.com/egeland/autoortho-rs/issues/117)) ([d7343a1](https://github.com/egeland/autoortho-rs/commit/d7343a1b557b2d5af8cee062d2c2ed395c76d999))
* release v0.6.60 ([#248](https://github.com/egeland/autoortho-rs/issues/248)) ([1506bb1](https://github.com/egeland/autoortho-rs/commit/1506bb1b809b2463ec9f2721c78e42850e8e3dee))
* release v0.6.61 ([#250](https://github.com/egeland/autoortho-rs/issues/250)) ([ffd00f3](https://github.com/egeland/autoortho-rs/commit/ffd00f3d472ba92d641c7a30cc73cf0e4a764ac2))
* release v0.6.62 ([#252](https://github.com/egeland/autoortho-rs/issues/252)) ([a943754](https://github.com/egeland/autoortho-rs/commit/a943754226d54c4b6475b99cd166810ece9271a8))
* release v0.6.63 ([#254](https://github.com/egeland/autoortho-rs/issues/254)) ([7a49d9b](https://github.com/egeland/autoortho-rs/commit/7a49d9bfa60e22c7702570bfa85b52fdbc5e8a75))
* release v0.6.64 ([#259](https://github.com/egeland/autoortho-rs/issues/259)) ([40d00c5](https://github.com/egeland/autoortho-rs/commit/40d00c57b5b37374b47a3d11918467cc26f1a9f2))
* release v0.6.65 ([#262](https://github.com/egeland/autoortho-rs/issues/262)) ([f0a7f68](https://github.com/egeland/autoortho-rs/commit/f0a7f68d1d55deb12caeedd77e6043fb645c1cc5))
* release v0.6.66 ([#270](https://github.com/egeland/autoortho-rs/issues/270)) ([d621627](https://github.com/egeland/autoortho-rs/commit/d621627c8dd8f30283236d6d5983f8bc4e437f16))
* release v0.6.7 ([#119](https://github.com/egeland/autoortho-rs/issues/119)) ([df1c795](https://github.com/egeland/autoortho-rs/commit/df1c79543032a480e3fffd845f5a52fa9211f6ee))
* release v0.6.8 ([#121](https://github.com/egeland/autoortho-rs/issues/121)) ([f045fd5](https://github.com/egeland/autoortho-rs/commit/f045fd544849936a5445b96b4ebdb855ca99f3da))
* release v0.6.9 ([#123](https://github.com/egeland/autoortho-rs/issues/123)) ([4907022](https://github.com/egeland/autoortho-rs/commit/4907022222eb054ea168521a76a4ebe87d3d2e50))
* replace WinFSP with Dokan2 for Windows FUSE ([#271](https://github.com/egeland/autoortho-rs/issues/271)) ([ecc894b](https://github.com/egeland/autoortho-rs/commit/ecc894b949e22560006258b370398da408719453))
* switch from release-plz to release-please ([#279](https://github.com/egeland/autoortho-rs/issues/279)) ([d7b2bc9](https://github.com/egeland/autoortho-rs/commit/d7b2bc90c79a0eed1898262de6d0e013fab97184))
* update dependencies per cargo update ([#232](https://github.com/egeland/autoortho-rs/issues/232)) ([74f00c5](https://github.com/egeland/autoortho-rs/commit/74f00c5b395c2bec5d456fe38cecf71b50e69098))


### Code Refactoring

* code deduplication and quality improvements ([#253](https://github.com/egeland/autoortho-rs/issues/253)) ([b64dfd4](https://github.com/egeland/autoortho-rs/commit/b64dfd4c1d4d7921867a341ed96371850a9f1562))
* Combine CI and release into single workflow ([6e90cf4](https://github.com/egeland/autoortho-rs/commit/6e90cf4630c471aa96f7077901561561e6f88780))
* **config:** group default helper functions before Default impl ([#196](https://github.com/egeland/autoortho-rs/issues/196)) ([88ef113](https://github.com/egeland/autoortho-rs/commit/88ef11390619fa501223bdb1447edd276f009fe1))
* extract common initialization to AppContext ([#74](https://github.com/egeland/autoortho-rs/issues/74)) ([6d770b8](https://github.com/egeland/autoortho-rs/commit/6d770b89bed0b209be9da24f29f6c76ea8b8266d))
* extract hardcoded port 5847 to WEB_UI_PORT constant ([#79](https://github.com/egeland/autoortho-rs/issues/79)) ([706a946](https://github.com/egeland/autoortho-rs/commit/706a946b5bc494de259d982d86895a9c61179a80))
* extract SimBrief prefetch logic to dedicated function ([#78](https://github.com/egeland/autoortho-rs/issues/78)) ([c56e8a1](https://github.com/egeland/autoortho-rs/commit/c56e8a1a1c1ae0f7fe696d0ce36d55e7d75d6ac0))
* Extract UI message handlers to separate module ([e4952bf](https://github.com/egeland/autoortho-rs/commit/e4952bf710c03b701c113cd7d825f335655bc46c))
* move scenery paths from Scenery screen to Settings ([88ab328](https://github.com/egeland/autoortho-rs/commit/88ab3288813723cb3f1efa4225df7d18cd3bf2fc))
* remove optional fuse feature flag ([de4f728](https://github.com/egeland/autoortho-rs/commit/de4f728386ab2d272c96c890c1cc4edfd82e17cc))
* remove release-please, use cargo-dist alone for releases ([#48](https://github.com/egeland/autoortho-rs/issues/48)) ([01250b1](https://github.com/egeland/autoortho-rs/commit/01250b142bf1ba014bc44e10b9f1d0d01d318695))
* replace manual CLI parsing with clap ([#76](https://github.com/egeland/autoortho-rs/issues/76)) ([d9ab86c](https://github.com/egeland/autoortho-rs/commit/d9ab86ccc46a4c1f0ff2f6d72d3ad01379cc78a9))
* use pwall2222/inno-setup-download action ([#219](https://github.com/egeland/autoortho-rs/issues/219)) ([68b01f7](https://github.com/egeland/autoortho-rs/commit/68b01f7fbf3494314c3715e3ba891c3df1dc019b))

## [0.7.0](https://github.com/egeland/autoortho-rs/compare/v0.6.66...v0.7.0) (2026-05-05)


### ⚠ BREAKING CHANGES

* Requires dokan >= 0.3 from crates.io

### Bug Fixes

* add skip-github-release to release-please workflow ([#280](https://github.com/egeland/autoortho-rs/issues/280)) ([ba6f60d](https://github.com/egeland/autoortho-rs/commit/ba6f60dd4ea4e5972350b2669828c918733c3ad2))
* use crates.io dokan API (FileSystemMounter) instead of Drive ([fca561c](https://github.com/egeland/autoortho-rs/commit/fca561c3fd2b0781e2de7f9ac181534b2498db6e))
* use crates.io dokan instead of git ([#272](https://github.com/egeland/autoortho-rs/issues/272)) ([d5c3cdd](https://github.com/egeland/autoortho-rs/commit/d5c3cdd4d20193da5ef9c9cb3687dca46dc55d36))
* use dokan crates.io API (Drive builder) instead of old FileSystemMounter ([230fd75](https://github.com/egeland/autoortho-rs/commit/230fd751dafe259c29f08a6c8b185ed2ac059820))
* use reference type for mount_point ([#277](https://github.com/egeland/autoortho-rs/issues/277)) ([74f35c9](https://github.com/egeland/autoortho-rs/commit/74f35c910ebd240fd9f85bcc153c7e6320c84798))


### Miscellaneous

* replace WinFSP with Dokan2 for Windows FUSE ([#271](https://github.com/egeland/autoortho-rs/issues/271)) ([ecc894b](https://github.com/egeland/autoortho-rs/commit/ecc894b949e22560006258b370398da408719453))
* switch from release-plz to release-please ([#279](https://github.com/egeland/autoortho-rs/issues/279)) ([d7b2bc9](https://github.com/egeland/autoortho-rs/commit/d7b2bc90c79a0eed1898262de6d0e013fab97184))

## [0.6.66](https://github.com/egeland/autoortho-rs/compare/v0.6.65...v0.6.66) - 2026-05-03

### Added

- Replace Windows WinFsp with Dokan for FUSE ([#266](https://github.com/egeland/autoortho-rs/pull/266))

## [0.6.65](https://github.com/egeland/autoortho-rs/compare/v0.6.64...v0.6.65) - 2026-05-02

### Other

- Fix Windows FUSE mount collision issue (fspmount not found) ([#261](https://github.com/egeland/autoortho-rs/pull/261))

## [0.6.64](https://github.com/egeland/autoortho-rs/compare/v0.6.63...v0.6.64) - 2026-05-02

### Added

- create reusable Windows build workflow with artifact support ([#256](https://github.com/egeland/autoortho-rs/pull/256))

### Fixed

- *(windows)* Handle stale WinFsp mounts with retry logic ([#258](https://github.com/egeland/autoortho-rs/pull/258))

## [0.6.63](https://github.com/egeland/autoortho-rs/compare/v0.6.62...v0.6.63) - 2026-05-02

### Other

- code deduplication and quality improvements ([#253](https://github.com/egeland/autoortho-rs/pull/253))

## [0.6.62](https://github.com/egeland/autoortho-rs/compare/v0.6.61...v0.6.62) - 2026-04-30

### Fixed

- add stale mount cleanup before FUSE mount (P1) ([#251](https://github.com/egeland/autoortho-rs/pull/251))

## [0.6.61](https://github.com/egeland/autoortho-rs/compare/v0.6.60...v0.6.61) - 2026-04-29

### Added

- enable Prefetch Route button in dashboard ([#249](https://github.com/egeland/autoortho-rs/pull/249))

## [0.6.60](https://github.com/egeland/autoortho-rs/compare/v0.6.59...v0.6.60) - 2026-04-29

### Added

- add debug mode setting with UI toggle ([#247](https://github.com/egeland/autoortho-rs/pull/247))

## [0.6.59](https://github.com/egeland/autoortho-rs/compare/v0.6.58...v0.6.59) - 2026-04-29

### Fixed

- make FUSE mount conditional to fix Windows build ([#244](https://github.com/egeland/autoortho-rs/pull/244))
- *(ci)* add WinFsp bin to PATH for Windows DLL resolution ([#243](https://github.com/egeland/autoortho-rs/pull/243))
- connect to map, and actually use FUSE, formatted ([#241](https://github.com/egeland/autoortho-rs/pull/241))
- connect to map, and actually use FUSE ([#239](https://github.com/egeland/autoortho-rs/pull/239))

### Other

- Fix/workflow ([#245](https://github.com/egeland/autoortho-rs/pull/245))
- bump lru from 0.16.4 to 0.17.0 ([#227](https://github.com/egeland/autoortho-rs/pull/227))

## [0.6.58](https://github.com/egeland/autoortho-rs/compare/v0.6.57...v0.6.58) - 2026-04-27

### Fixed

- unzip without the root dir ([#236](https://github.com/egeland/autoortho-rs/pull/236))
- workflow skip tests on releases ([#234](https://github.com/egeland/autoortho-rs/pull/234))

## [0.6.57](https://github.com/egeland/autoortho-rs/compare/v0.6.56...v0.6.57) - 2026-04-26

### Fixed

- try geminis workflow files, from web gemini, optimise release ([#231](https://github.com/egeland/autoortho-rs/pull/231))

### Other

- update dependencies per cargo update ([#232](https://github.com/egeland/autoortho-rs/pull/232))

## [0.6.56](https://github.com/egeland/autoortho-rs/compare/v0.6.55...v0.6.56) - 2026-04-26

### Fixed

- try geminis workflow files, from web gemini ([#229](https://github.com/egeland/autoortho-rs/pull/229))

## [0.6.55](https://github.com/egeland/autoortho-rs/compare/v0.6.54...v0.6.55) - 2026-04-26

### Fixed

- workflow fix attempt 3 ([#221](https://github.com/egeland/autoortho-rs/pull/221))

## [0.6.54](https://github.com/egeland/autoortho-rs/compare/v0.6.53...v0.6.54) - 2026-04-25

### Other

- use pwall2222/inno-setup-download action ([#219](https://github.com/egeland/autoortho-rs/pull/219))

## [0.6.53](https://github.com/egeland/autoortho-rs/compare/v0.6.52...v0.6.53) - 2026-04-24

### Fixed

- resolve Inno Setup installation failure ([#217](https://github.com/egeland/autoortho-rs/pull/217))

## [0.6.52](https://github.com/egeland/autoortho-rs/compare/v0.6.51...v0.6.52) - 2026-04-24

### Fixed

- correct Inno Setup installation command and PATH handling ([#215](https://github.com/egeland/autoortho-rs/pull/215))

## [0.6.51](https://github.com/egeland/autoortho-rs/compare/v0.6.50...v0.6.51) - 2026-04-24

### Fixed

- force Node.js 24 for Inno Setup Action on Windows ([#213](https://github.com/egeland/autoortho-rs/pull/213))

## [0.6.50](https://github.com/egeland/autoortho-rs/compare/v0.6.49...v0.6.50) - 2026-04-24

### Fixed

- use correct Inno Setup silent install flags to prevent hanging ([#211](https://github.com/egeland/autoortho-rs/pull/211))

## [0.6.49](https://github.com/egeland/autoortho-rs/compare/v0.6.48...v0.6.49) - 2026-04-23

### Fixed

- update Inno Setup download URL to v6.7.1 ([#209](https://github.com/egeland/autoortho-rs/pull/209))

## [0.6.48](https://github.com/egeland/autoortho-rs/compare/v0.6.47...v0.6.48) - 2026-04-23

### Fixed

- workflow fix attempt ([#207](https://github.com/egeland/autoortho-rs/pull/207))

## [0.6.47](https://github.com/egeland/autoortho-rs/compare/v0.6.46...v0.6.47) - 2026-04-23

### Fixed

- *(workflow)* separate Inno Setup step from run block ([#205](https://github.com/egeland/autoortho-rs/pull/205))

## [0.6.46](https://github.com/egeland/autoortho-rs/compare/v0.6.45...v0.6.46) - 2026-04-21

### Fixed

- try fixing windows builds ([#202](https://github.com/egeland/autoortho-rs/pull/202))

## [0.6.45](https://github.com/egeland/autoortho-rs/compare/v0.6.44...v0.6.45) - 2026-04-20

### Other

- Replace broken MSI installer with Inno Setup installer for Windows ([#200](https://github.com/egeland/autoortho-rs/pull/200))

## [0.6.44](https://github.com/egeland/autoortho-rs/compare/v0.6.43...v0.6.44) - 2026-04-20

### Fixed

- resolve WiX path issue for Windows installer build ([#198](https://github.com/egeland/autoortho-rs/pull/198))

## [0.6.43](https://github.com/egeland/autoortho-rs/compare/v0.6.42...v0.6.43) - 2026-04-19

### Other

- *(config)* group default helper functions before Default impl ([#196](https://github.com/egeland/autoortho-rs/pull/196))

## [0.6.42](https://github.com/egeland/autoortho-rs/compare/v0.6.41...v0.6.42) - 2026-04-17

### Fixed

- *(ci)* stage winfsp DLL to dist profile dir, not release ([#194](https://github.com/egeland/autoortho-rs/pull/194))

## [0.6.41](https://github.com/egeland/autoortho-rs/compare/v0.6.40...v0.6.41) - 2026-04-17

### Fixed

- *(windows)* bundle winfsp-x64.dll in zip and MSI, graceful init error ([#192](https://github.com/egeland/autoortho-rs/pull/192))

## [0.6.40](https://github.com/egeland/autoortho-rs/compare/v0.6.39...v0.6.40) - 2026-04-16

### Added

- Make GUI the default launch mode ([#189](https://github.com/egeland/autoortho-rs/pull/189))

## [0.6.39](https://github.com/egeland/autoortho-rs/compare/v0.6.38...v0.6.39) - 2026-04-15

### Fixed

- correct matrix.targets condition for WinFSP in release.yml ([#187](https://github.com/egeland/autoortho-rs/pull/187))

## [0.6.38](https://github.com/egeland/autoortho-rs/compare/v0.6.37...v0.6.38) - 2026-04-15

### Fixed

- set WINFSP_DIR env var for winfsp-sys build in ci ([#185](https://github.com/egeland/autoortho-rs/pull/185))

## [0.6.37](https://github.com/egeland/autoortho-rs/compare/v0.6.36...v0.6.37) - 2026-04-15

### Fixed

- correct deny.toml syntax ([#183](https://github.com/egeland/autoortho-rs/pull/183))

## [0.6.36](https://github.com/egeland/autoortho-rs/compare/v0.6.35...v0.6.36) - 2026-04-15

### Fixed

- set WINFSP_DIR env var for winfsp-sys build ([#181](https://github.com/egeland/autoortho-rs/pull/181))

## [0.6.35](https://github.com/egeland/autoortho-rs/compare/v0.6.34...v0.6.35) - 2026-04-14

### Other

- update Cargo.lock dependencies

## [0.6.34](https://github.com/egeland/autoortho-rs/compare/v0.6.33...v0.6.34) - 2026-04-14

### Other

- bump zip, tokio, actions/create-github-app-token ([#177](https://github.com/egeland/autoortho-rs/pull/177))

## [0.6.33](https://github.com/egeland/autoortho-rs/compare/v0.6.32...v0.6.33) - 2026-04-14

### Fixed

- use direct MSI install for WinFSP ([#175](https://github.com/egeland/autoortho-rs/pull/175))

## [0.6.32](https://github.com/egeland/autoortho-rs/compare/v0.6.31...v0.6.32) - 2026-04-13

### Fixed

- use matrix.targets check for WinFSP install in release workflow ([#173](https://github.com/egeland/autoortho-rs/pull/173))

## [0.6.31](https://github.com/egeland/autoortho-rs/compare/v0.6.30...v0.6.31) - 2026-04-13

### Fixed

- use runner.os to check Windows ([#171](https://github.com/egeland/autoortho-rs/pull/171))

## [0.6.30](https://github.com/egeland/autoortho-rs/compare/v0.6.29...v0.6.30) - 2026-04-13

### Fixed

- always install WinFSP in release workflow ([#169](https://github.com/egeland/autoortho-rs/pull/169))

## [0.6.29](https://github.com/egeland/autoortho-rs/compare/v0.6.28...v0.6.29) - 2026-04-13

### Fixed

- fix WinFSP install condition - check for 'pc-windows-msvc' target ([#167](https://github.com/egeland/autoortho-rs/pull/167))

## [0.6.28](https://github.com/egeland/autoortho-rs/compare/v0.6.27...v0.6.28) - 2026-04-13

### Fixed

- fix WinFSP install condition in release workflow ([#165](https://github.com/egeland/autoortho-rs/pull/165))

## [0.6.27](https://github.com/egeland/autoortho-rs/compare/v0.6.26...v0.6.27) - 2026-04-13

### Fixed

- install WinFSP in release workflow for Windows build ([#163](https://github.com/egeland/autoortho-rs/pull/163))

## [0.6.26](https://github.com/egeland/autoortho-rs/compare/v0.6.25...v0.6.26) - 2026-04-13

### Fixed

- enable winfsp system feature and fix CI ([#161](https://github.com/egeland/autoortho-rs/pull/161))

## [0.6.25](https://github.com/egeland/autoortho-rs/compare/v0.6.24...v0.6.25) - 2026-04-12

### Fixed

- bundle winfsp-x64.dll in WiX MSI installer ([#158](https://github.com/egeland/autoortho-rs/pull/158))

## [0.6.24](https://github.com/egeland/autoortho-rs/compare/v0.6.23...v0.6.24) - 2026-04-12

### Added

- add TileCoord newtype struct and update PLAN.md ([#153](https://github.com/egeland/autoortho-rs/pull/153))

## [0.6.23](https://github.com/egeland/autoortho-rs/compare/v0.6.22...v0.6.23) - 2026-04-12

### Added

- feat/tune up ([#151](https://github.com/egeland/autoortho-rs/pull/151))

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
