use std::collections::HashMap;
use csv::ReaderBuilder;
use serde_json::Value;
use quick_xml::events::Event;
use quick_xml::Reader;

pub struct ExtractedDoc {
    pub text: String,
    pub page_count: usize,
    pub is_corrupted: bool,
    pub metadata: HashMap<String, String>,
}

pub fn parse_txt(content: &[u8]) -> ExtractedDoc {
    let text = String::from_utf8_lossy(content).into_owned();
    let char_count = text.chars().count();
    // Sensible estimation of 3000 characters per page for flat files
    let page_count = std::cmp::max(1, ((char_count as f64) / 3000.0).ceil() as usize);

    ExtractedDoc {
        text,
        page_count,
        is_corrupted: false,
        metadata: HashMap::new(),
    }
}

pub fn parse_md(content: &[u8]) -> ExtractedDoc {
    let mut doc = parse_txt(content);
    doc.metadata.insert("format".to_string(), "markdown".to_string());
    doc
}

pub fn parse_csv(content: &[u8]) -> ExtractedDoc {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content);
    let mut text = String::new();
    let mut is_corrupted = false;
    let mut row_count = 0;
    
    if let Ok(headers) = reader.headers() {
        for header in headers {
            text.push_str(header);
            text.push(' ');
        }
        text.push('\n');
    }

    for result in reader.records() {
        match result {
            Ok(record) => {
                row_count += 1;
                for field in &record {
                    text.push_str(field);
                    text.push(' ');
                }
                text.push('\n');
            }
            Err(_) => {
                is_corrupted = true;
                break;
            }
        }
    }

    let char_count = text.chars().count();
    let page_count = std::cmp::max(1, ((char_count as f64) / 3000.0).ceil() as usize);
    let mut metadata = HashMap::new();
    metadata.insert("rows".to_string(), row_count.to_string());
    metadata.insert("format".to_string(), "csv".to_string());

    ExtractedDoc {
        text,
        page_count,
        is_corrupted,
        metadata,
    }
}

pub fn parse_json(content: &[u8]) -> ExtractedDoc {
    let mut text = String::new();
    let mut is_corrupted = false;
    let mut metadata = HashMap::new();
    metadata.insert("format".to_string(), "json".to_string());

    match serde_json::from_slice::<Value>(content) {
        Ok(value) => {
            fn extract_strings(val: &Value, out: &mut String) {
                match val {
                    Value::String(s) => {
                        out.push_str(s);
                        out.push(' ');
                    }
                    Value::Array(arr) => {
                        for item in arr {
                            extract_strings(item, out);
                        }
                    }
                    Value::Object(obj) => {
                        for (k, v) in obj {
                            out.push_str(k);
                            out.push(' ');
                            extract_strings(v, out);
                        }
                    }
                    Value::Number(n) => {
                        out.push_str(&n.to_string());
                        out.push(' ');
                    }
                    Value::Bool(b) => {
                        out.push_str(&b.to_string());
                        out.push(' ');
                    }
                    Value::Null => {}
                }
            }
            extract_strings(&value, &mut text);
        }
        Err(_) => {
            is_corrupted = true;
        }
    }

    let char_count = text.chars().count();
    let page_count = std::cmp::max(1, ((char_count as f64) / 3000.0).ceil() as usize);

    ExtractedDoc {
        text,
        page_count,
        is_corrupted,
        metadata,
    }
}

pub fn parse_xml(content: &[u8]) -> ExtractedDoc {
    let mut reader = Reader::from_reader(content);
    reader.trim_text(true);
    let mut text = String::new();
    let mut is_corrupted = false;
    let mut buf = Vec::new();
    let mut metadata = HashMap::new();
    metadata.insert("format".to_string(), "xml".to_string());

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

    let char_count = text.chars().count();
    let page_count = std::cmp::max(1, ((char_count as f64) / 3000.0).ceil() as usize);

    ExtractedDoc {
        text,
        page_count,
        is_corrupted,
        metadata,
    }
}

pub fn parse_html(content: &[u8]) -> ExtractedDoc {
    let mut reader = Reader::from_reader(content);
    reader.trim_text(true);
    reader.check_end_names(false);
    let mut text = String::new();
    let mut is_corrupted = false;
    let mut buf = Vec::new();
    let mut in_script_or_style = false;
    let mut metadata = HashMap::new();
    metadata.insert("format".to_string(), "html".to_string());

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"script" || name.as_ref() == b"style" {
                    in_script_or_style = true;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"script" || name.as_ref() == b"style" {
                    in_script_or_style = false;
                }
            }
            Ok(Event::Text(e)) => {
                if !in_script_or_style {
                    if let Ok(s) = e.unescape() {
                        text.push_str(&s);
                        text.push(' ');
                    }
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

    let char_count = text.chars().count();
    let page_count = std::cmp::max(1, ((char_count as f64) / 3000.0).ceil() as usize);

    ExtractedDoc {
        text,
        page_count,
        is_corrupted,
        metadata,
    }
}
