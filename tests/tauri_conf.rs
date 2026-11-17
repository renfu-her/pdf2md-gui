//! Keeps `tauri.conf.json > version` in sync with the Cargo package version,
//! and locks the main window geometry (512x610, fixed, centered).

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

#[test]
fn main_window_is_fixed_512x610_centered() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let raw = std::fs::read_to_string(manifest_dir.join("tauri.conf.json"))
        .expect("tauri.conf.json must exist");
    let config: serde_json::Value =
        serde_json::from_str(&raw).expect("tauri.conf.json must be valid JSON");
    let window = config
        .pointer("/app/windows/0")
        .expect("tauri.conf.json must declare app.windows[0]");
    assert_eq!(window.get("width").and_then(|v| v.as_u64()), Some(512));
    assert_eq!(
        window.get("height").and_then(|v| v.as_u64()),
        Some(610),
        "main window height must be 610 (was 640)",
    );
    assert_eq!(window.get("resizable").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(window.get("center").and_then(|v| v.as_bool()), Some(true));
}
