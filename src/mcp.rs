use std::io::{self, BufRead, Write};
use serde_json::{json, Value};

use super::batch::AnalysisConfig;
use super::parallel::{execute_single_task, ToolTask};

pub fn get_tools_list() -> Value {
    json!([
        {
            "name": "docarmor_scan",
            "description": "Scan a document or text for security vulnerabilities (e.g. zip bombs, memory limits), quality metrics, PII, and token counts.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file on disk to inspect"
                    },
                    "content": {
                        "type": "string",
                        "description": "Raw text content to inspect directly"
                    },
                    "file_name": {
                        "type": "string",
                        "description": "Optional file name when content is provided"
                    }
                }
            }
        },
        {
            "name": "docarmor_to_kb",
            "description": "Convert a document or text into structured Knowledge Base markdown with hierarchical Table of Contents and deep anchor links.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to document file to convert into Knowledge Base markdown"
                    },
                    "content": {
                        "type": "string",
                        "description": "Raw document content string to convert"
                    },
                    "file_name": {
                        "type": "string",
                        "description": "Optional file name"
                    },
                    "target_model": {
                        "type": "string",
                        "description": "Target LLM model profile for token budgeting (default: 'claude-3-5-sonnet')"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["full", "compact", "outline"],
                        "description": "Knowledge Base detail mode (default: 'full')"
                    }
                }
            }
        },
        {
            "name": "docarmor_redact_pii",
            "description": "Detect and mask sensitive Personally Identifiable Information (PII) such as emails, phone numbers, SSNs, and credit cards using native Rust pattern matchers.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to redact PII from"
                    },
                    "entities": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional list of entities to redact (e.g. ['email', 'phone', 'ssn'])"
                    }
                },
                "required": ["text"]
            }
        },
        {
            "name": "docarmor_token_budget",
            "description": "Calculate exact GPT/Claude token count using high-speed tiktoken CoreBPE and compute multi-model cost estimates.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Raw text to calculate token budget for"
                    },
                    "path": {
                        "type": "string",
                        "description": "File path to read and compute token budget for"
                    },
                    "target_model": {
                        "type": "string",
                        "description": "Target model (e.g. 'claude-3-5-sonnet', 'gpt-4o', 'gemini-2.0-flash', 'deepseek-r1')"
                    }
                }
            }
        },
        {
            "name": "docarmor_repo_digest",
            "description": "Aggregates an entire multi-file codebase or directory tree into a single unified Knowledge Base brain with directory index.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir_path": {
                        "type": "string",
                        "description": "Directory path of repository or project"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Whether to traverse subdirectories recursively (default: true)"
                    },
                    "target_model": {
                        "type": "string",
                        "description": "Target LLM model profile"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["full", "compact", "outline"],
                        "description": "Knowledge base mode (default: 'full')"
                    }
                },
                "required": ["dir_path"]
            }
        },
        {
            "name": "docarmor_parallel_tools",
            "description": "Execute multiple heterogeneous tool calls concurrently across CPU cores using Rust's Rayon thread pool.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tasks": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "task_type": { "type": "string", "enum": ["scan", "to_kb", "redact_pii", "token_budget", "repo_digest"] },
                                "file_path": { "type": "string" },
                                "content": { "type": "string" },
                                "target_model": { "type": "string" },
                                "mode": { "type": "string" }
                            },
                            "required": ["task_type"]
                        },
                        "description": "Array of tool tasks to execute simultaneously"
                    }
                },
                "required": ["tasks"]
            }
        }
    ])
}

