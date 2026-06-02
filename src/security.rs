use std::io::Cursor;
use zip::ZipArchive;

pub enum SecurityRisk {
    Low,
    Medium,
    High,
}

impl SecurityRisk {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityRisk::Low => "low",
            SecurityRisk::Medium => "medium",
            SecurityRisk::High => "high",
        }
    }
}

pub fn analyze_security(content: &[u8], is_corrupted: bool, page_count: usize, _file_name: &str) -> SecurityRisk {
    if is_corrupted {
        return SecurityRisk::High;
    }

    let file_size = content.len();

    // 1. Excessive file size or page count (Resource exhaustion check)
    if file_size > 50 * 1024 * 1024 || page_count > 2000 {
        return SecurityRisk::High;
    }

    if file_size > 10 * 1024 * 1024 || page_count > 500 {
        return SecurityRisk::Medium;
    }

    // 2. Zip / Compression bomb check
    if content.len() > 4 && &content[0..4] == b"PK\x03\x04" {
        let cursor = Cursor::new(content);
        if let Ok(mut archive) = ZipArchive::new(cursor) {
            let mut decompressed_size: u64 = 0;
            for i in 0..archive.len() {
                if let Ok(file) = archive.by_index(i) {
                    decompressed_size += file.size();
                }
            }
            let compressed_size = file_size as u64;
            if compressed_size > 0 {
                let ratio = (decompressed_size as f64) / (compressed_size as f64);
                // If compression ratio > 100x and decompress size > 10MB, it is highly likely a zip bomb
                if ratio > 100.0 && decompressed_size > 10 * 1024 * 1024 {
                    return SecurityRisk::High;
                }
                if ratio > 50.0 && decompressed_size > 5 * 1024 * 1024 {
                    return SecurityRisk::Medium;
                }
            }
        }
    }

    SecurityRisk::Low
}
