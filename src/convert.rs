//! Conversion core shared by the GUI and the CLI preview.
//!
//! The option mapping below mirrors the upstream `pdf2md` binary
//! (`--compact`, `--pages`, `--select-pages`, `--password`,
//! `--detect-only`, `--analyze`), so a button/input-driven conversion in the
//! GUI behaves exactly like its displayed CLI command.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use pdf_inspector::{
    MarkdownProfile, PdfOptions, PdfProcessResult, PdfType, ProcessMode, process_pdf_with_options,
};

/// GUI-side options; each field maps 1:1 to a `pdf2md` CLI flag.
#[derive(Debug, Clone, Default)]
pub struct ConvertOptions {
    /// `--compact`: token-efficient markdown profile.
    pub compact: bool,
    /// `--pages`: insert `<!-- Page N -->` markers between pages.
    pub page_numbers: bool,
    /// `--detect-only`: classification only, no extraction.
    pub detect_only: bool,
    /// `--analyze`: detection + layout analysis, no markdown.
    pub analyze: bool,
    /// `--select-pages`: e.g. `1,3,5-10`. Empty = all pages.
    pub page_spec: String,
    /// `--password`: password for encrypted PDFs. Empty = none.
    pub password: String,
}

impl ConvertOptions {
    /// Mirror upstream precedence: `--detect-only` wins over `--analyze`.
    pub fn process_mode(&self) -> ProcessMode {
        if self.detect_only {
            ProcessMode::DetectOnly
        } else if self.analyze {
            ProcessMode::Analyze
        } else {
            ProcessMode::Full
        }
    }
}

/// Parse a `--select-pages` spec like `1,3,5-10`.
///
/// Mirrors `parse_page_spec` in the upstream `pdf2md` binary: comma-separated
/// items, inclusive `start-end` ranges, 1-indexed pages, same error messages.
pub fn parse_page_spec(spec: &str) -> Result<HashSet<u32>, String> {
    let mut pages = HashSet::new();
    for part in spec.split(',') {
        let part = part.trim();
        if let Some((start, end)) = part.split_once('-') {
            let start: u32 = start
                .trim()
                .parse()
                .map_err(|_| format!("invalid page number: {}", start.trim()))?;
            let end: u32 = end
                .trim()
                .parse()
                .map_err(|_| format!("invalid page number: {}", end.trim()))?;
            if start == 0 || end == 0 {
                return Err("page numbers are 1-indexed".to_string());
            }
            if start > end {
                return Err(format!("invalid range: {start}-{end}"));
            }
            pages.extend(start..=end);
        } else {
            let page: u32 = part
                .parse()
                .map_err(|_| format!("invalid page number: {part}"))?;
            if page == 0 {
                return Err("page numbers are 1-indexed".to_string());
            }
            pages.insert(page);
        }
    }
    Ok(pages)
}

/// Run the conversion with the given GUI options.
///
/// Mirrors the non-OCR path of `pdf2md`: compact profile, page markers, page
/// filter and password are applied to [`PdfOptions`], then
/// [`process_pdf_with_options`] is called.
pub fn run_conversion(pdf_path: &Path, opts: &ConvertOptions) -> Result<PdfProcessResult, String> {
    let mut options = PdfOptions::new().mode(opts.process_mode());
    if opts.compact {
        options.markdown.profile = MarkdownProfile::Compact;
    }
    options.markdown.include_page_numbers = opts.page_numbers;

    let spec = opts.page_spec.trim();
    if !spec.is_empty() {
        options.page_filter = Some(parse_page_spec(spec)?);
    }

    let password = opts.password.trim();
    if !password.is_empty() {
        options.password = Some(password.to_string());
    }

    process_pdf_with_options(pdf_path, options).map_err(|e| e.to_string())
}

/// Traditional Chinese label for a detected [`PdfType`].
pub fn pdf_type_label(pdf_type: &PdfType) -> &'static str {
    match pdf_type {
        PdfType::TextBased => "文字型（可直接擷取）",
        PdfType::Scanned => "掃描型（需要 OCR）",
        PdfType::ImageBased => "圖片型（需要 OCR）",
        PdfType::Mixed => "混合型（部分頁面需要 OCR）",
    }
}

/// Mirror of the `pdf2md` exit-2 rule: scanned / image-only PDFs carry no
/// usable text layer, so the GUI shows an OCR notice instead of markdown.
pub fn needs_ocr(pdf_type: &PdfType) -> bool {
    matches!(pdf_type, PdfType::Scanned | PdfType::ImageBased)
}

/// Reason a drag-and-drop payload was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropReject {
    /// Nothing was dropped.
    Empty,
    /// Files were dropped, but none of them is a PDF.
    NoPdf,
}

