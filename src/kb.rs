use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Instant;
use rayon::prelude::*;
use serde_json::{json, Value};

use super::parsers::parse_document;
use super::classifier::classify_domain;
use super::pii::detect_pii;
use super::recommendations::get_token_count;
use super::batch::AnalysisConfig;

pub fn get_model_rate(model_name: &str, config: &AnalysisConfig) -> f64 {
    let lower = model_name.to_lowercase();
    
    // 1. Exact match in dynamic registry
    if let Some(&rate) = config.model_rates.get(&lower) {
        return rate;
    }

    // 2. Substring match against any registered keys in user registry
    for (key, &rate) in &config.model_rates {
        if lower.contains(key.as_str()) {
            return rate;
        }
    }

    // 3. Fully dynamic fallback to user's configured default rate
    config.llm_input_rate_per_million
}

pub fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for c in text.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if (c == ' ' || c == '-' || c == '_') && !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

pub fn is_code_file(ext: &str) -> bool {
    matches!(
        ext,
        "rs" | "py" | "go" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "h" | "hpp" | "sh" | "bash" | "sql"
    )
}

pub fn extract_code_signatures(text: &str, ext: &str) -> Vec<String> {
    let mut sigs = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
            continue;
        }
        
        let is_sig = match ext {
            "rs" => trimmed.starts_with("pub fn") || trimmed.starts_with("fn ") || trimmed.starts_with("pub struct") || trimmed.starts_with("pub enum") || trimmed.starts_with("impl"),
            "py" => trimmed.starts_with("def ") || trimmed.starts_with("class ") || trimmed.starts_with("async def "),
            "go" => trimmed.starts_with("func ") || trimmed.starts_with("type "),
            "js" | "ts" | "jsx" | "tsx" => trimmed.starts_with("function ") || trimmed.starts_with("export ") || trimmed.starts_with("class ") || trimmed.starts_with("interface ") || trimmed.starts_with("type "),
            "java" | "cpp" | "c" | "h" => trimmed.starts_with("public ") || trimmed.starts_with("private ") || trimmed.starts_with("class ") || trimmed.starts_with("struct "),
            _ => trimmed.starts_with("def ") || trimmed.starts_with("fn ") || trimmed.starts_with("func "),
        };

        if is_sig {
            let sig_clean = if trimmed.len() > 100 {
                format!("{}...", &trimmed[..100])
            } else {
                trimmed.to_string()
            };
            sigs.push(sig_clean);
            if sigs.len() >= 25 {
                break;
            }
        }
    }
    sigs
}

#[derive(Debug, Clone)]
pub struct SectionBlock {
    pub title: String,
    pub level: usize,
    pub lines: Vec<String>,
}

pub fn parse_document_sections(text: &str, _ext: &str, domain_class: &str) -> Vec<SectionBlock> {
    let mut sections: Vec<SectionBlock> = Vec::new();
    let mut current_title = format!("1. Core {} Overview", domain_class);
    let mut current_level = 2;
    let mut current_lines = Vec::new();
    let mut section_counter = 1;
    let mut in_code_block = false;

    for line in text.lines() {
        let trimmed = line.trim();
        
        // Track code fence toggle to prevent splitting inside code blocks
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            current_lines.push(line.to_string());
            continue;
        }

        if in_code_block {
            current_lines.push(line.to_string());
            continue;
        }

        // Markdown heading detection: line starts with one or more '#' followed by space
        let hash_count = line.chars().take_while(|&c| c == '#').count();
        let is_md_header = hash_count > 0 && line[hash_count..].starts_with(' ');
        
        // Non-markdown section keywords: "Section X:", "Chapter X:"
        let lower = trimmed.to_lowercase();
        let is_keyword_header = (lower.starts_with("section ") || lower.starts_with("chapter ") || lower.starts_with("part "))
            && trimmed.contains(':') && trimmed.len() < 80;

        if is_md_header || is_keyword_header {
            if !current_lines.is_empty() {
                sections.push(SectionBlock {
                    title: current_title.clone(),
                    level: current_level,
                    lines: current_lines.clone(),
                });
                current_lines.clear();
            }

            section_counter += 1;
            if is_md_header {
                current_level = hash_count;
                let clean_title = line[hash_count..].trim();
                current_title = if clean_title.is_empty() {
                    format!("{}. Section {}", section_counter, section_counter)
                } else {
                    format!("{}. {}", section_counter, clean_title)
                };
            } else {
                current_level = 2;
                current_title = format!("{}. {}", section_counter, trimmed);
            }
        } else {
            current_lines.push(line.to_string());
        }
    }

    if !current_lines.is_empty() || sections.is_empty() {
        sections.push(SectionBlock {
            title: current_title,
            level: current_level,
            lines: current_lines,
        });
    }

    sections
}

