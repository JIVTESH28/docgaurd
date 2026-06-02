
use std::sync::Arc;
use tiktoken_rs::CoreBPE;

pub struct RecommendationResult {
    pub token_count: usize,
    pub fits_context: bool,
    pub requires_summarization: bool,
    pub recommended_chunking: String,
    pub rag_ready: bool,
    pub estimated_embedding_cost: f64,
    pub estimated_llm_cost: f64,
}

pub fn get_token_count(text: &str, bpe_opt: &Option<Arc<CoreBPE>>, tokenizer_name: &str) -> usize {
    if let Some(ref bpe) = bpe_opt {
        bpe.encode_ordinary(text).len()
    } else {
        let bpe_res = match tokenizer_name {
            "r50k_base" => tiktoken_rs::r50k_base(),
            "p50k_base" => tiktoken_rs::p50k_base(),
            _ => tiktoken_rs::cl100k_base(), // default to cl100k_base used by GPT-4 and Claude
        };
        
        match bpe_res {
            Ok(bpe) => bpe.encode_ordinary(text).len(),
            Err(_) => text.len() / 4, // fallback approximation (approx 4 chars per token)
        }
    }
}

pub fn validate_context_window(token_count: usize, model_name: &str) -> bool {
    let limit = match model_name.to_lowercase().as_str() {
        "gpt-4" | "gpt-4-turbo" | "gpt-4o" => 128_000,
        "gpt-3.5-turbo" | "gpt-3.5" => 16_385,
        "claude-3" | "claude-3.5-sonnet" | "claude-3-opus" | "claude-3-haiku" => 200_000,
        "gemini-1.5" | "gemini-1.5-pro" | "gemini-1.5-flash" => 1_000_000,
        "llama-3" => 8_192,
        "mistral-7b" | "mixtral" => 32_768,
        _ => 8_192, // Safe default context window
    };
    token_count <= limit
}

pub fn recommend_chunking(
    token_count: usize,
    format: &str,
    domain: &str,
) -> String {
    if token_count < 512 {
        "no chunking".to_string()
    } else if format == "csv" || format == "xlsx" || format == "json" {
        "fixed chunking".to_string()
    } else if domain == "Research" && token_count > 15_000 {
        "agentic chunking".to_string()
    } else if format == "pdf" || format == "docx" || format == "pptx" {
        "hierarchical chunking".to_string()
    } else {
        "semantic chunking".to_string()
    }
}

pub fn generate_recommendations(
    text: &str,
    page_count: usize,
    format: &str,
    domain: &str,
    quality_score: f64,
    requires_ocr: bool,
    security_risk: &str,
    bpe_opt: &Option<Arc<CoreBPE>>,
    tokenizer_name: &str,
    target_model: &str,
    embedding_rate_per_million: f64,
    llm_input_rate_per_million: f64,
) -> RecommendationResult {
    let token_count = get_token_count(text, bpe_opt, tokenizer_name);
    let fits_context = validate_context_window(token_count, target_model);
    let requires_summarization = token_count > 4000 || page_count > 10;
    let recommended_chunking = recommend_chunking(token_count, format, domain);
    
    let rag_ready = quality_score > 0.4 && security_risk == "low" && !requires_ocr && token_count > 20;

    let estimated_embedding_cost = (token_count as f64 / 1_000_000.0) * embedding_rate_per_million;
    let estimated_llm_cost = (token_count as f64 / 1_000_000.0) * llm_input_rate_per_million;

    // Round values to 4 decimal places for cleanliness
    let estimated_embedding_cost = (estimated_embedding_cost * 10000.0).round() / 10000.0;
    let estimated_llm_cost = (estimated_llm_cost * 10000.0).round() / 10000.0;

    RecommendationResult {
        token_count,
        fits_context,
        requires_summarization,
        recommended_chunking,
        rag_ready,
        estimated_embedding_cost,
        estimated_llm_cost,
    }
}
