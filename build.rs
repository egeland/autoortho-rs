use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Try to fetch current Chrome stable version
    let ua_string = fetch_chrome_ua()
        .expect("Failed to fetch Chrome stable version from network — cannot build without it");

    // Write the User-Agent to a Rust module
    let ua_module = format!(
        r#"// Auto-generated at build time from Chrome release channels
pub const CHROME_USER_AGENT: &str = "{ua}";
"#,
        ua = ua_string.replace('\\', "\\\\").replace('"', "\\\"")
    );

    fs::write(format!("{}/user_agent.rs", out_dir), ua_module).unwrap();

    // Tell Cargo to rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

fn fetch_chrome_ua() -> Option<String> {
    // Fetch Chrome stable version from Google
    let output = Command::new("curl")
        .args([
            "-s",
            "https://googlechromelabs.github.io/chrome-for-testing/LATEST_RELEASE_STABLE",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Construct User-Agent string with the fetched version
    let ua = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
        version
    );

    Some(ua)
}
