# Document Intelligence Gateway (DIG)

Document Intelligence Gateway (DIG) is a high-performance document analysis, validation, and guardrail platform built in Rust with Python bindings (`docgaurd` via PyO3 + Maturin).

DIG sits between raw document ingestion and downstream AI/LLM pipelines, providing document validation, exact token analysis, security scanning, quality scoring, routing recommendations, and preprocessing intelligence before documents enter RAG pipelines, embedding systems, vector databases, or LLM workflows.

---

## Where to Use DIG (Core Use Cases)

- **RAG Ingestion Guardrails**: Evaluate documents before chunking or embedding to detect empty files, scanned-only PDFs, low-quality structures, or unsupported formats, preventing downstream vector database waste.
- **AI Cost & Budget Management**: Compute exact OpenAI token counts and estimate embedding/LLM input processing costs dynamically before making expensive external API calls.
- **System Resource Security**: Intercept Zip bombs, compression bombs, resource exhaustion attacks (oversized files or excessive pages), and malformed structures at the gateway before they consume system memory.
- **Smart Chunking & Storage Routing**: Dynamically recommend chunking strategies (Hierarchical, Semantic, Fixed, or Agentic) and route incoming documents to downstream AI agents based on heuristic domain classifications.
- **Global De-duplication**: Check exact duplicate files concurrently across batches using high-performance cryptographic SHA-256 fingerprinting.

---

## Available Methods (Exposed to Python)

DIG exposes the `DocumentAnalyzer` class containing the following Python-accessible methods:

### Core Analysis Methods (Full Gateway Intelligence)
- `analyze_file(self, file_path: str) -> str`
  Parses and analyzes a single document located on the local disk. Returns a comprehensive JSON telemetry string.
- `analyze_bytes(self, content: bytes, file_name: str) -> str`
  Parses and analyzes an in-memory document byte buffer. Returns a comprehensive JSON telemetry string.
- `analyze_batch(self, file_paths: List[str]) -> str`
  Concurrently processes a list of document files in parallel across all CPU cores. Returns a JSON telemetry string containing aggregated batch statistics and individual file reports.
- `analyze_directory(self, dir_path: str, recursive: bool = True) -> str`
  Recursively discovers and batches all document files inside a directory in parallel. Returns a JSON telemetry string containing aggregated statistics and individual file reports.

### Ultra-Fast Single-Metric Helpers (Sub-Millisecond Bypasses)
- `count_words(self, file_path: str) -> int`
- `count_tokens(self, file_path: str) -> int`
- `count_chars(self, file_path: str) -> int`
- `count_words_bytes(self, content: bytes, file_name: str) -> int`
- `count_tokens_bytes(self, content: bytes, file_name: str) -> int`
- `count_chars_bytes(self, content: bytes, file_name: str) -> int`

---

## Installation

Install `docgaurd` instantly in your Python virtual environment using `pip`:

```bash
pip install docgaurd
```

*Note: No Rust compilers, C libraries, or Maturin installations are required. Pre-compiled native binary wheels are automatically provided for macOS, Windows, and Linux.*

---

## Python Integration Examples

Here is a comprehensive script demonstrating the use of all available methods:

```python
import json
import docgaurd

# 1. Initialize the Analyzer with customizable thresholds
analyzer = docgaurd.DocumentAnalyzer({
    "target_model": "gpt-4",                   # Model for context limits
    "tokenizer_name": "cl100k_base",           # Tokenizer profile to count with
    "embedding_rate_per_million": 0.02,        # Custom embedding price ($)
    "llm_input_rate_per_million": 5.00,        # Custom LLM input price ($)
    "max_file_size": 52428800                  # Max file size limit in bytes (50MB)
})

# ==============================================================================
# USE CASE A: Parsing a local file on disk
# ==============================================================================
try:
    report_json = analyzer.analyze_file("contract_agreement.pdf")
    report = json.loads(report_json)
    print(f"File Quality: {report['quality_score']} | Class: {report['document_class']}")
except FileNotFoundError:
    print("file not found on disk")

# ==============================================================================
# USE CASE B: Parsing an in-memory byte stream (e.g. from FastAPI/Django Uploads)
# ==============================================================================
uploaded_bytes = b"This is a sample document content containing generic text data for validation."
bytes_report = json.loads(analyzer.analyze_bytes(uploaded_bytes, "uploaded_file.txt"))
print(f"Tokens: {bytes_report['token_count']} | RAG Ready: {bytes_report['rag_ready']}")

# ==============================================================================
# USE CASE C: Batch processing multiple files in parallel (with failure isolation)
# ==============================================================================
file_queue = ["technical_specs.docx", "annual_report.pptx", "financial_sheet.xlsx"]
batch_report = json.loads(analyzer.analyze_batch(file_queue))
print(f"Processed: {batch_report['summary']['successful_files']} | Duplicates: {batch_report['summary']['duplicate_files']}")

# ==============================================================================
# USE CASE D: Scanning a complete directory recursively
# ==============================================================================
dir_report = json.loads(analyzer.analyze_directory("./archive", recursive=True))
print(f"Total directory tokens: {dir_report['summary']['total_tokens']}")

# ==============================================================================
# USE CASE E: Ultra-Fast Single-Metric Bypasses (Sub-Millisecond latency)
# ==============================================================================
word_count = analyzer.count_words("technical_specs.docx")
token_count = analyzer.count_tokens_bytes(uploaded_bytes, "uploaded_file.txt")
char_count = analyzer.count_chars("annual_report.pptx")
print(f"Single stats: words={word_count}, tokens={token_count}, chars={char_count}")
```

---

## Output Telemetry Schema

DIG generates a structured, metadata-rich telemetry report for every analyzed file:

```json
{
  "file_name": "contract_agreement.pdf",
  "file_type": "pdf",
  "sha256": "07c270b274dae324f906e0aa3a8d606471931e9c1afc241ddbc8f9ae52baffe7",
  "token_count": 2424,
  "word_count": 1612,
  "character_count": 11448,
  "page_count": 4,
  "requires_ocr": false,
  "quality_score": 0.8,
  "duplicate": false,
  "security_risk": "low",
  "fits_context": true,
  "rag_ready": true,
  "requires_summarization": false,
  "recommended_chunking": "semantic chunking",
  "document_class": "Legal",
  "recommended_agent": "LegalAgent",
  "estimated_embedding_cost": 0.0,
  "estimated_llm_cost": 0.0121,
  "processing_time_ms": 12.34
}
```

### Telemetry Field Descriptions

| Field | Type | Description |
| :--- | :--- | :--- |
| `file_name` | String | Base name of the analyzed file. |
| `file_type` | String | Lowercase file extension (e.g. `pdf`, `docx`, `txt`). |
| `sha256` | String | Cryptographic SHA-256 hash representing the exact content payload. |
| `token_count` | Integer | Exact token count matching the selected model tokenizer profile. |
| `word_count` | Integer | Number of words counted based on unicode whitespace dividers. |
| `character_count` | Integer | UTF-8 character length of the extracted document text. |
| `page_count` | Integer | Page count (e.g. PDF pages, PowerPoint slides, Excel sheets, estimated text lines). |
| `requires_ocr` | Boolean | Flags `true` if document has page structures but low text density (image-only scanned). |
| `quality_score` | Float | Cleanliness index (`0.0` - `1.0`) graded by density, metadata, ratio, and OCR markers. |
| `duplicate` | Boolean | Flags `true` if identical SHA-256 has already been processed in the concurrent batch queue. |
| `security_risk` | String | Security score (`low`, `medium`, `high`) validating Zip bombs and size thresholds. |
| `fits_context` | Boolean | Checks if `token_count` fits inside the target model's context window. |
| `rag_ready` | Boolean | Evaluates suitability for search databases (`true` if secure, non-scanned, and clean). |
| `requires_summarization` | Boolean | Recommends pre-summarizing if the token count or page density is excessively large. |
| `recommended_chunking` | String | Suggested chunking strategy (`no chunking`, `fixed`, `semantic`, `hierarchical`, `agentic`). |
| `document_class` | String | Classified topical domain (Finance, Procurement, Legal, HR, Tech Doc, Research, etc.). |
| `recommended_agent` | String | Recommended target downstream AI Agent target (e.g. `LegalAgent`). |
| `estimated_embedding_cost`| Float | Predicted vector database indexing cost. |
| `estimated_llm_cost` | Float | Predicted input processing cost. |
| `processing_time_ms` | Float | Internal Gateway execution latency in milliseconds. |

---

## License

This project is licensed under the [MIT License](LICENSE).
