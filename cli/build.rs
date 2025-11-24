// Build script to generate unique fingerprint for each build
use std::process::Command;

fn main() {
    // Generate build fingerprint from git commit + timestamp
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string();

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();

    let fingerprint = format!("{}-{}", git_hash, timestamp);

    // Set as environment variable for compile-time embedding
    println!("cargo:rustc-env=BUILD_FINGERPRINT={}", fingerprint);

    // Rerun if git HEAD changes
    println!("cargo:rerun-if-changed=../.git/HEAD");
}
