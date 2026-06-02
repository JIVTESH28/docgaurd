use std::collections::HashMap;
use std::io::Cursor;
use calamine::{open_workbook_from_rs, Reader, Xlsx};
use super::text::ExtractedDoc;

pub fn parse_xlsx(content: &[u8]) -> ExtractedDoc {
    let mut metadata = HashMap::new();
    metadata.insert("format".to_string(), "xlsx".to_string());

    let cursor = Cursor::new(content);
    let mut workbook: Xlsx<Cursor<&[u8]>> = match open_workbook_from_rs(cursor) {
        Ok(w) => w,
        Err(_) => {
            return ExtractedDoc {
                text: String::new(),
                page_count: 0,
                is_corrupted: true,
                metadata,
            };
        }
    };

    let mut text = String::new();
    let mut sheet_count = 0;
    let mut total_cells = 0;
    let mut empty_cells = 0;

    // Retrieve sheet names and parse each sheet
    let sheet_names = workbook.sheet_names().to_vec();
    for sheet_name in &sheet_names {
        if let Ok(range) = workbook.worksheet_range(sheet_name) {
            sheet_count += 1;
            text.push_str(sheet_name);
            text.push('\n');
            
            for row in range.rows() {
                for cell in row {
                    let cell_str = cell.to_string();
                    if !cell_str.is_empty() {
                        total_cells += 1;
                        text.push_str(&cell_str);
                        text.push(' ');
                    } else {
                        empty_cells += 1;
                    }
                }
                text.push('\n');
            }
        }
    }

    let page_count = std::cmp::max(1, sheet_count);
    metadata.insert("sheets".to_string(), sheet_count.to_string());
    metadata.insert("total_cells".to_string(), total_cells.to_string());
    metadata.insert("empty_cells".to_string(), empty_cells.to_string());
    metadata.insert("total_pages".to_string(), page_count.to_string());

    ExtractedDoc {
        text,
        page_count,
        is_corrupted: false,
        metadata,
    }
}
