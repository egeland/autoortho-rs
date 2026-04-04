// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use std::process::Command;
use std::str;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_help() {
        let output = Command::new(env!("CARGO_BIN_EXE_autoortho"))
            .arg("--help")
            .output()
            .expect("Failed to execute --help");

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert!(output.status.success());
        assert!(stdout.contains("AutoOrtho"));
        assert!(stdout.contains("reset-window") || stdout.contains("reset window"));
        assert!(stdout.contains("gui") || stdout.contains("GUI"));
        assert!(stdout.contains("test-tile") || stdout.contains("test tile"));
        assert!(stdout.contains("mount"));
    }

    #[test]
    fn test_cli_reset_window() {
        let output = Command::new(env!("CARGO_BIN_EXE_autoortho"))
            .arg("reset-window")
            .output()
            .expect("Failed to execute reset-window");

        assert!(output.status.success());
    }

    #[test]
    fn test_cli_invalid_arg() {
        let output = Command::new(env!("CARGO_BIN_EXE_autoortho"))
            .arg("--invalid-arg")
            .output()
            .expect("Failed to execute --invalid-arg");

        // Should fail with error message about unknown argument
        assert!(!output.status.success());
        let stderr = str::from_utf8(&output.stderr).unwrap();
        assert!(stderr.contains("error") || stderr.contains("unknown"));
    }

    #[test]
    fn test_cli_test_tile_provider_default() {
        // This will actually try to fetch a tile, but should not panic on parsing
        let output = Command::new(env!("CARGO_BIN_EXE_autoortho"))
            .arg("test-tile")
            .output()
            .expect("Failed to execute test-tile");

        // Should either succeed or fail on network, not on arg parsing
        let stderr = str::from_utf8(&output.stderr).unwrap();
        assert!(!stderr.contains("unrecognized"));
    }

    #[test]
    fn test_cli_test_tile_provider_explicit() {
        let output = Command::new(env!("CARGO_BIN_EXE_autoortho"))
            .arg("test-tile")
            .arg("ARC")
            .output()
            .expect("Failed to execute test-tile ARC");

        // Should not fail on argument parsing
        let stderr = str::from_utf8(&output.stderr).unwrap();
        assert!(!stderr.contains("unrecognized"));
    }

    #[test]
    fn test_cli_mount_help() {
        let output = Command::new(env!("CARGO_BIN_EXE_autoortho"))
            .arg("mount")
            .arg("--help")
            .output()
            .expect("Failed to execute mount --help");

        let stdout = str::from_utf8(&output.stdout).unwrap();
        assert!(output.status.success());
        assert!(stdout.contains("mount") || stdout.contains("Mount"));
    }
}
