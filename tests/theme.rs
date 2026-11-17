//! Locks the multi-scheme theme system: 5 palettes (Urban Sunset +
//! Autumn Harvest + Midnight Sun + Desert Sunset + Midnight Sky), each with
//! light/dark roles, plus a persisted scheme picker.
//!
//! Sources:
//! - https://colordrop.io/palette/373 (Urban Sunset)
//! - https://colordrop.io/palette/103 (Autumn Harvest)
//! - https://colordrop.io/palette/365 (Midnight Sun)
//! - https://colordrop.io/palette/421 (Desert Sunset)
//! - https://colordrop.io/palette/111 (Midnight Sky)

use std::path::PathBuf;

/// Union of the 5 palettes (20 distinct hexes).
const SERIES_HEX: [&str; 20] = [
    "393939", "ff5a09", "00303f", "7a9d96", // Urban Sunset
    "2d2200", "6d6601", "bf5704", "d99400", // Autumn Harvest
    "3a4660", "c9af98", "ed8a63", "845007", // Midnight Sun
    "0d0b1b", "0f0d21", "cf9b61", "5b6044", // Desert Sunset
    "1c1e24", "283654", "4d638c", "cfe0f9", // Midnight Sky
];
const SERIES_RGB: [[u16; 3]; 13] = [
    [122, 157, 150], // sunset code/notice tints
    [255, 90, 9],
    [217, 148, 0], // harvest tints
    [191, 87, 4],
    [201, 175, 152], // ember tints
    [237, 138, 99],
    [207, 155, 97], // desert tints
    [207, 224, 249], // sky tints
    [77, 99, 140],
    [57, 57, 57],
    [0, 48, 63],
    [91, 96, 68],
    [40, 54, 84],
];

/// (scheme, theme, bg, fg, accent, action, pressed, action-fg, pressed-fg).
/// Sunset-light lives in the base `:root` fallback; every other combo has
/// its own `[data-scheme][data-theme]` block.
type Role = (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str);
const ROLES: [Role; 10] = [
    ("sunset", "light", "ffffff", "393939", "00303f", "ff5a09", "00303f", "ffffff", "ffffff"),
    ("sunset", "dark", "00303f", "7a9d96", "7a9d96", "ff5a09", "393939", "ffffff", "ffffff"),
    ("harvest", "light", "ffffff", "2d2200", "bf5704", "bf5704", "2d2200", "ffffff", "ffffff"),
    ("harvest", "dark", "2d2200", "d99400", "d99400", "bf5704", "6d6601", "ffffff", "ffffff"),
    ("ember", "light", "ffffff", "3a4660", "845007", "845007", "3a4660", "ffffff", "ffffff"),
    ("ember", "dark", "3a4660", "c9af98", "c9af98", "ed8a63", "c9af98", "3a4660", "3a4660"),
    ("desert", "light", "ffffff", "0f0d21", "5b6044", "5b6044", "0f0d21", "ffffff", "ffffff"),
    ("desert", "dark", "0f0d21", "cf9b61", "cf9b61", "cf9b61", "5b6044", "0f0d21", "ffffff"),
    ("sky", "light", "ffffff", "1c1e24", "283654", "4d638c", "1c1e24", "ffffff", "ffffff"),
    ("sky", "dark", "1c1e24", "cfe0f9", "4d638c", "4d638c", "cfe0f9", "ffffff", "1c1e24"),
];

const SCHEMES: [&str; 5] = ["sunset", "harvest", "ember", "desert", "sky"];

fn ui_file(name: &str) -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(manifest_dir.join("ui").join(name))
        .unwrap_or_else(|_| panic!("ui/{name} must exist"))
}

/// Extract the declaration block for one scheme+theme combo.
fn scheme_block<'a>(css: &'a str, scheme: &str, theme: &str) -> &'a str {
    let marker = if scheme == "sunset" && theme == "light" {
        ":root {".to_string()
    } else {
        format!("[data-scheme=\"{scheme}\"][data-theme=\"{theme}\"]")
    };
    let pos = css
        .find(marker.as_str())
        .unwrap_or_else(|| panic!("{marker} block must exist in styles.css"));
    let rest = &css[pos..];
    let open = rest.find('{').expect("rule block must open");
    let close = rest[open..]
        .find('}')
        .expect("rule block must close");
    &rest[open..open + close]
}

fn hexes(css: &str) -> Vec<String> {
    let bytes = css.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 7 <= bytes.len() {
        if bytes[i] == b'#' && bytes[i + 1..i + 7].iter().all(|b| b.is_ascii_hexdigit()) {
            out.push(css[i + 1..i + 7].to_lowercase());
            i += 7;
        } else {
            i += 1;
        }
    }
    out
}

fn rgba_triples(css: &str) -> Vec<[u16; 3]> {
    let mut out = Vec::new();
    let mut rest = css;
    while let Some(pos) = rest.find("rgba(") {
        rest = &rest[pos + 5..];
        let end = rest.find(')').expect("rgba() must close");
        let parts: Vec<&str> = rest[..end].split(',').collect();
        assert_eq!(parts.len(), 4, "rgba() must have 4 components");
        out.push([
            parts[0].trim().parse().expect("red must be a number"),
            parts[1].trim().parse().expect("green must be a number"),
            parts[2].trim().parse().expect("blue must be a number"),
        ]);
        rest = &rest[end..];
    }
    out
}

#[test]
fn every_chromatic_color_comes_from_the_five_palettes() {
    let css = ui_file("styles.css");
    assert!(!hexes(&css).is_empty(), "theme must declare colors");
    for hex in hexes(&css) {
        assert!(
            SERIES_HEX.contains(&hex.as_str()) || hex == "ffffff" || hex == "000000",
            "#{hex} is outside the five palettes (neutrals allowed: white/black)"
        );
    }
    for triple in rgba_triples(&css) {
        assert!(
            SERIES_RGB.contains(&triple) || triple == [0, 0, 0] || triple == [255, 255, 255],
            "rgba{triple:?} is outside the five palettes (neutrals allowed: white/black)"
        );
    }
}

#[test]
fn each_scheme_theme_combo_uses_its_roles() {
    let css = ui_file("styles.css").to_lowercase();
    for (scheme, theme, bg, fg, accent, action, pressed, afg, pfg) in ROLES {
        let block = scheme_block(&css, scheme, theme);
        for (var, hex) in [
            ("--bg", bg),
            ("--fg", fg),
            ("--accent", accent),
            ("--action", action),
            ("--action-pressed", pressed),
            ("--action-fg", afg),
            ("--action-pressed-fg", pfg),
        ] {
            assert!(
                block.contains(&format!("{var}: #{hex}")),
                "{scheme}/{theme} must define {var} as #{hex}"
            );
        }
    }
}

#[test]
fn scheme_picker_exists_and_choice_is_persisted() {
    let html = ui_file("index.html");
    for scheme in SCHEMES {
        assert!(
            html.contains(&format!("data-scheme-choice=\"{scheme}\"")),
            "picker must offer the {scheme} scheme"
        );
    }
    let js = ui_file("app.js");
    assert!(
        js.contains("data-scheme"),
        "switching logic must set the data-scheme attribute"
    );
    assert!(
        js.contains("localStorage"),
        "scheme choice must be persisted across launches"
    );
    assert!(
        js.contains("matchMedia"),
        "auto theme must keep following the system"
    );
}
