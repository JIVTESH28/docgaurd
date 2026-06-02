use std::collections::HashMap;

pub fn evaluate_quality(
    text: &str,
    page_count: usize,
    file_size: usize,
    metadata: &HashMap<String, String>,
    is_corrupted: bool,
) -> (f64, bool) {
    if is_corrupted {
        return (0.0, false);
    }

    let char_count = text.chars().count();
    let word_count = text.split_whitespace().count();

    // 1. OCR necessity check:
    // If a document has page counts but the character count per page is extremely low (< 50 chars), it is image-only or requires OCR.
    let requires_ocr = if page_count > 0 {
        (char_count as f64) / (page_count as f64) < 50.0
    } else {
        word_count == 0
    };

    // 2. Text density scoring (normalized around standard page content ratios)
    let avg_words_per_page = if page_count > 0 {
        (word_count as f64) / (page_count as f64)
    } else {
        word_count as f64
    };

    let density_score = if avg_words_per_page == 0.0 {
        0.0
    } else if avg_words_per_page < 50.0 {
        0.2
    } else if avg_words_per_page < 150.0 {
        0.6
    } else if avg_words_per_page <= 800.0 {
        1.0
    } else {
        0.7 // Overly high density (possibly raw log dumps or compressed values)
    };

    // 3. Extractable content ratio
    let text_bytes_len = text.len();
    let content_ratio = if file_size > 0 {
        (text_bytes_len as f64) / (file_size as f64)
    } else {
        0.0
    };
    let ratio_score = content_ratio.min(1.0);

    // 4. Metadata richness evaluation
    let mut metadata_hits = 0;
    for (k, v) in metadata {
        if k != "format" && k != "total_pages" && k != "failed_pages" && !v.is_empty() {
            metadata_hits += 1;
        }
    }
    let metadata_score = (metadata_hits as f64 / 4.0).min(1.0);

    // 5. Final combined score
    let base_score = 0.5 * density_score + 0.3 * ratio_score + 0.2 * metadata_score;
    let mut quality_score = base_score;

    if requires_ocr {
        quality_score *= 0.3; // Penalty for scanned/empty text
    }

    let rounded_score = (quality_score * 100.0).round() / 100.0;

    (rounded_score.clamp(0.0, 1.0), requires_ocr)
}
