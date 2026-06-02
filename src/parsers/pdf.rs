use std::collections::HashMap;
use lopdf::{Document, Object};
use std::io::Cursor;
use super::text::ExtractedDoc;

fn object_to_string(doc: &Document, obj: &Object) -> Option<String> {
    match obj {
        Object::String(bytes, _) => {
            Some(String::from_utf8_lossy(bytes).into_owned())
        }
        Object::Reference(ref_id) => {
            if let Ok(deref_obj) = doc.get_object(*ref_id) {
                object_to_string(doc, deref_obj)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn parse_pdf(content: &[u8]) -> ExtractedDoc {
    let mut metadata = HashMap::new();
    metadata.insert("format".to_string(), "pdf".to_string());
    
    let cursor = Cursor::new(content);
    let doc = match Document::load_from(cursor) {
        Ok(d) => d,
        Err(_) => {
            return ExtractedDoc {
                text: String::new(),
                page_count: 0,
                is_corrupted: true,
                metadata,
            };
        }
    };

    let page_ids = doc.get_pages();
    let page_count = page_ids.len();

    let page_numbers: Vec<u32> = page_ids.keys().cloned().collect();
    let mut extracted_text = String::new();
    let mut failed_pages = 0;

    for page_num in &page_numbers {
        match doc.extract_text(&[*page_num]) {
            Ok(t) => {
                extracted_text.push_str(&t);
                extracted_text.push(' ');
            }
            Err(_) => {
                failed_pages += 1;
            }
        }
    }

    // Extract Info metadata if present
    let mut info_dict = None;
    if let Ok(info_obj) = doc.trailer.get(b"Info") {
        match info_obj {
            Object::Reference(ref_id) => {
                if let Ok(deref_obj) = doc.get_object(*ref_id) {
                    if let Ok(dict) = deref_obj.as_dict() {
                        info_dict = Some(dict);
                    }
                }
            }
            Object::Dictionary(dict) => {
                info_dict = Some(dict);
            }
            _ => {}
        }
    }

    if let Some(dict) = info_dict {
        let keys_to_extract: Vec<(&str, &[u8])> = vec![
            ("Title", b"Title"),
            ("Author", b"Author"),
            ("Creator", b"Creator"),
            ("Producer", b"Producer"),
            ("CreationDate", b"CreationDate"),
            ("ModDate", b"ModDate"),
        ];
        for (meta_name, key_bytes) in keys_to_extract {
            if let Ok(obj) = dict.get(key_bytes) {
                if let Some(val_str) = object_to_string(&doc, obj) {
                    metadata.insert(meta_name.to_string(), val_str);
                }
            }
        }
    }

    metadata.insert("failed_pages".to_string(), failed_pages.to_string());
    metadata.insert("total_pages".to_string(), page_count.to_string());

    ExtractedDoc {
        text: extracted_text,
        page_count,
        is_corrupted: false,
        metadata,
    }
}
