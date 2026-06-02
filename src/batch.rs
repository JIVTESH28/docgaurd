use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Instant;
use rayon::prelude::*;
use serde_json::{json, Value};
use sha2::{Sha256, Digest};

use super::parsers::parse_document;
use super::security::analyze_security;
use super::quality::evaluate_quality;
use super::classifier::classify_domain;
use super::recommendations::generate_recommendations;

use std::sync::Arc;
use tiktoken_rs::CoreBPE;

pub struct AnalysisConfig {
    pub target_model: String,
    pub tokenizer_name: String,
    pub embedding_rate_per_million: f64,
    pub llm_input_rate_per_million: f64,
    pub max_file_size: usize,
    pub bpe: Option<Arc<CoreBPE>>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        let bpe = tiktoken_rs::cl100k_base().ok().map(Arc::new);
        AnalysisConfig {
            target_model: "gpt-4".to_string(),
            tokenizer_name: "cl100k_base".to_string(),
            embedding_rate_per_million: 0.02,
            llm_input_rate_per_million: 5.00,
            max_file_size: 50 * 1024 * 1024, // 50MB default limit
            bpe,
        }
    }
}

pub fn calculate_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn analyze_single_document(
    content: &[u8],
    file_name: &str,
    is_duplicate: bool,
    sha256_hash: &str,
    config: &AnalysisConfig,
) -> Value {
    let start_time = Instant::now();
    let file_size = content.len();
    let ext = file_name.split('.').last().unwrap_or("").to_lowercase();

    if file_size == 0 {
        return json!({
            "file_name": file_name,
            "file_type": ext,
            "error": "empty document",
            "token_count": 0,
            "word_count": 0,
            "character_count": 0,
            "page_count": 0,
            "requires_ocr": false,
            "quality_score": 0.0,
            "duplicate": is_duplicate,
            "sha256": sha256_hash,
            "security_risk": "high",
            "fits_context": true,
            "rag_ready": false,
            "requires_summarization": false,
            "recommended_chunking": "no chunking",
            "document_class": "Technical Documentation",
            "recommended_agent": "TechnicalDocAgent",
            "estimated_embedding_cost": 0.0,
            "estimated_llm_cost": 0.0,
            "processing_time_ms": start_time.elapsed().as_secs_f64() * 1000.0
        });
    }

    if file_size > config.max_file_size {
        return json!({
            "file_name": file_name,
            "file_type": ext,
            "error": "oversized document",
            "token_count": 0,
            "word_count": 0,
            "character_count": 0,
            "page_count": 0,
            "requires_ocr": false,
            "quality_score": 0.0,
            "duplicate": is_duplicate,
            "sha256": sha256_hash,
            "security_risk": "high",
            "fits_context": false,
            "rag_ready": false,
            "requires_summarization": false,
            "recommended_chunking": "no chunking",
            "document_class": "Technical Documentation",
            "recommended_agent": "TechnicalDocAgent",
            "estimated_embedding_cost": 0.0,
            "estimated_llm_cost": 0.0,
            "processing_time_ms": start_time.elapsed().as_secs_f64() * 1000.0
        });
    }

    let doc = parse_document(content, file_name);

    if doc.metadata.contains_key("unsupported_format") {
        return json!({
            "file_name": file_name,
            "file_type": ext,
            "error": "unsupported format",
            "token_count": 0,
            "word_count": 0,
            "character_count": 0,
            "page_count": 0,
            "requires_ocr": false,
            "quality_score": 0.0,
            "duplicate": is_duplicate,
            "sha256": sha256_hash,
            "security_risk": "high",
            "fits_context": true,
            "rag_ready": false,
            "requires_summarization": false,
            "recommended_chunking": "no chunking",
            "document_class": "Technical Documentation",
            "recommended_agent": "TechnicalDocAgent",
            "estimated_embedding_cost": 0.0,
            "estimated_llm_cost": 0.0,
            "processing_time_ms": start_time.elapsed().as_secs_f64() * 1000.0
        });
    }

    let char_count = doc.text.chars().count();
    let word_count = doc.text.split_whitespace().count();

    let security_res = analyze_security(content, doc.is_corrupted, doc.page_count, file_name);
    let security_risk_str = security_res.as_str();

    let (quality_score, requires_ocr) = evaluate_quality(
        &doc.text,
        doc.page_count,
        file_size,
        &doc.metadata,
        doc.is_corrupted || (doc.text.is_empty() && doc.page_count == 0)
    );

    let (domain, agent) = classify_domain(&doc.text);

    let recs = generate_recommendations(
        &doc.text,
        doc.page_count,
        &ext,
        domain,
        quality_score,
        requires_ocr,
        security_risk_str,
        &config.bpe,
        &config.tokenizer_name,
        &config.target_model,
        config.embedding_rate_per_million,
        config.llm_input_rate_per_million,
    );

    let processing_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

    json!({
        "file_name": file_name,
        "file_type": ext,
        "sha256": sha256_hash,
        "token_count": recs.token_count,
        "word_count": word_count,
        "character_count": char_count,
        "page_count": doc.page_count,
        "requires_ocr": requires_ocr,
        "quality_score": quality_score,
        "duplicate": is_duplicate,
        "security_risk": security_risk_str,
        "fits_context": recs.fits_context,
        "rag_ready": recs.rag_ready,
        "requires_summarization": recs.requires_summarization,
        "recommended_chunking": recs.recommended_chunking,
        "document_class": domain,
        "recommended_agent": agent,
        "estimated_embedding_cost": recs.estimated_embedding_cost,
        "estimated_llm_cost": recs.estimated_llm_cost,
        "processing_time_ms": (processing_time_ms * 100.0).round() / 100.0
    })
}

