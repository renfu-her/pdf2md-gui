//! Locks the UI theme to the Urban Sunset palette (from
//! https://colordrop.io/palette/373): every chromatic color in
//! `ui/styles.css` must come from the 4-color palette; white/black stay
//! allowed as neutrals.

use std::path::PathBuf;

/// Urban Sunset from https://colordrop.io/palette/373
const SERIES_HEX: [&str; 4] = ["393939", "ff5a09", "00303f", "7a9d96"];
const SERIES_RGB: [[u16; 3]; 4] = [
    [57, 57, 57],
    [255, 90, 9],
    [0, 48, 63],
    [122, 157, 150],
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
    // Light: dark-grey text on white, navy accent with white text.
    assert!(
        css.contains("  --fg: #393939"),
        "light text must be the dark grey"
    );
    assert!(
        css.contains("  --accent: #00303f"),
        "light accent must be the navy"
    );
    // Dark: navy page, grey-blue text, grey-blue accent with navy text.
    assert!(css.contains("  --bg: #00303f"), "dark page must be navy");
    assert!(
        css.contains("  --fg: #7a9d96"),
        "dark text must be the grey-blue"
    );
    assert!(
        css.contains("  --accent: #7a9d96"),
        "dark accent must be the grey-blue"
    );
    // Action buttons: solid orange; pressed state visible on both themes
    // (navy on light, dark grey on the navy dark page).
    assert!(
        css.contains("  --action: #ff5a09"),
        "button normal state must be the orange"
    );
    assert!(
        css.contains("  --action-pressed: #00303f"),
        "light pressed state must be navy"
    );
    assert!(
        css.contains("  --action-pressed: #393939"),
        "dark pressed state must be dark grey"
    );
}
