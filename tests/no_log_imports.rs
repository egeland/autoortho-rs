// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Verify that production code uses `tracing` macros, not `log` directly.
//! See docs/adr/tracing-standard.md

#[test]
fn no_use_log_in_src() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip comments and test modules
            if trimmed.starts_with("//") || trimmed.starts_with("#[cfg(test)]") {
                continue;
            }
            if trimmed.contains("use log::") {
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
        "Found `use log::` imports (should be `use tracing::`):\n{}",
        violations.join("\n")
    );
}
