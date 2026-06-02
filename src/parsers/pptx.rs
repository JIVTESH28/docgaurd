use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;
use quick_xml::events::Event;
use quick_xml::Reader;
use super::text::ExtractedDoc;

pub fn parse_pptx(content: &[u8]) -> ExtractedDoc {
    let mut metadata = HashMap::new();
    metadata.insert("format".to_string(), "pptx".to_string());

    let cursor = Cursor::new(content);
    let mut archive = match ZipArchive::new(cursor) {
        Ok(a) => a,
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
    let mut is_corrupted = false;
    let mut slide_names = Vec::new();

    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_names.push(name);
            }
        }
    }

    slide_names.sort_by(|a, b| {
        let extract_num = |s: &str| -> u32 {
            s.strip_prefix("ppt/slides/slide")
                .and_then(|rem| rem.strip_suffix(".xml"))
                .and_then(|num_str| num_str.parse::<u32>().ok())
                .unwrap_or(0)
        };
        extract_num(a).cmp(&extract_num(b))
    });

    let slide_count = slide_names.len();

    for slide_name in &slide_names {
        match archive.by_name(slide_name) {
            Ok(mut file) => {
                let mut xml_bytes = Vec::new();
                if file.read_to_end(&mut xml_bytes).is_ok() {
                    let mut reader = Reader::from_reader(xml_bytes.as_slice());
                    reader.trim_text(true);
                    let mut buf = Vec::new();
                    loop {
                        match reader.read_event_into(&mut buf) {
                            Ok(Event::Text(e)) => {
                                if let Ok(s) = e.unescape() {
                                    text.push_str(&s);
                                    text.push(' ');
                                }
                            }
                            Ok(Event::Eof) => break,
                            Err(_) => {
                                is_corrupted = true;
                                break;
                            }
                            _ => {}
                        }
                        buf.clear();
                    }
                    text.push('\n');
                } else {
                    is_corrupted = true;
                }
            }
            Err(_) => {
                is_corrupted = true;
            }
        }
    }

    let page_count = std::cmp::max(1, slide_count);
    metadata.insert("slides".to_string(), slide_count.to_string());
    metadata.insert("total_pages".to_string(), page_count.to_string());

    ExtractedDoc {
        text,
        page_count,
        is_corrupted,
        metadata,
    }
}
