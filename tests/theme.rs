//! Locks the UI theme to the colorate brand series (RGB linear blend
//! `#F2904F -> #3B4270`): every chromatic color in `ui/styles.css` must come
//! from the 7-step series; white/black stay allowed as neutrals.

use std::path::PathBuf;

/// RGB linear blend from https://colorate.azurewebsites.net/Gradient/F2904F_3B4270
const SERIES_HEX: [&str; 7] = [
    "f2904f", "d38354", "b5765a", "96695f", "785c65", "594f6a", "3b4270",
];
const SERIES_RGB: [[u16; 3]; 7] = [
    [242, 144, 79],
    [211, 131, 84],
    [181, 118, 90],
    [150, 105, 95],
    [120, 92, 101],
    [89, 79, 106],
    [59, 66, 112],
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
    // Light: navy text on white, warm-brown accent with white text.
    assert!(css.contains("  --fg: #3b4270"), "light text must be navy");
    assert!(
        css.contains("  --accent: #96695f"),
        "light accent must be the warm-brown series step"
    );
    // Dark: navy page, orange accent with navy text.
    assert!(css.contains("  --bg: #3b4270"), "dark page must be navy");
    assert!(
        css.contains("  --accent: #f2904f"),
        "dark accent must be the orange series step"
    );
}
