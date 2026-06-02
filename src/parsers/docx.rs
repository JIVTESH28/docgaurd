use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;
use quick_xml::events::Event;
use quick_xml::Reader;
use super::text::ExtractedDoc;

pub fn parse_docx(content: &[u8]) -> ExtractedDoc {
    let mut metadata = HashMap::new();
    metadata.insert("format".to_string(), "docx".to_string());

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

    match archive.by_name("word/document.xml") {
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
            } else {
                is_corrupted = true;
            }
        }
        Err(_) => {
            is_corrupted = true;
        }
    }

    if let Ok(mut file) = archive.by_name("docProps/core.xml") {
        let mut xml_bytes = Vec::new();
        if file.read_to_end(&mut xml_bytes).is_ok() {
            let mut reader = Reader::from_reader(xml_bytes.as_slice());
            reader.trim_text(true);
            let mut buf = Vec::new();
            let mut current_tag = String::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) => {
                        current_tag = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                    }
                    Ok(Event::Text(e)) => {
                        if let Ok(s) = e.unescape() {
                            let clean_tag = current_tag.split(':').last().unwrap_or(&current_tag);
                            if clean_tag == "creator" || clean_tag == "title" || clean_tag == "subject" || clean_tag == "description" {
                                metadata.insert(clean_tag.to_string(), s.into_owned());
                            }
                        }
                    }
                    Ok(Event::End(_)) => {
                        current_tag.clear();
                    }
                    Ok(Event::Eof) => break,
                    _ => {}
                }
                buf.clear();
            }
        }
    }

    let mut page_count = 0;
    if let Ok(mut file) = archive.by_name("docProps/app.xml") {
        let mut xml_bytes = Vec::new();
        if file.read_to_end(&mut xml_bytes).is_ok() {
            let mut reader = Reader::from_reader(xml_bytes.as_slice());
            reader.trim_text(true);
            let mut buf = Vec::new();
            let mut in_pages = false;
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) => {
                        let name = e.local_name();
                        if name.as_ref() == b"Pages" {
                            in_pages = true;
                        }
                    }
                    Ok(Event::Text(e)) => {
                        if in_pages {
                            if let Ok(s) = e.unescape() {
                                if let Ok(val) = s.parse::<usize>() {
                                    page_count = val;
                                }
                            }
                        }
                    }
                    Ok(Event::End(ref e)) => {
                        let name = e.local_name();
                        if name.as_ref() == b"Pages" {
                            in_pages = false;
                        }
                    }
                    Ok(Event::Eof) => break,
                    _ => {}
                }
                buf.clear();
            }
        }
    }

    if page_count == 0 {
        let char_count = text.chars().count();
        page_count = std::cmp::max(1, ((char_count as f64) / 2500.0).ceil() as usize);
    }
    metadata.insert("total_pages".to_string(), page_count.to_string());

    ExtractedDoc {
        text,
        page_count,
        is_corrupted,
        metadata,
    }
}
