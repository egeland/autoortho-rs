// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Verify dead code has been removed.
//! See architecture review: ~/tmp/architecture-review-2026-06-25.md

use std::path::Path;

#[test]
fn no_tile_service_trait_in_src() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("trait TileService") {
                violations.push(format!(
                    "{}:{}: {}",
                    entry.path().strip_prefix(&src_dir).unwrap().display(),
                    line_num + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found `trait TileService` definition (should be deleted):\n{}",
        violations.join("\n")
    );
}

#[test]
fn no_tile_service_module_in_services() {
    let mod_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("services")
        .join("mod.rs");
    let content = std::fs::read_to_string(&mod_file).unwrap();
    assert!(
        !content.contains("tile_service"),
        "services/mod.rs still references tile_service module"
    );
}
