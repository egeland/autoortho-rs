// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Verify that MockProvider and FailingProvider are defined once in test_utils,
//! not duplicated across test modules.

#[test]
fn mock_provider_single_definition() {
    let test_utils = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/test_utils.rs"),
    )
    .unwrap();

    assert!(
        test_utils.contains("pub struct MockProvider"),
        "MockProvider should be defined in test_utils.rs"
    );
    assert!(
        test_utils.contains("pub struct FailingProvider"),
        "FailingProvider should be defined in test_utils.rs"
    );
}

#[test]
fn no_mock_provider_in_tiles_or_fuse() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let path = entry.path();
        // Skip test_utils.rs itself
        if path.file_name().unwrap_or_default() == "test_utils.rs" {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("struct MockProvider") || trimmed.contains("struct FailingProvider") {
                violations.push(format!(
                    "{}:{}: {}",
                    path.strip_prefix(&src_dir).unwrap().display(),
                    line_num + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "MockProvider/FailingProvider should be in test_utils.rs, not duplicated:\n{}",
        violations.join("\n")
    );
}
