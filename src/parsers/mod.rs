pub mod pdf;
pub mod docx;
pub mod pptx;
pub mod xlsx;
pub mod text;

pub use text::ExtractedDoc;
use std::collections::HashMap;

pub fn parse_document(content: &[u8], file_name: &str) -> ExtractedDoc {
    let ext = file_name.split('.').last().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => pdf::parse_pdf(content),
        "docx" => docx::parse_docx(content),
        "pptx" => pptx::parse_pptx(content),
        "xlsx" => xlsx::parse_xlsx(content),
        "txt" => text::parse_txt(content),
        "md" => text::parse_md(content),
        "csv" => text::parse_csv(content),
        "json" => text::parse_json(content),
        "xml" => text::parse_xml(content),
        "html" | "htm" => text::parse_html(content),
        _ => {
            let mut metadata = HashMap::new();
            metadata.insert("unsupported_format".to_string(), "true".to_string());
            ExtractedDoc {
                text: String::new(),
                page_count: 0,
                is_corrupted: true,
                metadata,
            }
        }
    }
}
