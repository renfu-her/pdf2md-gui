//! Preview-tab action contract: copy-full / save-md must be prominent and
//! the save destination must stay visible inside the Preview tab.
//!
//! Seam: static Preview UI (`ui/index.html`, `ui/styles.css`, `ui/app.js`)
//! observed as text, mirroring the existing `tests/installer.rs` style.

use std::path::PathBuf;

fn ui_file(name: &str) -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(manifest_dir.join("ui").join(name))
        .unwrap_or_else(|_| panic!("ui/{name} must exist"))
}

fn rule_block<'a>(css: &'a str, selector: &str) -> &'a str {
    let pos = css
        .find(selector)
        .unwrap_or_else(|| panic!("{selector} must be styled in styles.css"));
    let rest = &css[pos..];
    let open = rest.find('{').expect("rule block must open");
    let close = rest.find('}').expect("rule block must close");
    &rest[open..close]
}

fn preview_section(html: &str) -> &str {
    let start = html
        .find("id=\"tab-preview\"")
        .expect("preview section must exist");
    let section_start = html[..start].rfind("<section").expect("section tag");
    let rest = &html[section_start..];
    let end = rest.find("</section>").expect("section end") + "</section>".len();
    &rest[..end]
}

#[test]
fn preview_tab_has_own_visible_status() {
    let html = ui_file("index.html");
    let preview = preview_section(&html);
    assert!(
        preview.contains("id=\"previewStatus\""),
        "preview tab needs its own status element so save/copy feedback is visible without switching tabs"
    );
    assert!(
        preview.contains("role=\"status\""),
        "preview status must be exposed as role=status"
    );
}

#[test]
fn preview_copy_and_save_buttons_are_prominent() {
    let html = ui_file("index.html");
    let css = ui_file("styles.css");
    let preview = preview_section(&html);

    for id in ["id=\"btnCopyMd\"", "id=\"btnSaveMd\""] {
        let pos = preview.find(id).unwrap_or_else(|| panic!("{id} must live in preview tab"));
        let tag_start = preview[..pos].rfind('<').expect("button tag");
        let tag = &preview[tag_start..preview[pos..].find('>').unwrap_or(0) + pos + 1];
        assert!(
            tag.contains("preview-action") || tag.contains("primary"),
            "{id} must carry a prominent action class, got: {tag}"
        );
    }
    assert!(
        css.contains(".preview-action") || css.contains("btnCopyMd"),
        "styles.css must style the preview actions prominently"
    );
    // Brand action: one solid color, darker while pressed (see CHANGED.md).
    // Hexes live once in `:root`; buttons reference them via variables.
    let root = rule_block(&css, ":root").to_lowercase();
    for (var, hex) in [
        ("--action", "#ff5a09"),
        ("--action-pressed", "#00303f"),
        ("--action-pressed-fg", "#ffffff"),
    ] {
        assert!(
            root.contains(&format!("{var}: {hex}")),
            ":root must define {var} as {hex}"
        );
    }
    for selector in ["button.preview-action {", "button.primary {"] {
        let block = rule_block(&css, selector).to_lowercase();
        assert!(
            block.contains("background: var(--action)"),
            "{selector} must use the solid brand color"
        );
        assert!(
            !block.contains("linear-gradient"),
            "{selector} must be solid, not gradient"
        );
    }
    let pressed = rule_block(&css, ":active").to_lowercase();
    assert!(
        pressed.contains("var(--action-pressed)")
            && pressed.contains("var(--action-pressed-fg)"),
        "pressed buttons must use the pressed brand step and its text color"
    );
}

#[test]
fn save_destination_is_shown_inside_preview() {
    let html = ui_file("index.html");
    let js = ui_file("app.js");
    let preview = preview_section(&html);
    assert!(
        preview.contains("id=\"savePath\"") || preview.contains("id=\"previewStatus\""),
        "preview tab must show where the .md was saved"
    );
    assert!(
        js.contains("previewStatus"),
        "save/copy feedback must update the preview-visible status, not only the upload tab"
    );
    assert!(
        js.contains("已儲存"),
        "saving must render the destination path (\"已儲存：<path>\")"
    );
}