pub fn analyze_batch_files(
    file_paths: &[String],
    config: &AnalysisConfig,
) -> Value {
    let start_time = Instant::now();

    let file_data_results: Vec<(String, Result<Vec<u8>, String>)> = file_paths
        .par_iter()
        .map(|path_str| {
            let path = Path::new(path_str);
            let file_name = path.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| path_str.clone());
            
            match fs::read(path) {
                Ok(bytes) => (file_name, Ok(bytes)),
                Err(e) => (file_name, Err(e.to_string())),
            }
        })
        .collect();

    let mut seen_hashes = HashSet::new();
    let mut files_metadata = Vec::new();

    for (file_name, bytes_res) in file_data_results {
        match bytes_res {
            Ok(bytes) => {
                let hash = calculate_sha256(&bytes);
                let is_dup = seen_hashes.contains(&hash);
                if !is_dup {
                    seen_hashes.insert(hash.clone());
                }
                files_metadata.push((file_name, Ok(bytes), hash, is_dup));
            }
            Err(e) => {
                files_metadata.push((file_name, Err(e), String::new(), false));
            }
        }
    }

    let file_reports: Vec<Value> = files_metadata
        .into_par_iter()
        .map(|(file_name, bytes_res, hash, is_duplicate)| {
            match bytes_res {
                Ok(bytes) => {
                    analyze_single_document(&bytes, &file_name, is_duplicate, &hash, config)
                }
                Err(e) => {
                    let ext = file_name.split('.').last().unwrap_or("").to_lowercase();
                    json!({
                        "file_name": file_name,
                        "file_type": ext,
                        "error": format!("file read error: {}", e),
                        "token_count": 0,
                        "word_count": 0,
                        "character_count": 0,
                        "page_count": 0,
                        "requires_ocr": false,
                        "quality_score": 0.0,
                        "duplicate": false,
                        "sha256": "",
                        "security_risk": "high",
                        "fits_context": true,
                        "rag_ready": false,
                        "requires_summarization": false,
                        "recommended_chunking": "no chunking",
                        "document_class": "Technical Documentation",
                        "recommended_agent": "TechnicalDocAgent",
                        "estimated_embedding_cost": 0.0,
                        "estimated_llm_cost": 0.0,
                        "processing_time_ms": 0.0
                    })
                }
            }
        })
        .collect();

    let total_processing_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    
    let mut total_files = 0;
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut total_tokens = 0;
    let mut total_words = 0;
    let mut total_characters = 0;
    let mut total_pages = 0;
    let mut total_embedding_cost = 0.0;
    let mut total_llm_cost = 0.0;
    let mut duplicate_count = 0;
    let mut ocr_required_count = 0;
    
    let mut class_distribution = HashMap::new();
    let mut risk_distribution = HashMap::new();

    for report in &file_reports {
        total_files += 1;
        if report.get("error").is_some() {
            failure_count += 1;
        } else {
            success_count += 1;
            
            total_tokens += report.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            total_words += report.get("word_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            total_characters += report.get("character_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            total_pages += report.get("page_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            
            total_embedding_cost += report.get("estimated_embedding_cost").and_then(|v| v.as_f64()).unwrap_or(0.0);
            total_llm_cost += report.get("estimated_llm_cost").and_then(|v| v.as_f64()).unwrap_or(0.0);

            if report.get("duplicate").and_then(|v| v.as_bool()).unwrap_or(false) {
                duplicate_count += 1;
            }
            if report.get("requires_ocr").and_then(|v| v.as_bool()).unwrap_or(false) {
                ocr_required_count += 1;
            }

            if let Some(doc_class) = report.get("document_class").and_then(|v| v.as_str()) {
                *class_distribution.entry(doc_class.to_string()).or_insert(0) += 1;
            }
            if let Some(risk) = report.get("security_risk").and_then(|v| v.as_str()) {
                *risk_distribution.entry(risk.to_string()).or_insert(0) += 1;
            }
        }
    }

    json!({
        "summary": {
            "total_files": total_files,
            "successful_files": success_count,
            "failed_files": failure_count,
            "total_tokens": total_tokens,
            "total_words": total_words,
            "total_characters": total_characters,
            "total_pages": total_pages,
            "duplicate_files": duplicate_count,
            "ocr_required_files": ocr_required_count,
            "total_estimated_embedding_cost": (total_embedding_cost * 10000.0).round() / 10000.0,
            "total_estimated_llm_cost": (total_llm_cost * 10000.0).round() / 10000.0,
            "class_distribution": class_distribution,
            "security_risk_distribution": risk_distribution,
            "batch_processing_time_ms": (total_processing_time_ms * 100.0).round() / 100.0
        },
        "results": file_reports
    })
}
