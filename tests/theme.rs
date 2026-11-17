//! Locks the UI theme to the bittersweet series (shades from
//! https://colordrop.io/colors/bittersweet): every chromatic color in
//! `ui/styles.css` must come from the 10-step series; white/black stay
//! allowed as neutrals.

use std::path::PathBuf;

/// Shades of bittersweet from https://colordrop.io/colors/bittersweet
const SERIES_HEX: [&str; 10] = [
    "ffdad6", "ffb1a9", "fe887b", "fe5f4d", "fe3620", "f41901", "c61501", "981001",
    "6b0b01", "3d0600",
];
const SERIES_RGB: [[u16; 3]; 10] = [
    [255, 218, 214],
    [255, 177, 169],
    [254, 136, 123],
    [254, 95, 77],
    [254, 54, 32],
    [244, 25, 1],
    [198, 21, 1],
    [152, 16, 1],
    [107, 11, 1],
    [61, 6, 0],
];

fn css() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(manifest_dir.join("ui").join("styles.css"))
        .expect("ui/styles.css must exist")
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
fn every_chromatic_color_comes_from_the_brand_series() {
    let css = css();
    assert!(!hexes(&css).is_empty(), "theme must declare colors");
    for hex in hexes(&css) {
        assert!(
            SERIES_HEX.contains(&hex.as_str()) || hex == "ffffff" || hex == "000000",
            "#{hex} is outside the brand series (neutrals allowed: white/black)"
        );
    }
    for triple in rgba_triples(&css) {
        assert!(
            SERIES_RGB.contains(&triple) || triple == [0, 0, 0] || triple == [255, 255, 255],
            "rgba{triple:?} is outside the brand series (neutrals allowed: white/black)"
        );
    }
}

#[test]
fn theme_roles_follow_the_series() {
    let css = css().to_lowercase();
    // Light: near-black red text on white, vivid red accent with white text.
    assert!(
        css.contains("  --fg: #3d0600"),
        "light text must be the darkest series step"
    );
    assert!(
        css.contains("  --accent: #f41901"),
        "light accent must be the vivid red series step"
    );
    // Dark: near-black red page, light red accent with dark text.
    assert!(
        css.contains("  --bg: #3d0600"),
        "dark page must be the darkest series step"
    );
    assert!(
        css.contains("  --accent: #fe887b"),
        "dark accent must be a light red series step"
    );
    // Action buttons: one solid color plus a darker pressed state.
    assert!(
        css.contains("  --action: #f41901"),
        "button normal state must be the vivid red series step"
    );
    assert!(
        css.contains("  --action-pressed: #c61501"),
        "button pressed state must be one step darker"
    );
}