pub fn convert_single_to_kb(
    content: &[u8],
    file_name: &str,
    target_model: &str,
    config: &AnalysisConfig,
) -> Value {
    convert_single_to_kb_with_mode(content, file_name, target_model, "full", config)
}

pub fn convert_single_to_kb_with_mode(
    content: &[u8],
    file_name: &str,
    target_model: &str,
    mode: &str,
    config: &AnalysisConfig,
) -> Value {
    let start_time = Instant::now();
    let ext = file_name.split('.').last().unwrap_or("").to_lowercase();
    let mut doc = parse_document(content, file_name);

    if doc.text.is_empty() && !content.is_empty() {
        let fallback_str = String::from_utf8_lossy(content).into_owned();
        if !fallback_str.trim().is_empty() {
            doc.text = fallback_str;
            doc.metadata.remove("unsupported_format");
            if doc.page_count == 0 {
                doc.page_count = 1;
            }
        }
    }

    if doc.text.is_empty() || content.is_empty() {
        return json!({
            "markdown": format!("# Error\n\nCould not parse document `{}` (unsupported format or empty content).", file_name),
            "telemetry": {
                "file_name": file_name,
                "file_type": ext,
                "target_model": target_model,
                "mode": mode,
                "raw_tokens": 0,
                "kb_tokens": 0,
                "tokens_saved": 0,
                "reduction_percentage": 0.0,
                "estimated_raw_cost_usd": 0.0,
                "estimated_kb_cost_usd": 0.0,
                "cost_savings_usd": 0.0,
                "processing_time_ms": (start_time.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0,
                "document_class": "Unknown",
                "contains_pii": false,
                "pii_categories_found": Vec::<String>::new()
            }
        });
    }

    let text_tokens = get_token_count(&doc.text, &config.bpe, &config.tokenizer_name);
    let is_pdf_or_image = ext == "pdf" || ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "tiff" || ext == "webp";
    
    // Calculate raw multi-modal vision/PDF/document overhead for LLM input
    let raw_tokens = if is_pdf_or_image && doc.page_count > 0 {
        std::cmp::max(text_tokens, doc.page_count * 2000)
    } else {
        std::cmp::max(text_tokens, (doc.text.chars().count() as f64 / 3.5) as usize)
    };

    let (domain_class, agent_target) = classify_domain(&doc.text);
    let pii_found = detect_pii(&doc.text);
    let contains_pii = !pii_found.is_empty();

    // Extract genuine, informative summary takeaways (skip markdown badges, image links, table separators)
    let mut summary_takeaways = Vec::new();
    for line in doc.text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() 
            || trimmed.starts_with('#') 
            || trimmed.starts_with("---")
            || trimmed.starts_with("***")
            || trimmed.starts_with("```")
            || trimmed.starts_with("![")
            || trimmed.starts_with("[![")
            || trimmed.starts_with('<')
            || trimmed.starts_with('|')
            || trimmed.len() < 20
        {
            continue;
        }
        let clean = trimmed.trim_start_matches(|c| c == '*' || c == '-' || c == '+' || c == ' ').trim();
        let display = if clean.len() > 140 {
            format!("{}...", &clean[..140])
        } else {
            clean.to_string()
        };
        if !summary_takeaways.contains(&display) {
            summary_takeaways.push(display);
            if summary_takeaways.len() >= 6 {
                break;
            }
        }
    }
    if summary_takeaways.is_empty() {
        summary_takeaways.push(format!(
            "Document categorized under {} containing {} characters across {} pages.",
            domain_class, doc.text.chars().count(), doc.page_count
        ));
    }

    let code_signatures = if is_code_file(&ext) {
        extract_code_signatures(&doc.text, &ext)
    } else {
        Vec::new()
    };

    let section_blocks = parse_document_sections(&doc.text, &ext, &domain_class);

    // Build Table of Contents with nested hierarchy and CommonMark-compliant slugs
    let mut toc_markdown = String::from("## Table of Contents\n\n");
    toc_markdown.push_str("- [Executive Summary](#executive-summary)\n");
    toc_markdown.push_str("- [Domain Taxonomy & Governance](#domain-taxonomy-governance)\n");
    if !code_signatures.is_empty() {
        toc_markdown.push_str("- [Code Architecture & Symbol Outline](#code-architecture-symbol-outline)\n");
    }
    
    for section in &section_blocks {
        let slug = slugify(&section.title);
        let indent = if section.level > 2 {
            "  ".repeat(section.level - 2)
        } else {
            String::new()
        };
        toc_markdown.push_str(&format!("{}- [{}](#{})\n", indent, section.title, slug));
    }
    toc_markdown.push_str("- [Navigation & Index](#navigation-index)\n\n");

    let mut kb_md = String::new();
    kb_md.push_str(&format!("# Knowledge Base: {}\n\n", file_name));
    kb_md.push_str(&format!(
        "> **DocArmor Knowledge Base Layer** | Target Model: `{}` | Class: `{}` | Target Agent: `{}` | Mode: `{}`\n\n",
        target_model, domain_class, agent_target, mode
    ));
    kb_md.push_str("---\n\n");

    kb_md.push_str(&toc_markdown);
    kb_md.push_str("---\n\n");

    // Executive Summary Section
    kb_md.push_str("## Executive Summary\n\n");
    kb_md.push_str(&format!(
        "This document is classified under **{}**. Contains {} page(s) and {} extracted characters.\n\n",
        domain_class, doc.page_count, doc.text.chars().count()
    ));
    kb_md.push_str("### Key Takeaways\n");
    for takeaway in &summary_takeaways {
        kb_md.push_str(&format!("- {}\n", takeaway));
    }
    kb_md.push_str("\n[↑ Back to Table of Contents](#table-of-contents)\n\n");
    kb_md.push_str("---\n\n");

    // Domain Taxonomy Section
    kb_md.push_str("## Domain Taxonomy & Governance\n\n");
    kb_md.push_str("| Metric / Attribute | Value |\n");
    kb_md.push_str("| :--- | :--- |\n");
    kb_md.push_str(&format!("| **Document Class** | `{}` |\n", domain_class));
    kb_md.push_str(&format!("| **Target Agent** | `{}` |\n", agent_target));
    kb_md.push_str(&format!("| **Contains PII** | `{}` |\n", contains_pii));
    if contains_pii {
        kb_md.push_str(&format!("| **PII Categories** | `{}` |\n", pii_found.join(", ")));
    }
    kb_md.push_str(&format!("| **Page Count** | `{}` |\n", doc.page_count));
    kb_md.push_str("\n[↑ Back to Table of Contents](#table-of-contents)\n\n");
    kb_md.push_str("---\n\n");

    // Code Architecture Section (if code file)
    if !code_signatures.is_empty() {
        kb_md.push_str("## Code Architecture & Symbol Outline\n\n");
        kb_md.push_str("```\n");
        for sig in &code_signatures {
            kb_md.push_str(sig);
            kb_md.push('\n');
        }
        kb_md.push_str("```\n\n");
        kb_md.push_str("[↑ Back to Table of Contents](#table-of-contents)\n\n");
        kb_md.push_str("---\n\n");
    }

    // Knowledge Modules: emit each section completely, respecting the requested mode
    for section in &section_blocks {
        let slug = slugify(&section.title);
        kb_md.push_str(&format!("## {}\n\n", section.title));

        match mode {
            "compact" => {
                // Deduplicate consecutive blank lines and strip noise
                let mut prev_empty = false;
                let mut compact_lines = Vec::new();
                for l in &section.lines {
                    let trimmed = l.trim();
                    if trimmed.is_empty() {
                        if !prev_empty {
                            compact_lines.push("");
                            prev_empty = true;
                        }
                    } else {
                        prev_empty = false;
                        compact_lines.push(l.as_str());
                    }
                }
                let body = compact_lines.join("\n");
                if is_code_file(&ext) && !body.starts_with("```") {
                    kb_md.push_str(&format!("```{}\n{}\n```\n\n", ext, body));
                } else {
                    kb_md.push_str(&body);
                    kb_md.push_str("\n\n");
                }
            }
            "outline" => {
                // Preview only top 3 lines of section
                let preview: Vec<&str> = section.lines.iter()
                    .map(|s| s.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .take(3)
                    .collect();
                if !preview.is_empty() {
                    kb_md.push_str(&preview.join("\n"));
                    if section.lines.len() > 3 {
                        kb_md.push_str("\n\n*(... content summarized in outline mode ...)*\n\n");
                    } else {
                        kb_md.push_str("\n\n");
                    }
                }
            }
            _ => {
                // "full" mode: full section content preserved intact
                let body = section.lines.join("\n");
                if is_code_file(&ext) && !body.starts_with("```") {
                    kb_md.push_str(&format!("```{}\n{}\n```\n\n", ext, body));
                } else {
                    kb_md.push_str(&body);
                    kb_md.push_str("\n\n");
                }
            }
        }

        kb_md.push_str(&format!("[↑ Back to Table of Contents](#table-of-contents) | [Anchor Link](#{})\n\n---\n\n", slug));
    }

    // Navigation & Index
    kb_md.push_str("## Navigation & Index\n\n");
    kb_md.push_str("- [Executive Summary](#executive-summary)\n");
    kb_md.push_str("- [Domain Taxonomy & Governance](#domain-taxonomy-governance)\n");
    for section in &section_blocks {
        let slug = slugify(&section.title);
        kb_md.push_str(&format!("- [{}](#{})\n", section.title, slug));
    }
    kb_md.push_str("\n*Generated automatically by DocArmor Pre-Ingestion Knowledge Base Engine.*\n");

    let kb_tokens = get_token_count(&kb_md, &config.bpe, &config.tokenizer_name);
    let tokens_saved = if raw_tokens > kb_tokens { raw_tokens - kb_tokens } else { 0 };
    let reduction_percentage = if raw_tokens > 0 {
        ((tokens_saved as f64) / (raw_tokens as f64)) * 100.0
    } else {
        0.0
    };

    let model_rate = get_model_rate(target_model, config);
    let estimated_raw_cost_usd = ((raw_tokens as f64) / 1_000_000.0) * model_rate;
    let estimated_kb_cost_usd = ((kb_tokens as f64) / 1_000_000.0) * model_rate;
    let cost_savings_usd = estimated_raw_cost_usd - estimated_kb_cost_usd;

    let processing_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

    json!({
        "markdown": kb_md,
        "telemetry": {
            "file_name": file_name,
            "file_type": ext,
            "target_model": target_model,
            "mode": mode,
            "raw_tokens": raw_tokens,
            "kb_tokens": kb_tokens,
            "tokens_saved": tokens_saved,
            "reduction_percentage": (reduction_percentage * 10.0).round() / 10.0,
            "estimated_raw_cost_usd": (estimated_raw_cost_usd * 100000.0).round() / 100000.0,
            "estimated_kb_cost_usd": (estimated_kb_cost_usd * 100000.0).round() / 100000.0,
            "cost_savings_usd": (cost_savings_usd * 100000.0).round() / 100000.0,
            "processing_time_ms": (processing_time_ms * 100.0).round() / 100.0,
            "document_class": domain_class,
            "recommended_agent": agent_target,
            "contains_pii": contains_pii,
            "pii_categories_found": pii_found
        }
    })
}

fn collect_directory_files(dir: &Path, recursive: bool, files: &mut Vec<String>) {
    let ignore_dirs: HashSet<&str> = [
        ".git", "target", "node_modules", "venv", "__pycache__", ".idea", ".vscode",
        "dist", "build", ".next", ".cache"
    ].iter().cloned().collect();

    let ignore_exts: HashSet<&str> = [
        "so", "dylib", "dll", "exe", "pyc", "o", "a", "zip", "tar", "gz"
    ].iter().cloned().collect();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().map(|s| s.to_string_lossy()).unwrap_or_default();

            if path.is_file() {
                let ext = path.extension().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
                if !ignore_exts.contains(ext.as_str()) {
                    if let Some(p_str) = path.to_str() {
                        files.push(p_str.to_string());
                    }
                }
            } else if path.is_dir() && recursive {
                if !ignore_dirs.contains(name.as_ref()) {
                    collect_directory_files(&path, recursive, files);
                }
            }
        }
    }
}

