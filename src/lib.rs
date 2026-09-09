use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::Path;

pub mod parsers;
pub mod security;
pub mod quality;
pub mod classifier;
pub mod recommendations;
pub mod batch;
pub mod pii;
pub mod kb;
pub mod parallel;
pub mod mcp;

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
                if let Some(max_sz) = cfg.get("max_file_size") {
                    if let Ok(sz) = max_sz.extract::<usize>(py) {
                        analysis_config.max_file_size = sz;
                    }
                }
            });
        }

        DocumentAnalyzer {
            config: analysis_config,
        }
    }

    #[pyo3(signature = (file_path, file_name = None, is_duplicate = false))]
    fn analyze_file(
        &self,
        file_path: String,
        file_name: Option<String>,
        is_duplicate: bool,
    ) -> PyResult<String> {
        let path = Path::new(&file_path);
        let name = file_name.unwrap_or_else(|| {
            path.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| file_path.clone())
        });

        let bytes = std::fs::read(path).map_err(|e| {
            pyo3::exceptions::PyFileNotFoundError::new_err(format!("Could not read file: {}", e))
        })?;

        let sha256_hash = calculate_sha256(&bytes);
        let result_val = analyze_single_document(&bytes, &name, is_duplicate, &sha256_hash, &self.config);
        
        serde_json::to_string(&result_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[pyo3(signature = (content, file_name, is_duplicate = false))]
    fn analyze_bytes(
        &self,
        content: Vec<u8>,
        file_name: String,
        is_duplicate: bool,
    ) -> PyResult<String> {
        let sha256_hash = calculate_sha256(&content);
        let result_val = analyze_single_document(&content, &file_name, is_duplicate, &sha256_hash, &self.config);
        
        serde_json::to_string(&result_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[pyo3(signature = (file_paths))]
    fn analyze_batch(&self, file_paths: Vec<String>) -> PyResult<String> {
        let result_val = analyze_batch_files(&file_paths, &self.config);
        
        serde_json::to_string(&result_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[pyo3(signature = (dir_path, recursive = None))]
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

    #[pyo3(signature = (file_path, target_model = None, mode = None))]
    fn convert_file_to_kb(&self, file_path: String, target_model: Option<String>, mode: Option<String>) -> PyResult<String> {
        let path = Path::new(&file_path);
        let file_name = path.file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_path.clone());

        let bytes = std::fs::read(path).map_err(|e| {
            pyo3::exceptions::PyFileNotFoundError::new_err(format!("Could not read file: {}", e))
        })?;

        let model = target_model.unwrap_or_else(|| self.config.target_model.clone());
        let kb_mode = mode.unwrap_or_else(|| "full".to_string());
        let res_val = kb::convert_single_to_kb_with_mode(&bytes, &file_name, &model, &kb_mode, &self.config);
        
        serde_json::to_string(&res_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[pyo3(signature = (content, file_name, target_model = None, mode = None))]
    fn convert_bytes_to_kb(&self, content: Vec<u8>, file_name: String, target_model: Option<String>, mode: Option<String>) -> PyResult<String> {
        let model = target_model.unwrap_or_else(|| self.config.target_model.clone());
        let kb_mode = mode.unwrap_or_else(|| "full".to_string());
        let res_val = kb::convert_single_to_kb_with_mode(&content, &file_name, &model, &kb_mode, &self.config);
        
        serde_json::to_string(&res_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[pyo3(signature = (dir_path, recursive = None, target_model = None, mode = None))]
    fn convert_directory_to_kb(&self, dir_path: String, recursive: Option<bool>, target_model: Option<String>, mode: Option<String>) -> PyResult<String> {
        let is_recursive = recursive.unwrap_or(true);
        let model = target_model.unwrap_or_else(|| self.config.target_model.clone());
        let kb_mode = mode.unwrap_or_else(|| "full".to_string());
        let res_val = kb::convert_directory_to_kb_with_mode(&dir_path, is_recursive, &model, &kb_mode, &self.config);
        
        serde_json::to_string(&res_val)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    // Rayon Parallel Tool & Agent Execution Layer
    #[pyo3(signature = (tasks_json))]
    fn execute_parallel_tasks(&self, py: Python<'_>, tasks_json: String) -> PyResult<String> {
        let tasks: Vec<parallel::ToolTask> = serde_json::from_str(&tasks_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid tasks JSON: {}", e)))?;
        
        let results = py.allow_threads(|| {
            parallel::execute_parallel_tasks(tasks, &self.config)
        });

        serde_json::to_string(&results)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[pyo3(signature = (file_paths))]
    fn parallel_scan(&self, py: Python<'_>, file_paths: Vec<String>) -> PyResult<String> {
        let results = py.allow_threads(|| {
            parallel::parallel_scan_documents(&file_paths, &self.config)
        });

        serde_json::to_string(&results)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[pyo3(signature = (file_paths, target_model = None, mode = None))]
    fn parallel_convert_to_kb(&self, py: Python<'_>, file_paths: Vec<String>, target_model: Option<String>, mode: Option<String>) -> PyResult<String> {
        let model = target_model.unwrap_or_else(|| self.config.target_model.clone());
        let kb_mode = mode.unwrap_or_else(|| "full".to_string());
        
        let results = py.allow_threads(|| {
            parallel::parallel_convert_to_kb(&file_paths, &model, &kb_mode, &self.config)
        });

        serde_json::to_string(&results)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[pyo3(signature = (texts, entities = None))]
    fn parallel_redact_pii(&self, py: Python<'_>, texts: Vec<String>, entities: Option<Vec<String>>) -> PyResult<Vec<String>> {
        let ent = entities.unwrap_or_default();
        let results = py.allow_threads(|| {
            parallel::parallel_redact_texts(&texts, &ent)
        });
        Ok(results)
    }

    // MCP Server Handlers
    fn run_mcp_server(&self) -> PyResult<()> {
        mcp::run_stdio_server(&self.config)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    #[pyo3(signature = (message_json))]
    fn handle_mcp_message(&self, message_json: String) -> PyResult<Option<String>> {
        let msg: serde_json::Value = serde_json::from_str(&message_json)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {}", e)))?;
        
        let resp = mcp::handle_json_rpc_message(&msg, &self.config);
        match resp {
            Some(v) => {
                let s = serde_json::to_string(&v)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(Some(s))
            }
            None => Ok(None)
        }
    }

    #[pyo3(signature = (text, entities = None))]
    fn redact_pii(&self, py: Python<'_>, text: String, entities: Option<Vec<String>>) -> PyResult<String> {
        py.allow_threads(|| {
            Ok(pii::redact_pii(&text, &entities.unwrap_or_default()))
        })
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
fn docarmor(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    fn test_pii_detection_and_redaction() {
        let text = "My email is test@example.com and phone is 123-456-7890.";
        let found = pii::detect_pii(text);
        assert!(found.contains(&"email".to_string()));
        assert!(found.contains(&"phone".to_string()));
        assert!(!found.contains(&"ssn".to_string()));

        let redacted_all = pii::redact_pii(text, &[]);
        assert_eq!(redacted_all, "My email is [EMAIL] and phone is [PHONE].");

        let redacted_only_email = pii::redact_pii(text, &["email".to_string()]);
        assert_eq!(redacted_only_email, "My email is [EMAIL] and phone is 123-456-7890.");
    }

    #[test]
    fn test_kb_conversion() {
        let content = b"This is a legal agreement contract liability clause for governing law. Section 1: Indemnity rules and warranty.\nSection 2: Intellectual property rights and terms.";
        let config = AnalysisConfig::default();
        let res = kb::convert_single_to_kb(content, "contract.txt", "claude-3-5-sonnet", &config);
        
        let markdown = res.get("markdown").unwrap().as_str().unwrap();
        assert!(markdown.contains("# Knowledge Base: contract.txt"));
        assert!(markdown.contains("Table of Contents"));
        assert!(markdown.contains("Executive Summary"));

        let telemetry = res.get("telemetry").unwrap();
        assert_eq!(telemetry.get("target_model").unwrap().as_str().unwrap(), "claude-3-5-sonnet");
        assert!(telemetry.get("raw_tokens").unwrap().as_u64().unwrap() > 0);
        assert!(telemetry.get("kb_tokens").unwrap().as_u64().unwrap() > 0);
    }
}
