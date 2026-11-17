#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! pdf2md-gui Tauri backend: thin command layer over [`convert`].
//!
//! All conversion logic (option mapping, page-spec parsing, CLI parity)
//! lives in the library so it stays unit-tested; commands only translate
//! between Tauri IPC types and the core.

use pdf2md_gui::convert::{
    ConvertOptions, build_cli_preview, needs_ocr, pdf_type_label, run_conversion,
    select_dropped_pdf,
};
use tauri::{DragDropEvent, Emitter, WindowEvent};

/// Frontend options from the Upload tab checkboxes/inputs.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrontendOptions {
    compact: bool,
    page_numbers: bool,
    detect_only: bool,
    analyze: bool,
    #[serde(default)]
    page_spec: String,
    #[serde(default)]
    password: String,
}

impl From<FrontendOptions> for ConvertOptions {
    fn from(opts: FrontendOptions) -> Self {
        ConvertOptions {
            compact: opts.compact,
            page_numbers: opts.page_numbers,
            detect_only: opts.detect_only,
            analyze: opts.analyze,
            page_spec: opts.page_spec,
            password: opts.password,
        }
    }
}

/// JSON-serializable conversion report for the Preview tab.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ConvertReport {
    pdf_type: String,
    confidence: f32,
    page_count: u32,
    processing_time_ms: u64,
    markdown: Option<String>,
    needs_ocr: bool,
    pages_needing_ocr: Vec<u32>,
    layout: String,
    warnings: Vec<String>,
}

impl ConvertReport {
    fn from_result(result: pdf_inspector::PdfProcessResult) -> Self {
        let mut warnings = Vec::new();
        if result.has_encoding_issues {
            warnings.push("偵測到字型編碼異常，文字可能缺字；建議改用 OCR 流程。".to_string());
        }
        let mut layout = if result.layout.is_complex {
            "複雜版面".to_string()
        } else {
            "單純".to_string()
        };
        if !result.layout.pages_with_tables.is_empty() {
            layout.push_str(&format!(
                "；含表格頁面：{}",
                join_pages(&result.layout.pages_with_tables)
            ));
        }
        if !result.layout.pages_with_columns.is_empty() {
            layout.push_str(&format!(
                "；多欄頁面：{}",
                join_pages(&result.layout.pages_with_columns)
            ));
        }
        ConvertReport {
            pdf_type: pdf_type_label(&result.pdf_type).to_string(),
            confidence: result.confidence,
            page_count: result.page_count,
            processing_time_ms: result.processing_time_ms,
            markdown: result.markdown,
            needs_ocr: needs_ocr(&result.pdf_type),
            pages_needing_ocr: result.pages_needing_ocr,
            layout,
            warnings,
        }
    }
}

fn join_pages(pages: &[u32]) -> String {
    let mut sorted = pages.to_vec();
    sorted.sort_unstable();
    sorted
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join("、")
}

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn pick_pdf() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("PDF 文件", &["pdf"])
        .pick_file()
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
fn convert_pdf(path: String, opts: FrontendOptions) -> Result<ConvertReport, String> {
    let options = ConvertOptions::from(opts);
    run_conversion(std::path::Path::new(&path), &options)
        .map(ConvertReport::from_result)
        .map_err(|message| message.to_string())
}

#[tauri::command]
fn cli_preview(path: String, output: String, opts: FrontendOptions) -> String {
    build_cli_preview(
        std::path::Path::new(&path),
        &output,
        &ConvertOptions::from(opts),
    )
}

#[tauri::command]
fn pick_save_path(default_name: String) -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Markdown", &["md"])
        .set_file_name(default_name)
        .save_file()
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
fn save_markdown(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            app_version,
            pick_pdf,
            convert_pdf,
            cli_preview,
            pick_save_path,
            save_markdown
        ])
        .on_window_event(|window, event| {
            // Each drop is delivered exactly once by the backend; reuse the
            // same PDF filter as the picker so both paths agree.
            if let WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event {
                match select_dropped_pdf(paths.as_slice()) {
                    Ok(path) => {
                        let _ = window.emit("pdf-dropped", path.to_string_lossy().to_string());
                    }
                    Err(reject) => {
                        let _ = window.emit("pdf-drop-rejected", reject.to_string());
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to run pdf2md-gui");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_options_map_to_convert_options() {
        let opts = FrontendOptions {
            compact: true,
            page_spec: "1,3".to_string(),
            ..FrontendOptions::default()
        };
        let converted = ConvertOptions::from(opts);
        assert!(converted.compact);
        assert_eq!(converted.page_spec, "1,3");
        assert!(!converted.page_numbers);
    }

    #[test]
    fn report_maps_result_fields() {
        let result = pdf_inspector::PdfProcessResult {
            pdf_type: pdf_inspector::PdfType::Mixed,
            markdown: Some("# hi".to_string()),
            page_count: 2,
            processing_time_ms: 7,
            pages_needing_ocr: vec![2],
            ocr_reasons_by_page: Vec::new(),
            title: None,
            author: None,
            subject: None,
            keywords: None,
            creator: None,
            producer: None,
            creation_date: None,
            mod_date: None,
            confidence: 0.9,
            layout: pdf_inspector::LayoutComplexity::default(),
            has_encoding_issues: true,
            cmap_gaps: Vec::new(),
        };
        let report = ConvertReport::from_result(result);
        assert!(!report.needs_ocr);
        assert_eq!(report.pages_needing_ocr, vec![2]);
        assert_eq!(report.markdown.as_deref(), Some("# hi"));
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.layout, "單純");
    }
}