pub fn convert_directory_to_kb(
    dir_path: &str,
    recursive: bool,
    target_model: &str,
    config: &AnalysisConfig,
) -> Value {
    convert_directory_to_kb_with_mode(dir_path, recursive, target_model, "full", config)
}

pub fn convert_directory_to_kb_with_mode(
    dir_path: &str,
    recursive: bool,
    target_model: &str,
    mode: &str,
    config: &AnalysisConfig,
) -> Value {
    let start_time = Instant::now();
    let root_path = Path::new(dir_path);

    if !root_path.is_dir() {
        return json!({
            "markdown": format!("# Error\n\nProvided path `{}` is not a valid directory.", dir_path),
            "telemetry": {
                "dir_path": dir_path,
                "target_model": target_model,
                "error": "Not a directory"
            }
        });
    }

    let project_name = root_path.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| dir_path.to_string());

    let mut file_paths = Vec::new();
    collect_directory_files(root_path, recursive, &mut file_paths);

    // Read and parse files in parallel using Rayon
    let file_results: Vec<(String, String, Value)> = file_paths
        .par_iter()
        .filter_map(|p_str| {
            let path = Path::new(p_str);
            let rel_path = path.strip_prefix(root_path)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| p_str.clone());

            let bytes = fs::read(path).ok()?;
            let result = convert_single_to_kb_with_mode(&bytes, &rel_path, target_model, mode, config);
            Some((rel_path, p_str.clone(), result))
        })
        .collect();

    let mut total_raw_tokens: usize = 0;
    let mut file_entries: Vec<Value> = Vec::new();
    let mut tree_markdown = String::from("## Project File Directory Tree & Module Index\n\n");
    let mut combined_sections_md = String::new();

    for (rel_path, _full_path, kb_val) in &file_results {
        let telemetry = &kb_val["telemetry"];
        let raw = telemetry["raw_tokens"].as_u64().unwrap_or(0) as usize;
        let file_kb_tok = telemetry["kb_tokens"].as_u64().unwrap_or(0) as usize;

        total_raw_tokens += raw;

        let file_slug = slugify(rel_path);
        tree_markdown.push_str(&format!(
            "- [`{}`](#file-{}) — **{}** raw tokens | Class: `{}`\n",
            rel_path,
            file_slug,
            raw,
            telemetry["document_class"].as_str().unwrap_or("Unknown")
        ));

        file_entries.push(json!({
            "file_path": rel_path,
            "raw_tokens": raw,
            "kb_tokens": file_kb_tok,
            "reduction_percentage": telemetry["reduction_percentage"]
        }));

        combined_sections_md.push_str(&format!("<a id=\"file-{}\"></a>\n", file_slug));
        combined_sections_md.push_str(&format!("### File Module: `{}`\n\n", rel_path));
        combined_sections_md.push_str(&format!(
            "> Class: `{}` | Target Agent: `{}`\n\n",
            telemetry["document_class"].as_str().unwrap_or("General"),
            telemetry["recommended_agent"].as_str().unwrap_or("GeneralAgent")
        ));

        if let Some(md_text) = kb_val["markdown"].as_str() {
            // Strip redundant top title and TOC so modules cleanly embed into the project brain
            let mut skip_header = true;
            let mut module_lines = Vec::new();
            for l in md_text.lines() {
                if skip_header {
                    if l.starts_with("## Executive Summary") {
                        skip_header = false;
                        module_lines.push(l);
                    }
                } else if !l.starts_with("## Navigation & Index") {
                    module_lines.push(l);
                }
            }
            if module_lines.is_empty() {
                combined_sections_md.push_str(md_text);
            } else {
                combined_sections_md.push_str(&module_lines.join("\n"));
            }
        }
        combined_sections_md.push_str("\n\n[↑ Back to Project Index](#project-file-directory-tree-module-index)\n\n---\n\n");
    }

    let mut project_kb_md = String::new();
    project_kb_md.push_str(&format!("# Project Knowledge Base: {}\n\n", project_name));
    project_kb_md.push_str(&format!(
        "> **DocArmor Unified Project Brain** | Target Model: `{}` | Total Files: `{}` | Mode: `{}`\n\n",
        target_model, file_results.len(), mode
    ));
    project_kb_md.push_str("---\n\n");

    project_kb_md.push_str(&tree_markdown);
    project_kb_md.push_str("\n---\n\n");

    project_kb_md.push_str("## Module Knowledge Breakdown\n\n");
    project_kb_md.push_str(&combined_sections_md);
    project_kb_md.push_str("\n*Generated automatically by DocArmor Project Knowledge Base Engine.*\n");

    let total_kb_tokens = get_token_count(&project_kb_md, &config.bpe, &config.tokenizer_name);

    let tokens_saved = if total_raw_tokens > total_kb_tokens {
        total_raw_tokens - total_kb_tokens
    } else {
        0
    };

    let reduction_percentage = if total_raw_tokens > 0 {
        ((tokens_saved as f64) / (total_raw_tokens as f64)) * 100.0
    } else {
        0.0
    };

    let model_rate = get_model_rate(target_model, config);
    let estimated_raw_cost_usd = ((total_raw_tokens as f64) / 1_000_000.0) * model_rate;
    let estimated_kb_cost_usd = ((total_kb_tokens as f64) / 1_000_000.0) * model_rate;
    let cost_savings_usd = estimated_raw_cost_usd - estimated_kb_cost_usd;

    let total_processing_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

    json!({
        "markdown": project_kb_md,
        "telemetry": {
            "project_name": project_name,
            "dir_path": dir_path,
            "target_model": target_model,
            "mode": mode,
            "total_files": file_results.len(),
            "raw_tokens": total_raw_tokens,
            "kb_tokens": total_kb_tokens,
            "tokens_saved": tokens_saved,
            "reduction_percentage": (reduction_percentage * 10.0).round() / 10.0,
            "estimated_raw_cost_usd": (estimated_raw_cost_usd * 100000.0).round() / 100000.0,
            "estimated_kb_cost_usd": (estimated_kb_cost_usd * 100000.0).round() / 100000.0,
            "cost_savings_usd": (cost_savings_usd * 100000.0).round() / 100000.0,
            "processing_time_ms": (total_processing_time_ms * 100.0).round() / 100.0,
            "files_summary": file_entries
        }
    })
}