impl std::fmt::Display for DropReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DropReject::Empty => write!(f, "沒有拖入任何檔案"),
            DropReject::NoPdf => write!(f, "只接受 .pdf 檔案"),
        }
    }
}

/// Case-insensitive `.pdf` extension check, shared by drop filtering and the
/// hover hint.
pub fn has_pdf_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
}

/// Default `.md` output for a newly chosen PDF: same stem, `.md` extension.
/// Returns `None` when the input has no `.pdf` extension so callers leave a
/// manually entered output alone instead of guessing.
pub fn default_markdown_output(pdf_path: &Path) -> Option<PathBuf> {
    if has_pdf_extension(pdf_path) {
        Some(pdf_path.with_extension("md"))
    } else {
        None
    }
}

/// Pick the PDF to accept from a drag-and-drop payload.
///
/// The first `.pdf` (case-insensitive) wins; anything else in the payload is
/// ignored so a mixed drop still loads the PDF.
pub fn select_dropped_pdf(paths: &[PathBuf]) -> Result<PathBuf, DropReject> {
    if paths.is_empty() {
        return Err(DropReject::Empty);
    }
    paths
        .iter()
        .find(|path| has_pdf_extension(path))
        .cloned()
        .ok_or(DropReject::NoPdf)
}

fn quote_arg(value: &str) -> String {
    if value.contains([' ', '\t', '"']) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

/// Build the equivalent `pdf2md` command line for display/copy.
///
/// The password is masked as `****` so the preview is safe to share.
pub fn build_cli_preview(pdf_path: &Path, output: &str, opts: &ConvertOptions) -> String {
    let mut cmd = format!("pdf2md {}", quote_arg(&pdf_path.to_string_lossy()));

    let output = output.trim();
    if !output.is_empty() {
        cmd.push_str(&format!(" {}", quote_arg(output)));
    }
    if opts.compact {
        cmd.push_str(" --compact");
    }
    if opts.page_numbers {
        cmd.push_str(" --pages");
    }
    let spec = opts.page_spec.trim();
    if !spec.is_empty() {
        cmd.push_str(&format!(" --select-pages {}", quote_arg(spec)));
    }
    if !opts.password.trim().is_empty() {
        cmd.push_str(" --password ****");
    }
    if opts.detect_only {
        cmd.push_str(" --detect-only");
    } else if opts.analyze {
        cmd.push_str(" --analyze");
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_spec_single_pages_and_ranges() {
        let pages = parse_page_spec("1,3,5-7").unwrap();
        assert_eq!(pages, HashSet::from([1, 3, 5, 6, 7]));
    }

    #[test]
    fn page_spec_rejects_zero_and_reversed_ranges() {
        assert!(parse_page_spec("0").is_err());
        assert!(parse_page_spec("5-2").is_err());
        assert!(parse_page_spec("abc").is_err());
    }

    #[test]
    fn cli_preview_contains_every_enabled_flag() {
        let opts = ConvertOptions {
            compact: true,
            page_numbers: true,
            page_spec: "1,3".to_string(),
            password: "secret".to_string(),
            ..ConvertOptions::default()
        };
        let cmd = build_cli_preview(Path::new("in.pdf"), "out.md", &opts);
        assert_eq!(
            cmd,
            "pdf2md in.pdf out.md --compact --pages --select-pages 1,3 --password ****"
        );
    }

    #[test]
    fn detect_only_wins_over_analyze_like_upstream() {
        let opts = ConvertOptions {
            detect_only: true,
            analyze: true,
            ..ConvertOptions::default()
        };
        assert_eq!(opts.process_mode(), ProcessMode::DetectOnly);
    }

    #[test]
    fn drop_picks_first_pdf_case_insensitively() {
        let picked = select_dropped_pdf(&[
            PathBuf::from("notes.txt"),
            PathBuf::from("scan.PDF"),
            PathBuf::from("doc.pdf"),
        ])
        .unwrap();
        assert_eq!(picked, PathBuf::from("scan.PDF"));
    }

    #[test]
    fn drop_without_any_pdf_is_rejected() {
        assert_eq!(
            select_dropped_pdf(&[PathBuf::from("a.txt")]),
            Err(DropReject::NoPdf)
        );
    }

    #[test]
    fn drop_with_no_files_is_rejected() {
        assert_eq!(select_dropped_pdf(&[]), Err(DropReject::Empty));
    }

    #[test]
    fn new_pdf_resets_default_output_to_same_stem() {
        assert_eq!(
            default_markdown_output(Path::new("D:\\docs\\report.pdf")),
            Some(PathBuf::from("D:\\docs\\report.md"))
        );
        assert_eq!(
            default_markdown_output(Path::new("scan.PDF")),
            Some(PathBuf::from("scan.md"))
        );
        assert_eq!(default_markdown_output(Path::new("notes.txt")), None);
    }
}
