use std::fs;
use std::path::Path;
use std::time::Instant;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::batch::{analyze_single_document, calculate_sha256, AnalysisConfig};
use super::kb::{convert_directory_to_kb_with_mode, convert_single_to_kb_with_mode};
use super::pii::{detect_pii, redact_pii};
use super::recommendations::get_token_count;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTask {
    pub id: Option<String>,
    pub task_type: String, // "scan", "to_kb", "redact_pii", "token_budget", "repo_digest"
    pub file_path: Option<String>,
    pub content: Option<String>,
    pub file_name: Option<String>,
    pub target_model: Option<String>,
    pub mode: Option<String>,
    pub entities: Option<Vec<String>>,
    pub recursive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTaskResult {
    pub id: Option<String>,
    pub task_type: String,
    pub success: bool,
    pub result: Value,
    pub error: Option<String>,
    pub elapsed_ms: f64,
}

pub fn execute_single_task(task: &ToolTask, config: &AnalysisConfig) -> ToolTaskResult {
    let start = Instant::now();
    let task_type = task.task_type.as_str();

    match task_type {
        "scan" => {
            let bytes = if let Some(ref path_str) = task.file_path {
                match fs::read(Path::new(path_str)) {
                    Ok(b) => b,
                    Err(e) => {
                        return ToolTaskResult {
                            id: task.id.clone(),
                            task_type: task.task_type.clone(),
                            success: false,
                            result: Value::Null,
                            error: Some(format!("Could not read file {}: {}", path_str, e)),
                            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                        };
                    }
                }
            } else if let Some(ref c) = task.content {
                c.as_bytes().to_vec()
            } else {
                return ToolTaskResult {
                    id: task.id.clone(),
                    task_type: task.task_type.clone(),
                    success: false,
                    result: Value::Null,
                    error: Some("Either 'file_path' or 'content' must be provided for scan".to_string()),
                    elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                };
            };

            let name = task.file_name.as_deref()
                .or_else(|| task.file_path.as_deref().and_then(|p| Path::new(p).file_name().and_then(|f| f.to_str())))
                .unwrap_or("document.txt");

            let hash = calculate_sha256(&bytes);
            let val = analyze_single_document(&bytes, name, false, &hash, config);

            ToolTaskResult {
                id: task.id.clone(),
                task_type: task.task_type.clone(),
                success: true,
                result: val,
                error: None,
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        }
        "to_kb" => {
            let target_model = task.target_model.as_deref().unwrap_or("claude-3-5-sonnet");
            let mode = task.mode.as_deref().unwrap_or("full");

            let (bytes, name) = if let Some(ref path_str) = task.file_path {
                match fs::read(Path::new(path_str)) {
                    Ok(b) => {
                        let fname = Path::new(path_str).file_name().and_then(|f| f.to_str()).unwrap_or("document.txt");
                        (b, fname.to_string())
                    }
                    Err(e) => {
                        return ToolTaskResult {
                            id: task.id.clone(),
                            task_type: task.task_type.clone(),
                            success: false,
                            result: Value::Null,
                            error: Some(format!("Could not read file {}: {}", path_str, e)),
                            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                        };
                    }
                }
            } else if let Some(ref c) = task.content {
                let fname = task.file_name.as_deref().unwrap_or("document.txt").to_string();
                (c.as_bytes().to_vec(), fname)
            } else {
                return ToolTaskResult {
                    id: task.id.clone(),
                    task_type: task.task_type.clone(),
                    success: false,
                    result: Value::Null,
                    error: Some("Either 'file_path' or 'content' must be provided for to_kb".to_string()),
                    elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                };
            };

            let val = convert_single_to_kb_with_mode(&bytes, &name, target_model, mode, config);
            ToolTaskResult {
                id: task.id.clone(),
                task_type: task.task_type.clone(),
                success: true,
                result: val,
                error: None,
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        }
        "redact_pii" => {
            let text = if let Some(ref path_str) = task.file_path {
                match fs::read_to_string(Path::new(path_str)) {
                    Ok(s) => s,
                    Err(e) => {
                        return ToolTaskResult {
                            id: task.id.clone(),
                            task_type: task.task_type.clone(),
                            success: false,
                            result: Value::Null,
                            error: Some(format!("Could not read text from {}: {}", path_str, e)),
                            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                        };
                    }
                }
            } else if let Some(ref c) = task.content {
                c.clone()
            } else {
                return ToolTaskResult {
                    id: task.id.clone(),
                    task_type: task.task_type.clone(),
                    success: false,
                    result: Value::Null,
                    error: Some("Either 'file_path' or 'content' must be provided for redact_pii".to_string()),
                    elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                };
            };

            let entities = task.entities.as_deref().unwrap_or(&[]);
            let found = detect_pii(&text);
            let redacted = redact_pii(&text, entities);

            ToolTaskResult {
                id: task.id.clone(),
                task_type: task.task_type.clone(),
                success: true,
                result: json!({
                    "redacted_text": redacted,
                    "pii_found": found,
                    "contains_pii": !found.is_empty(),
                    "length": text.len()
                }),
                error: None,
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        }
        "token_budget" => {
            let text = if let Some(ref path_str) = task.file_path {
                match fs::read_to_string(Path::new(path_str)) {
                    Ok(s) => s,
                    Err(e) => {
                        return ToolTaskResult {
                            id: task.id.clone(),
                            task_type: task.task_type.clone(),
                            success: false,
                            result: Value::Null,
                            error: Some(format!("Could not read text from {}: {}", path_str, e)),
                            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                        };
                    }
                }
            } else if let Some(ref c) = task.content {
                c.clone()
            } else {
                return ToolTaskResult {
                    id: task.id.clone(),
                    task_type: task.task_type.clone(),
                    success: false,
                    result: Value::Null,
                    error: Some("Either 'file_path' or 'content' must be provided for token_budget".to_string()),
                    elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                };
            };

            let target_model = task.target_model.as_deref().unwrap_or("claude-3-5-sonnet");
            let tokens = get_token_count(&text, &config.bpe, &config.tokenizer_name);
            let rate = super::kb::get_model_rate(target_model);
            let cost = ((tokens as f64) / 1_000_000.0) * rate;

            ToolTaskResult {
                id: task.id.clone(),
                task_type: task.task_type.clone(),
                success: true,
                result: json!({
                    "target_model": target_model,
                    "token_count": tokens,
                    "character_count": text.chars().count(),
                    "rate_per_million_usd": rate,
                    "estimated_cost_usd": (cost * 100000.0).round() / 100000.0
                }),
                error: None,
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        }
        "repo_digest" => {
            let dir = task.file_path.as_deref().unwrap_or(".");
            let recursive = task.recursive.unwrap_or(true);
            let target_model = task.target_model.as_deref().unwrap_or("claude-3-5-sonnet");
            let mode = task.mode.as_deref().unwrap_or("full");

            let val = convert_directory_to_kb_with_mode(dir, recursive, target_model, mode, config);
            ToolTaskResult {
                id: task.id.clone(),
                task_type: task.task_type.clone(),
                success: true,
                result: val,
                error: None,
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        }
        other => ToolTaskResult {
            id: task.id.clone(),
            task_type: task.task_type.clone(),
            success: false,
            result: Value::Null,
            error: Some(format!("Unknown task type '{}'", other)),
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        },
    }
}

pub fn execute_parallel_tasks(
    tasks: Vec<ToolTask>,
    config: &AnalysisConfig,
) -> Vec<ToolTaskResult> {
    tasks.into_par_iter()
        .map(|task| execute_single_task(&task, config))
        .collect()
}

pub fn parallel_scan_documents(
    file_paths: &[String],
    config: &AnalysisConfig,
) -> Vec<Value> {
    file_paths.par_iter()
        .map(|path_str| {
            let path = Path::new(path_str);
            let name = path.file_name().and_then(|f| f.to_str()).unwrap_or("document.txt");
            match fs::read(path) {
                Ok(bytes) => {
                    let hash = calculate_sha256(&bytes);
                    analyze_single_document(&bytes, name, false, &hash, config)
                }
                Err(e) => json!({
                    "file_name": name,
                    "file_path": path_str,
                    "error": format!("Could not read file: {}", e),
                    "token_count": 0,
                    "security_risk": "high"
                }),
            }
        })
        .collect()
}

pub fn parallel_convert_to_kb(
    file_paths: &[String],
    target_model: &str,
    mode: &str,
    config: &AnalysisConfig,
) -> Vec<Value> {
    file_paths.par_iter()
        .map(|path_str| {
            let path = Path::new(path_str);
            let name = path.file_name().and_then(|f| f.to_str()).unwrap_or("document.txt");
            match fs::read(path) {
                Ok(bytes) => convert_single_to_kb_with_mode(&bytes, name, target_model, mode, config),
                Err(e) => json!({
                    "file_name": name,
                    "file_path": path_str,
                    "error": format!("Could not read file: {}", e),
                    "markdown": format!("# Error\n\nCould not read file `{}`: {}", path_str, e)
                }),
            }
        })
        .collect()
}

pub fn parallel_redact_texts(
    texts: &[String],
    entities: &[String],
) -> Vec<String> {
    texts.par_iter()
        .map(|t| redact_pii(t, entities))
        .collect()
}