pub fn handle_tool_call(name: &str, arguments: &Value, config: &AnalysisConfig) -> (Value, bool) {
    match name {
        "docarmor_scan" => {
            let path = arguments["path"].as_str().map(|s| s.to_string());
            let content = arguments["content"].as_str().map(|s| s.to_string());
            let file_name = arguments["file_name"].as_str().map(|s| s.to_string());

            let task = ToolTask {
                id: None,
                task_type: "scan".to_string(),
                file_path: path,
                content,
                file_name,
                target_model: None,
                mode: None,
                entities: None,
                recursive: None,
            };
            let res = execute_single_task(&task, config);
            if res.success {
                (res.result, false)
            } else {
                (json!({ "error": res.error }), true)
            }
        }
        "docarmor_to_kb" => {
            let path = arguments["path"].as_str().map(|s| s.to_string());
            let content = arguments["content"].as_str().map(|s| s.to_string());
            let file_name = arguments["file_name"].as_str().map(|s| s.to_string());
            let target_model = arguments["target_model"].as_str().map(|s| s.to_string());
            let mode = arguments["mode"].as_str().map(|s| s.to_string());

            let task = ToolTask {
                id: None,
                task_type: "to_kb".to_string(),
                file_path: path,
                content,
                file_name,
                target_model,
                mode,
                entities: None,
                recursive: None,
            };
            let res = execute_single_task(&task, config);
            if res.success {
                (res.result, false)
            } else {
                (json!({ "error": res.error }), true)
            }
        }
        "docarmor_redact_pii" => {
            let text = arguments["text"].as_str().map(|s| s.to_string());
            let entities = arguments["entities"].as_array().map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>()
            });

            let task = ToolTask {
                id: None,
                task_type: "redact_pii".to_string(),
                file_path: None,
                content: text,
                file_name: None,
                target_model: None,
                mode: None,
                entities,
                recursive: None,
            };
            let res = execute_single_task(&task, config);
            if res.success {
                (res.result, false)
            } else {
                (json!({ "error": res.error }), true)
            }
        }
        "docarmor_token_budget" => {
            let text = arguments["text"].as_str().map(|s| s.to_string());
            let path = arguments["path"].as_str().map(|s| s.to_string());
            let target_model = arguments["target_model"].as_str().map(|s| s.to_string());

            let task = ToolTask {
                id: None,
                task_type: "token_budget".to_string(),
                file_path: path,
                content: text,
                file_name: None,
                target_model,
                mode: None,
                entities: None,
                recursive: None,
            };
            let res = execute_single_task(&task, config);
            if res.success {
                (res.result, false)
            } else {
                (json!({ "error": res.error }), true)
            }
        }
        "docarmor_repo_digest" => {
            let dir_path = arguments["dir_path"].as_str().unwrap_or(".").to_string();
            let recursive = arguments["recursive"].as_bool();
            let target_model = arguments["target_model"].as_str().map(|s| s.to_string());
            let mode = arguments["mode"].as_str().map(|s| s.to_string());

            let task = ToolTask {
                id: None,
                task_type: "repo_digest".to_string(),
                file_path: Some(dir_path),
                content: None,
                file_name: None,
                target_model,
                mode,
                entities: None,
                recursive,
            };
            let res = execute_single_task(&task, config);
            if res.success {
                (res.result, false)
            } else {
                (json!({ "error": res.error }), true)
            }
        }
        "docarmor_parallel_tools" => {
            if let Some(tasks_arr) = arguments["tasks"].as_array() {
                let parsed_tasks: Vec<ToolTask> = tasks_arr.iter().filter_map(|t| {
                    serde_json::from_value::<ToolTask>(t.clone()).ok()
                }).collect();

                let results = super::parallel::execute_parallel_tasks(parsed_tasks, config);
                (json!({ "results": results, "total_executed": results.len() }), false)
            } else {
                (json!({ "error": "Missing 'tasks' array in arguments" }), true)
            }
        }
        unknown => (json!({ "error": format!("Tool '{}' not found", unknown) }), true),
    }
}

pub fn handle_json_rpc_message(msg: &Value, config: &AnalysisConfig) -> Option<Value> {
    let method = msg["method"].as_str().unwrap_or("");
    let id = msg.get("id").cloned();

    match method {
        "initialize" => {
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "docarmor-mcp",
                        "version": "0.3.0"
                    }
                }
            }))
        }
        "notifications/initialized" => {
            // Client acknowledgment; no response required
            None
        }
        "ping" => {
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            }))
        }
        "tools/list" => {
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": get_tools_list()
                }
            }))
        }
        "tools/call" => {
            let tool_name = msg["params"]["name"].as_str().unwrap_or("");
            let arguments = &msg["params"]["arguments"];
            let (result_val, is_err) = handle_tool_call(tool_name, arguments, config);

            let text_output = if let Some(s) = result_val.as_str() {
                s.to_string()
            } else {
                serde_json::to_string_pretty(&result_val).unwrap_or_else(|_| result_val.to_string())
            };

            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": text_output
                        }
                    ],
                    "isError": is_err
                }
            }))
        }
        _ => {
            if id.is_some() {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method '{}' not found", method)
                    }
                }))
            } else {
                None
            }
        }
    }
}

pub fn run_stdio_server(config: &AnalysisConfig) -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line_res in stdin.lock().lines() {
        let line = line_res?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(msg) = serde_json::from_str::<Value>(trimmed) {
            if let Some(resp) = handle_json_rpc_message(&msg, config) {
                let resp_str = serde_json::to_string(&resp).unwrap_or_default();
                writeln!(stdout, "{}", resp_str)?;
                stdout.flush()?;
            }
        } else {
            let err_resp = json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": {
                    "code": -32700,
                    "message": "Parse error: invalid JSON"
                }
            });
            writeln!(stdout, "{}", serde_json::to_string(&err_resp).unwrap_or_default())?;
            stdout.flush()?;
        }
    }

    Ok(())
}
