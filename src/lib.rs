use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::Path;

mod parsers;
mod security;
mod quality;
mod classifier;
mod recommendations;
mod batch;

use batch::{AnalysisConfig, analyze_single_document, analyze_batch_files, calculate_sha256};

#[pyclass]
pub struct DocumentAnalyzer {
    config: AnalysisConfig,
}

#[pymethods]
impl DocumentAnalyzer {
    #[new]
    #[pyo3(signature = (config = None))]
    fn new(config: Option<HashMap<String, PyObject>>) -> Self {
        let mut analysis_config = AnalysisConfig::default();
        
        if let Some(cfg) = config {
            Python::with_gil(|py| {
                if let Some(model) = cfg.get("target_model") {
                    if let Ok(s) = model.extract::<String>(py) {
                        analysis_config.target_model = s;
                    }
                }
                if let Some(tok) = cfg.get("tokenizer_name") {
                    if let Ok(s) = tok.extract::<String>(py) {
                        analysis_config.tokenizer_name = s;
                    }
                }
                if let Some(emb) = cfg.get("embedding_rate_per_million") {
                    if let Ok(f) = emb.extract::<f64>(py) {
                        analysis_config.embedding_rate_per_million = f;
                    }
                }
                if let Some(llm) = cfg.get("llm_input_rate_per_million") {
                    if let Ok(f) = llm.extract::<f64>(py) {
                        analysis_config.llm_input_rate_per_million = f;
                    }
                }
                if let Some(size) = cfg.get("max_file_size") {
                    if let Ok(sz) = size.extract::<usize>(py) {
                        analysis_config.max_file_size = sz;
                    }
                }
            });

            use std::sync::Arc;
            let bpe_res = match &analysis_config.tokenizer_name as &str {
                "r50k_base" => tiktoken_rs::r50k_base(),
                "p50k_base" => tiktoken_rs::p50k_base(),
                _ => tiktoken_rs::cl100k_base(),
            };
            if let Ok(bpe) = bpe_res {
                analysis_config.bpe = Some(Arc::new(bpe));
            }
        }

        DocumentAnalyzer {
            config: analysis_config,
        }
    }

    fn analyze_file(&self, file_path: String) -> PyResult<String> {
        let path = Path::new(&file_path);
        let file_name = path.file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_path.clone());

        let bytes = std::fs::read(path).map_err(|e| {
            pyo3::exceptions::PyFileNotFoundError::new_err(format!("Could not read file: {}", e))
        })?;

        let hash = calculate_sha256(&bytes);
        let result_val = analyze_single_document(&bytes, &file_name, false, &hash, &self.config);
        
