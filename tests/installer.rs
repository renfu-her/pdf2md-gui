//! Locks the exe+setup contract: `installer/pdf2md-gui.iss` must stay in
//! sync with the Cargo package version so `ISCC.exe` always produces a
//! versioned `dist\pdf2md-gui-setup-<version>.exe` from the release exe.

use std::path::PathBuf;

fn iss_source() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(manifest_dir.join("installer").join("pdf2md-gui.iss"))
        .expect("installer/pdf2md-gui.iss must exist")
}

fn iss_define(source: &str, name: &str) -> String {
    let prefix = format!("#define {name} \"");
    for line in source.lines() {
        if let Some(rest) = line.strip_prefix(&prefix)
            && let Some(value) = rest.strip_suffix('"')
        {
            return value.to_string();
        }
    }
    panic!("#define {name} not found in installer/pdf2md-gui.iss");
}

#[test]
fn installer_iss_version_matches_cargo() {
    let source = iss_source();
    assert_eq!(
        iss_define(&source, "MyAppVersion"),
        env!("CARGO_PKG_VERSION"),
        "bump installer/pdf2md-gui.iss > MyAppVersion together with Cargo.toml",
    );
}

#[test]
fn installer_output_artifact_carries_version() {
    let source = iss_source();
    assert!(
        source.contains("OutputBaseFilename=pdf2md-gui-setup-{#MyAppVersion}"),
        "setup exe name must embed MyAppVersion (dist\\pdf2md-gui-setup-<version>.exe)",
    );
    assert!(
        source.contains(r#"Source: "..\target\release\{#MyAppExe}""#),
        "setup must package the release exe (cargo build --release first)",
    );
}
