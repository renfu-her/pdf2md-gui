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
    assert!(
        css.contains("background: var(--accent)")
            || css.contains("background:var(--accent)")
            || css.contains("background-color: var(--accent)"),
        "prominent preview actions must use the accent background"
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