        serde_json::to_string(&result_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn analyze_bytes(&self, content: Vec<u8>, file_name: String) -> PyResult<String> {
        let hash = calculate_sha256(&content);
        let result_val = analyze_single_document(&content, &file_name, false, &hash, &self.config);
        
        serde_json::to_string(&result_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn analyze_batch(&self, file_paths: Vec<String>) -> PyResult<String> {
        let result_val = analyze_batch_files(&file_paths, &self.config);
        
        serde_json::to_string(&result_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn analyze_directory(&self, dir_path: String, recursive: Option<bool>) -> PyResult<String> {
        let path = Path::new(&dir_path);
        if !path.is_dir() {
            return Err(pyo3::exceptions::PyNotADirectoryError::new_err("Provided path is not a directory"));
        }

        let is_recursive = recursive.unwrap_or(true);
        let mut file_paths = Vec::new();
        get_directory_files(path, is_recursive, &mut file_paths);

        let result_val = analyze_batch_files(&file_paths, &self.config);
        
        serde_json::to_string(&result_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn count_words(&self, file_path: String) -> PyResult<usize> {
        let path = Path::new(&file_path);
        let file_name = path.file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_path.clone());

        let bytes = std::fs::read(path).map_err(|e| {
            pyo3::exceptions::PyFileNotFoundError::new_err(format!("Could not read file: {}", e))
        })?;

        let doc = parsers::parse_document(&bytes, &file_name);
        if doc.metadata.contains_key("unsupported_format") {
            return Ok(0);
        }
        Ok(doc.text.split_whitespace().count())
    }

    fn count_tokens(&self, file_path: String) -> PyResult<usize> {
        let path = Path::new(&file_path);
        let file_name = path.file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_path.clone());

        let bytes = std::fs::read(path).map_err(|e| {
            pyo3::exceptions::PyFileNotFoundError::new_err(format!("Could not read file: {}", e))
        })?;

        let doc = parsers::parse_document(&bytes, &file_name);
        if doc.metadata.contains_key("unsupported_format") {
            return Ok(0);
        }
        Ok(recommendations::get_token_count(&doc.text, &self.config.bpe, &self.config.tokenizer_name))
    }

    fn count_chars(&self, file_path: String) -> PyResult<usize> {
        let path = Path::new(&file_path);
        let file_name = path.file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_path.clone());

        let bytes = std::fs::read(path).map_err(|e| {
            pyo3::exceptions::PyFileNotFoundError::new_err(format!("Could not read file: {}", e))
        })?;

        let doc = parsers::parse_document(&bytes, &file_name);
        if doc.metadata.contains_key("unsupported_format") {
            return Ok(0);
        }
        Ok(doc.text.chars().count())
    }

    fn count_words_bytes(&self, content: Vec<u8>, file_name: String) -> PyResult<usize> {
        let doc = parsers::parse_document(&content, &file_name);
        if doc.metadata.contains_key("unsupported_format") {
            return Ok(0);
        }
        Ok(doc.text.split_whitespace().count())
    }

    fn count_tokens_bytes(&self, content: Vec<u8>, file_name: String) -> PyResult<usize> {
        let doc = parsers::parse_document(&content, &file_name);
        if doc.metadata.contains_key("unsupported_format") {
            return Ok(0);
        }
        Ok(recommendations::get_token_count(&doc.text, &self.config.bpe, &self.config.tokenizer_name))
    }

    fn count_chars_bytes(&self, content: Vec<u8>, file_name: String) -> PyResult<usize> {
        let doc = parsers::parse_document(&content, &file_name);
        if doc.metadata.contains_key("unsupported_format") {
            return Ok(0);
        }
        Ok(doc.text.chars().count())
    }
}

fn get_directory_files(dir: &Path, recursive: bool, files: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(path_str) = path.to_str() {
                    files.push(path_str.to_string());
                }
            } else if path.is_dir() && recursive {
                get_directory_files(&path, recursive, files);
            }
        }
    }
}

#[pymodule]
fn docgaurd(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DocumentAnalyzer>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let bytes = b"hello world";
        let hash = calculate_sha256(bytes);
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_txt_parser() {
        let content = b"This is a sample document for testing the docgaurd pipeline.";
        let doc = parsers::text::parse_txt(content);
        assert_eq!(doc.page_count, 1);
        assert!(doc.text.contains("sample document"));
        assert!(!doc.is_corrupted);
    }

    #[test]
    fn test_csv_parser() {
        let csv_data = b"name,role,salary\nAlice,Manager,100000\nBob,Engineer,80000";
        let doc = parsers::text::parse_csv(csv_data);
        assert_eq!(doc.page_count, 1);
        assert!(doc.text.contains("Alice"));
        assert!(doc.text.contains("Engineer"));
        assert_eq!(doc.metadata.get("rows").unwrap(), "2");
    }

    #[test]
    fn test_json_parser() {
        let json_data = b"{\"title\": \"Supply Agreement\", \"parties\": [\"A\", \"B\"], \"pages\": 5}";
        let doc = parsers::text::parse_json(json_data);
        assert!(doc.text.contains("Supply Agreement"));
        assert!(doc.text.contains("parties"));
    }

    #[test]
    fn test_security_analysis() {
        let content = vec![0u8; 100];
        let risk = security::analyze_security(&content, false, 5, "test.txt");
        assert_eq!(risk.as_str(), "low");

        let risk_pages = security::analyze_security(&content, false, 2500, "test.pdf");
        assert_eq!(risk_pages.as_str(), "high");

        let risk_corrupted = security::analyze_security(&content, true, 5, "test.pdf");
        assert_eq!(risk_corrupted.as_str(), "high");
    }

    #[test]
    fn test_quality_and_ocr() {
        let text = "This is a digital text with a lot of words so it has high density and readability.";
        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "pdf".to_string());
        
        let (score, requires_ocr) = quality::evaluate_quality(text, 1, 1000, &metadata, false);
        assert!(score > 0.4);
        assert!(!requires_ocr);

        let empty_text = "";
        let (score_empty, requires_ocr_empty) = quality::evaluate_quality(empty_text, 1, 1000, &metadata, false);
        assert!(score_empty < 0.2);
        assert!(requires_ocr_empty);
    }

    #[test]
    fn test_classification() {
        let text = "We have safety stock replenishment for inventory forecast warehouse logistics shipment and sku demand planning.";
        let (class, agent) = classifier::classify_domain(text);
        assert_eq!(class, "Supply Planning");
        assert_eq!(agent, "SupplyPlanningAgent");

        let legal_text = "This contract agreement liability clause NDA governed by law and indemnity of parties.";
        let (legal_class, legal_agent) = classifier::classify_domain(legal_text);
        assert_eq!(legal_class, "Legal");
        assert_eq!(legal_agent, "LegalAgent");
    }

    #[test]
    fn test_recommendations() {
        let text = "sample text ".repeat(100);
        let token_count = recommendations::get_token_count(&text, &None, "cl100k_base");
        assert!(token_count > 50);

        let fits = recommendations::validate_context_window(token_count, "gpt-3.5-turbo");
        assert!(fits);

        let chunking = recommendations::recommend_chunking(token_count, "txt", "Research");
        assert_eq!(chunking, "semantic chunking");
    }

    #[test]
    fn test_pipeline() {
        let content = b"This is a legal document agreement contract liability clause for procurement vendor.";
        let hash = calculate_sha256(content);
        let config = AnalysisConfig::default();
        
        let report = analyze_single_document(content, "agreement.txt", false, &hash, &config);
        assert_eq!(report.get("file_name").unwrap().as_str().unwrap(), "agreement.txt");
        assert_eq!(report.get("document_class").unwrap().as_str().unwrap(), "Legal");
        assert_eq!(report.get("security_risk").unwrap().as_str().unwrap(), "low");
        assert_eq!(report.get("rag_ready").unwrap().as_bool().unwrap(), true);
    }

    #[test]
    fn test_single_metrics() {
        let content = b"This is a legal document agreement contract liability clause for procurement vendor.";
        let analyzer = DocumentAnalyzer {
            config: AnalysisConfig::default(),
        };
        
        let words = analyzer.count_words_bytes(content.to_vec(), "agreement.txt".to_string()).unwrap();
        assert_eq!(words, 12);

        let tokens = analyzer.count_tokens_bytes(content.to_vec(), "agreement.txt".to_string()).unwrap();
        assert!(tokens > 10);

        let chars = analyzer.count_chars_bytes(content.to_vec(), "agreement.txt".to_string()).unwrap();
        assert_eq!(chars, content.len());
    }
}
