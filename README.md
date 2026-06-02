# Document Intelligence Gateway (DIG)

Document Intelligence Gateway (DIG) is a high-performance document analysis and guardrail platform built in Rust with Python bindings (`docgaurd` via PyO3 + Maturin).

DIG sits between document ingestion and downstream AI systems, providing document validation, exact token analysis, security scanning, quality scoring, routing recommendations, and preprocessing intelligence before documents enter RAG pipelines, embedding systems, vector databases, or LLM workflows.

---

## Key Features

- **Multi-Format Parsers**: Pure Rust high-performance parsing of **PDF**, **DOCX**, **PPTX**, **XLSX**, **CSV**, **JSON**, **XML**, **HTML**, **MD**, and **TXT** files with zero native shared-library dependencies.
- **Resource & Security Scanning**: Integrated protection against zip bombs, compression bombs, oversized document uploads, and structural corruptions.
- **Accuracy Token Counting**: Exact token calculation using `tiktoken-rs` supporting `cl100k_base` (GPT-4 / Claude), `r50k_base` (GPT-3), and `p50k_base` tokenizers.
- **Quality & OCR Evaluation**: Scans text densities and digital ratio signatures to automatically identify scanned/unextractable inputs requiring OCR.
- **Keyword Classification & AI Agent Routing**: Instantly classifies files into Finance, Supply Planning, Manufacturing, Procurement, Legal, HR, Technical Documentation, and Research domains, recommending target downstream AI Agents (e.g., `SupplyPlanningAgent`).
- **Chunking Recommendations**: Selects optimal chunking profiles (Semantic, Hierarchical, Fixed, or Agentic) based on structure, density, and size.
- **Rayon Batch Parallelism**: Processes thousands of documents concurrently utilizing all available CPU cores with fully isolated failure recovery.

---

## Installation

Install `docgaurd` instantly in your Python virtual environment:

```bash
pip install docgaurd
```

No Rust compilers, C dependencies, or Maturin installations are required. Pre-compiled native binary wheels are automatically served for macOS, Windows, and Linux.

---

## Python Usage Example

```python
import json
import docgaurd

# 1. Initialize DocumentAnalyzer with custom rates and parameters
analyzer = docgaurd.DocumentAnalyzer({
    "target_model": "gpt-4",
    "tokenizer_name": "cl100k_base",
    "embedding_rate_per_million": 0.02,
    "llm_input_rate_per_million": 5.00,
    "max_file_size": 52428800 # 50MB
})

# 2. Analyze individual files or byte buffers
with open("quarterly_report.pdf", "rb") as f:
    pdf_bytes = f.read()

result_json = analyzer.analyze_bytes(pdf_bytes, "quarterly_report.pdf")
result = json.loads(result_json)
print(json.dumps(result, indent=2))

# 3. Analyze complete directories concurrently
directory_results = json.loads(analyzer.analyze_directory("./documents", recursive=True))
print(directory_results["summary"])
```

---

## Output Telemetry Schema

DIG generates a structured, metadata-rich telemetry report for every analyzed file:

```json
{
  "file_name": "sample.pdf",
  "file_type": "pdf",
  "sha256": "49c04a8085576f7ad01b883f9bef79fecaea3df501929457fa12c8d8c2f29991",
  "token_count": 183921,
  "word_count": 130281,
  "character_count": 928182,
  "page_count": 320,
  "requires_ocr": false,
  "quality_score": 0.91,
  "duplicate": false,
  "security_risk": "low",
  "fits_context": true,
  "rag_ready": true,
  "requires_summarization": true,
  "recommended_chunking": "hierarchical",
  "document_class": "Supply Planning",
  "recommended_agent": "SupplyPlanningAgent",
  "estimated_embedding_cost": 1.92,
  "estimated_llm_cost": 6.34,
  "processing_time_ms": 12.34
}
```

---

## License

This project is licensed under the [MIT License](LICENSE).
