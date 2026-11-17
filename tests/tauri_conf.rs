//! Keeps `tauri.conf.json > version` in sync with the Cargo package version.

use std::path::PathBuf;

#[test]
fn tauri_config_version_matches_cargo() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let raw = std::fs::read_to_string(manifest_dir.join("tauri.conf.json"))
        .expect("tauri.conf.json must exist");
    let config: serde_json::Value =
        serde_json::from_str(&raw).expect("tauri.conf.json must be valid JSON");
    assert_eq!(
        config.get("version").and_then(|v| v.as_str()),
        Some(env!("CARGO_PKG_VERSION")),
        "bump tauri.conf.json > version together with Cargo.toml",
    );
}
