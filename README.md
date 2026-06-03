# DocGaurd (Document Intelligence Gateway)

[![Python 3.8+](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PyPI version](https://badge.fury.io/py/docgaurd.svg)](https://badge.fury.io/py/docgaurd)

DocGaurd (Document Intelligence Gateway) is a high-performance document validation, security scanning, quality guardrail, and exact token counting engine. Built in Rust with native Python bindings via PyO3, DocGaurd sits between raw document ingestion and downstream LLM/RAG pipelines to prevent system exploitation, database bloat, and unexpected API costs.

---

### Features • Installation • Quick Start • Python API • Telemetry Schema • Supported Formats • Examples • License

---

## Features

*   **✨ Real GPT Tokenization** - Integrates high-performance `tiktoken-rs` in Rust to calculate exact GPT token budgets (not approximations) for models like GPT-4, GPT-3.5, Claude, or LLaMA.
*   **⚡ Multi-Format Support** - Seamlessly extracts text and parses metadata from PDF, TXT, MD, DOCX, PPTX, XLSX, CSV, JSON, XML, and HTML files.
*   **🛡️ Ingestion Security** - Built-in security scanners inspect compressed documents and file headers to intercept Zip bombs, compression bombs, and oversized resource limits before they reach system memory.
*   **🔍 Text Quality & OCR Necessity Detection** - Evaluates page text density, whitespace-to-character ratio, and empty page signals to flag scanned/image-only documents (`requires_ocr`) before vector database embedding.
*   **🚀 Native Parallel Batch Processing** - Utilizes Rust's concurrent work-stealing thread pool (`Rayon`) to process thousands of files or directory trees in parallel with zero GIL serialization.
*   **💾 Global De-duplication** - Computes high-performance SHA-256 content hashes in parallel to identify and skip exact duplicate files inside a batch queue automatically.
*   **💰 Dynamic Cost Estimation** - Estimates LLM input cost and vector database embedding cost dynamically before making external API requests.
*   **🎯 Intelligent Agent Routing** - Classifies text based on heuristic token frequencies and assigns a target downstream AI Agent (e.g., `LegalAgent`, `ProcurementAgent`).
*   **🔒 Rust-Native PII Redaction & Data Masking** - Detects and masks Personally Identifiable Information (PII) like emails, phone numbers, SSNs, IP addresses, and credit cards directly in Rust before data leaves your environment.

---

## Installation

### From PyPI (Recommended)
Install pre-compiled native binary wheels instantly on Windows, Linux, or macOS:
```bash
pip install docgaurd
```
*(No Rust compilers, C-libraries, or compilation tools are required on the host system).*

### From Source
```bash
git clone https://github.com/JIVTESH28/docgaurd.git
cd docgaurd
pip install .
```

---

## Quick Start

### Initialize the Analyzer
```python
import json
import docgaurd

# Initialize the gateway analyzer with custom thresholds
analyzer = docgaurd.DocumentAnalyzer({
    "target_model": "gpt-4",                   # Target context window check
    "tokenizer_name": "cl100k_base",           # Tiktoken profile
    "embedding_rate_per_million": 0.02,        # Cost per 1M tokens ($)
    "llm_input_rate_per_million": 5.00,        # Cost per 1M tokens ($)
    "max_file_size": 52428800                  # Max file size (50MB)
})
```

### Python API Usage

#### Single File Ingestion (Local Disk)
```python
report_str = analyzer.analyze_file("contract.pdf")
report = json.loads(report_str)
print(f"Tokens: {report['token_count']} | RAG Ready: {report['rag_ready']}")
```

#### In-Memory Bytes Ingestion (API Uploads)
```python
uploaded_bytes = b"Sample document text buffer."
report_str = analyzer.analyze_bytes(uploaded_bytes, "invoice.txt")
report = json.loads(report_str)
print(f"Domain Class: {report['document_class']} | RAG Ready: {report['rag_ready']}")
```

#### Natively Parallel Batch Processing
```python
file_list = ["agreement.docx", "data.xlsx", "spec.pdf"]
batch_report_str = analyzer.analyze_batch(file_list)
batch_report = json.loads(batch_report_str)

print(f"Successful files: {batch_report['summary']['successful_files']}")
print(f"Duplicates skipped: {batch_report['summary']['duplicate_files']}")
```

#### Directory Ingestion (Recursive Scan)
```python
dir_report_str = analyzer.analyze_directory("./archive", recursive=True)
dir_report = json.loads(dir_report_str)
print(f"Total directory tokens: {dir_report['summary']['total_tokens']}")
```

#### Ultra-Fast Single-Metric Bypasses
If you only need a single metric and want to bypass the rest of the gateway analysis pipeline (such as security checks, cost estimation, and domain classification), use the sub-millisecond helpers:
```python
# Raw metric count helpers (File-based)
word_count = analyzer.count_words("document.docx")
char_count = analyzer.count_chars("document.docx")
token_count = analyzer.count_tokens("document.docx")

# Raw metric count helpers (Byte-based)
token_count = analyzer.count_tokens_bytes(uploaded_bytes, "invoice.txt")

# Rust-Native PII Redaction & Data Masking
pii_text = "My email is test@example.com and phone is 123-456-7890."
# Redact all supported categories (email, phone, ssn, ip, credit_card)
redacted_all = analyzer.redact_pii(pii_text) # "My email is [EMAIL] and phone is [PHONE]."
# Or redact only specific categories
redacted_email = analyzer.redact_pii(pii_text, ["email"]) # "My email is [EMAIL] and phone is 123-456-7890."
```

---

## Telemetry Output Schema

DocGaurd generates a comprehensive, metadata-rich telemetry report for every analyzed file:

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
  "contains_pii": true,
  "pii_categories_found": ["email", "phone"],
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
| `contains_pii` | Boolean | Flags `true` if document text contains common PII entities (email, phone, SSN, IP, credit card). |
| `pii_categories_found` | List | Names of PII categories found in the document (e.g., `["email", "phone"]`). |
| `estimated_embedding_cost`| Float | Predicted vector database indexing cost. |
| `estimated_llm_cost` | Float | Predicted input processing cost. |
| `processing_time_ms` | Float | Internal Gateway execution latency in milliseconds. |

---

## Supported Formats

| Format | Extension | Extraction Method | Key Features |
| :--- | :--- | :--- | :--- |
| **PDF** | `.pdf` | Native lopdf Parser | Structural reading, scanned detection, page extraction |
| **Word** | `.docx` | Native docx XML Parser | Direct paragraph and table text extraction |
| **PowerPoint** | `.pptx` | Native pptx XML Parser | Shape text, slide processing, bullet analysis |
| **Excel** | `.xlsx` | Calamine Engine | Spreadsheet parsing, cell extraction, rows estimation |
| **CSV** | `.csv` | CSV Parser | Direct row, column parsing, delimiter validation |
| **Plain Text** | `.txt`, `.md` | Unicode Parser | Streaming flat extraction, lossy fallback encoding |
| **JSON** | `.json` | Serde JSON | Recursive nested key-value string extraction |
| **XML** | `.xml` | Quick XML Parser | Tag-stripped text, element-wise traversal |
| **HTML** | `.html` | Quick XML Parser | Element parsing, script/style extraction filtering |

---

## Configuration Limits

| Setting | Default Value | Purpose |
| :--- | :--- | :--- |
| `target_model` | `"gpt-4"` | Target context size limit check |
| `tokenizer_name` | `"cl100k_base"` | Tokenizer profile (cl100k_base, r50k_base, p50k_base) |
| `max_file_size` | `52,428,800` bytes (50MB) | Intercept oversized documents |
| `embedding_rate_per_million` | `$0.02` | Custom embedding cost rate |
---

## How the OCR Integration Works

DocGaurd implements a high-performance **hybrid OCR gateway** under the `OcrDocumentAnalyzer` class:

1. **Rust-Native Gatekeeping**:
   When a file is submitted, DocGaurd first uses its sub-millisecond Rust parsers to check the file type and structure.
   - If the document is a clean digital file (e.g., text PDF, Word doc, or markdown), the text is extracted instantly, and the heavy OCR engine is completely bypassed.
   - If the file is an image (`.png`, `.jpg`, `.jpeg`, etc.) or is flagged by the Rust quality scanner as a scanned/text-empty PDF (`requires_ocr: True`), the OCR engine is initialized.
2. **Lazy Loading**:
   To keep package imports sub-millisecond, PyTorch and EasyOCR model weights are loaded lazily on-demand *only* when the first scanned document or raw image is encountered.
3. **Hardware Auto-Detection**:
   The engine dynamically autodetects your host hardware to run deep learning models at maximum speed:
   - **macOS (Apple Silicon)**: Natively offloads tensor computations to the GPU via **Metal Performance Shaders (MPS)**.
   - **Windows/Linux with GPU**: Automatically targets your Nvidia GPU via **CUDA**.
   - **Fallback**: Runs on optimized multi-threaded **CPU**.
4. **Rust Telemetry Reconciliation**:
   Once text is extracted via OCR, the raw text bytes are passed back into DocGaurd's Rust core using a virtual text buffer. The Rust engine then computes exact GPT token budgets (`tiktoken-rs`), counts words/characters, runs domain classification, and generates cost estimations—reconciling all statistics back into a single unified JSON schema.

---

## Examples

### Example 1: RAG Ingestion Security & Quality Gatekeeper
Ensure that only secure, high-quality, digital documents enter your vector database:
```python
import json
import docgaurd

analyzer = docgaurd.DocumentAnalyzer()
report = json.loads(analyzer.analyze_file("user_upload.pdf"))

# Intercept risks at the gateway
if report["security_risk"] == "high":
    raise ValueError(f"CRITICAL: Security exception triggered for {report['file_name']}")

if report["requires_ocr"]:
    print(f"Routing {report['file_name']} to hardware-accelerated OCR pipeline.")
elif not report["rag_ready"]:
    print(f"Skipping {report['file_name']} due to low text quality score: {report['quality_score']}")
else:
    print(f"Ingesting clean document text. Context Size: {report['token_count']} tokens.")
```

### Example 2: API Cost Budgeting & Model Window Check
Calculate API transaction costs and verify if a document fits within a model's context window:
```python
import json
import docgaurd

analyzer = docgaurd.DocumentAnalyzer({
    "target_model": "gpt-3.5-turbo",
    "llm_input_rate_per_million": 1.50
})

report = json.loads(analyzer.analyze_file("long_transcript.txt"))

if not report["fits_context"]:
    print(f"Document exceeds target context window. Recommended chunking strategy: {report['recommended_chunking']}")
else:
    print(f"Document fits. Estimated processing cost: ${report['estimated_llm_cost']:.4f}")
```

### Example 3: Hardware-Accelerated OCR Integration (Metal/CUDA)
Incorporate unified OCR for scanned files directly from the installed package:
```python
import json
from docgaurd import OcrDocumentAnalyzer

# Initialize unified OcrDocumentAnalyzer (auto-routes to Apple Metal MPS or CUDA)
gateway = OcrDocumentAnalyzer()

report_json = gateway.analyze_file("scanned_receipt.jpg")
report = json.loads(report_json)

print(f"OCR Text: {report['text']}")
print(f"OCR Tokens: {report['token_count']} | RAG Ready: {report['rag_ready']}")
```

---

## License

This project is licensed under the [MIT License](LICENSE).
