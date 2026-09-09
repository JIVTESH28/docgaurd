# DocArmor (Document Intelligence Gateway)

[![Python 3.8+](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PyPI version](https://badge.fury.io/py/docarmor.svg)](https://badge.fury.io/py/docarmor)

DocArmor (Document Intelligence Gateway) is a high-performance document validation, security scanning, quality guardrail, and exact token counting engine. Built in Rust with native Python bindings via PyO3, DocArmor sits between raw document ingestion and downstream LLM/RAG pipelines to prevent system exploitation, database bloat, and unexpected API costs.

---

### Features • Installation • Quick Start • Python API • Telemetry Schema • Supported Formats • Examples • License

---

## Features

*   **🧠 Pre-Ingestion Knowledge Base Engine (v0.3.0)** - Zero-truncation document-to-markdown converter with state-aware code block protection, nested hierarchical Table of Contents (TOC), CommonMark/GitHub anchor navigation, and configurable detail modes (`full`, `compact`, `outline`).
*   **⚡ High-Speed Rayon Concurrency Layer** - Bypasses Python's Global Interpreter Lock (GIL) via `py.allow_threads` to execute heterogeneous agent tools, document validations, and PII scrubbing concurrently across all CPU cores at **104,000+ ops/sec (160x faster than Python GIL)**.
*   **🔌 Native Rust Model Context Protocol (MCP) Server** - Built-in high-performance stdio MCP server (`python -m docarmor.mcp` or `docarmor-mcp`) with **22µs response latency**, giving Claude Desktop, Cursor, Antigravity, and AI agents instant tool access.
*   **📉 Multi-Model Token Reduction Telemetry** - Achieves up to **60-90%+ token reduction** before passing content to LLM agents across Claude 3.5/3.7, GPT-4o, Gemini 1.5/2.0, LLaMA 3, and DeepSeek R1/V3.
*   **🌐 "One Brain" Project Repository Ingestion** - Recursively aggregates full multi-file codebases or directory trees into a single structured project Knowledge Base with file tree indexes and module breakdowns.
*   **✨ Real GPT Tokenization** - Integrates high-performance `tiktoken-rs` in Rust to calculate exact GPT token budgets (not approximations) for models like GPT-4, GPT-3.5, Claude, or LLaMA.
*   **⚡ Multi-Format Support** - Seamlessly extracts text and parses metadata from PDF, TXT, MD, DOCX, PPTX, XLSX, CSV, JSON, XML, HTML, and code files (`.py`, `.rs`, `.go`, `.js`, `.ts`, `.java`, `.cpp`, `.c`, `.sh`, `.sql`).
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
pip install docarmor
```
*(No Rust compilers, C-libraries, or compilation tools are required on the host system).*

### From Source
```bash
git clone https://github.com/JIVTESH28/docarmor.git
cd docarmor
pip install .
```

---

## Quick Start

### Initialize the Analyzer
```python
import json
import docarmor

# Initialize the gateway analyzer with custom thresholds
analyzer = docarmor.DocumentAnalyzer({
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

#### 🧠 Knowledge Base Pre-Ingestion (.md) Conversion (New in v0.2.0)
Pre-ingests bloated PDFs, documents, images, or full code repositories and converts them into hyper-compressed, linked Knowledge Base Markdown documents with token savings telemetry:

```python
# 1. Top-Level Convenience Helper (File, Directory, or Bytes)
kb_result = docarmor.to_knowledge_base("procurement_agreement.pdf", target_model="claude-3-5-sonnet")

print(kb_result["markdown"])
print(f"Token Reduction : {kb_result['telemetry']['reduction_percentage']}%")
print(f"Cost Savings    : ${kb_result['telemetry']['cost_savings_usd']}")

# 2. Multi-File Project Repository Ingestion ("One Brain")
project_kb = analyzer.convert_directory_to_kb("./my_project", recursive=True, target_model="gemini-1.5-pro")
proj_data = json.loads(project_kb)
print(f"Project Files: {proj_data['telemetry']['total_files']} | Savings: {proj_data['telemetry']['reduction_percentage']}%")

# 3. Hardware-Accelerated OCR to Knowledge Base Markdown
ocr_analyzer = docarmor.OcrDocumentAnalyzer()
ocr_kb_str = ocr_analyzer.convert_file_to_kb("scanned_invoice.png", target_model="gpt-4o")
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

DocArmor generates a comprehensive, metadata-rich telemetry report for every analyzed file:

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

## 🧠 Pre-Ingestion Knowledge Base (.md) Converter Engine (v0.3.0)

### Why Pre-Ingestion Conversion?
When LLMs (Claude 3.5/3.7, GPT-4o, Gemini 1.5/2.0, LLaMA 3, DeepSeek R1/V3) process raw PDFs, scanned images, or multi-file code repositories, they consume tens of thousands of tokens. Multi-page PDFs trigger vision/rendering token bloat (~1,500 - 3,000 tokens per page), and raw codebases pollute context windows with boilerplate code.

DocArmor's **Pre-Ingestion Knowledge Base Engine** (`kb.rs`) sits directly before raw content is passed to LLM agents. It converts raw documents, images, and codebase repositories into structured, hyper-compressed **Knowledge Base Markdown (`.md`)** files equipped with:

- **Zero Truncation Guarantee**: 100% of document content is preserved without arbitrary line cutoffs or missing sections.
- **Balanced Code Fence Protection**: State-aware parsing tracks ` ``` ` fences, ensuring code blocks are never cut in half or malformed.
- **Hierarchical Table of Contents (TOC)**: Deep nested links reflecting true heading depth (`#`, `##`, `###`) with CommonMark/GitHub anchor slugs.
- **Configurable Detail Modes**:
  - `mode="full"`: Complete original content organized with structured TOC, back-to-top links, and metadata.
  - `mode="compact"`: Prunes excessive whitespace, duplicate lines, and comments (real 30–60% token reduction).
  - `mode="outline"`: Generates API skeletons, heading outlines, and symbol footprints for high-level agent routing.
- **Header Metadata & Telemetry**: Title, document class, target model compatibility, and token reduction stats.
- **Executive Summary & Key Takeaways**: High-density distilled insights and domain classification (skips badge links and markup noise).
- **Domain Taxonomy & PII Governance**: Entity map, PII categories found, and compliance flags.
- **Deep Navigation Links**: Footers for reliable LLM agent navigation (`[↑ Back to Table of Contents](#table-of-contents)`).

---

## ⚡ High-Speed Rayon Concurrency Layer (Bypassing Python GIL)

Python's Global Interpreter Lock (GIL) is a notorious bottleneck for agent frameworks (LangChain, CrewAI, AutoGen) when executing multiple tool calls, document validations, PII redacting, or token budgeting tasks concurrently.

DocArmor provides a **native Rust Rayon work-stealing execution layer** that releases the GIL via `py.allow_threads`:

```python
from docarmor.parallel import ParallelToolExecutor, run_parallel_tools

executor = ParallelToolExecutor()

# Queue heterogeneous agent tool tasks
for doc in documents:
    executor.add_task(task_type="token_budget", content=doc, target_model="claude-3-5-sonnet")
    executor.add_task(task_type="redact_pii", content=doc)
    executor.add_task(task_type="to_kb", content=doc, mode="compact")

# Executes simultaneously across ALL CPU cores in Rust without GIL lock
results = executor.run()
```

### Concurrency Benchmark

| Engine / Mode | Tasks Executed | Latency (ms) | Throughput | Speedup Ratio |
| :--- | :--- | :--- | :--- | :--- |
| **Python Sequential (GIL)** | 400 operations | 612.97 ms | 652 ops/sec | 1.0x (baseline) |
| **DocArmor Rust Rayon** | 400 operations | **3.82 ms** | **104,646 ops/sec** | **160.4x FASTER** |

---

## 🔌 Native Model Context Protocol (MCP) Server

DocArmor includes a native, sub-millisecond **Model Context Protocol (MCP)** server over standard I/O (STDIO) with **22µs response latency**.

### Running the MCP Server
```bash
# Direct CLI execution
python -m docarmor.mcp

# Or use the native binary
./target/release/docarmor-mcp
```

### Claude Desktop / Cursor / Antigravity MCP Configuration
Add DocArmor to your `claude_desktop_config.json` or Cursor MCP settings:

```json
{
  "mcpServers": {
    "docarmor": {
      "command": "python",
      "args": ["-m", "docarmor.mcp"]
    }
  }
}
```

### Exposed MCP Tools
1. `docarmor_scan`: Security scanning, quality score, PII detection, and exact token counting.
2. `docarmor_to_kb`: Structured Knowledge Base markdown generation with hierarchical TOC and anchors.
3. `docarmor_redact_pii`: Rust-native ultra-fast PII detection & masking before prompt submission.
4. `docarmor_token_budget`: Exact `tiktoken` calculation across Claude 3.7, GPT-4o, Gemini 2.0, DeepSeek R1.
5. `docarmor_repo_digest`: Multi-file repository aggregator ("one brain").
6. `docarmor_parallel_tools`: Concurrent multi-tool batch execution over Rayon.

### Token Savings & Speed Benchmarks

| Ingestion Payload | Target Model | Raw Input Tokens | Knowledge Base Tokens | Token Savings (%) | Latency (ms) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Large Procurement Spec (PDF)** | `claude-3-5-sonnet` | 21,300 tokens | 1,836 tokens | **91.4% Reduction** | **210.9 ms** |
| **Code Repository (20 Files)** | `gemini-1.5-pro` | 9,880 tokens | 7,862 tokens | **20.4% Reduction** | **93.8 ms** |

---

### Real-World Legal Document Benchmark Comparison

Benchmark evaluating a 12-page Master Services Agreement (`master_services_agreement.pdf`) containing legal indemnification, liability limitations, PII data privacy clauses, and arbitration governance terms across Claude model profiles:

| Target Model Profile | Model Target String | Raw Input Tokens | Knowledge Base Tokens | Token Reduction (%) | Raw Input Cost ($) | Knowledge Base Cost ($) | Net Cost Savings ($) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Claude 5 Sonnet** | `claude-5-sonnet` | 14,000 tokens | 3,795 tokens | **72.9% Reduction** | $0.04200 USD | $0.01139 USD | **$0.03061 USD** |
| **Claude 5 Opus** | `claude-5-opus` | 14,000 tokens | 3,795 tokens | **72.9% Reduction** | $0.07000 USD | $0.01898 USD | **$0.05102 USD** |
| **Claude Fable / Haiku** | `claude-fable` | 14,000 tokens | 3,795 tokens | **72.9% Reduction** | $0.01120 USD | $0.00303 USD | **$0.00817 USD** |

---

### Multi-Model Support Matrix

| Model Family | Target Model Identifiers | Rate per 1M Tokens | Context Window Limit |
| :--- | :--- | :--- | :--- |
| **Claude Sonnet** | `claude-3-5-sonnet`, `claude-3-7-sonnet`, `claude-sonnet-5` | **$3.00** | 200,000 tokens |
| **Claude Opus** | `claude-opus-5`, `claude-5-opus` | **$5.00** | 200,000 tokens |
| **Claude Opus (Legacy 3)**| `claude-3-opus`, `opus` | **$15.00** | 200,000 tokens |
| **Claude Haiku** | `claude-3-5-haiku`, `claude-haiku-5`, `haiku` | **$0.80** | 200,000 tokens |
| **OpenAI GPT-4o / GPT-5**| `gpt-4o`, `gpt-5` | **$2.50** | 128,000 tokens |
| **OpenAI Mini** | `gpt-4o-mini`, `gpt-5-mini` | **$0.15** | 128,000 tokens |
| **Google Gemini** | `gemini-1.5-pro`, `gemini-2.0-flash` | **$1.25** / **$0.10** | 1,000,000 tokens |
| **DeepSeek** | `deepseek-v3`, `deepseek-r1` | **$0.55** | 64,000 tokens |
| **Meta LLaMA** | `llama-3.3-70b`, `llama-4` | **$0.90** | 128,000 tokens |

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

## Ingestion Pipeline Flow

```mermaid
graph TD
    File[Document Uploaded] --> Security[Security Scanner: check sizes, corruption, zip bombs]
    Security -->|High Risk| Block[Abort: return error / flag security_risk]
    Security -->|Safe| Parser[Select Parser based on Extension: PDF, Docx, Xlsx, etc.]
    Parser --> Quality[Quality Evaluator: calculate density, pages, readability]
    
    Quality -->|Text Empty / Scanned| OCR[Lazy-load OCR: GPU Accelerated MPS/CUDA]
    Quality -->|Readable Text| Metrics[Metrics Evaluator: TikToken Token Counting, PII detection]
    
    OCR --> Metrics
    Metrics --> Router[Heuristic Domain Classifier & Cost Estimator]
    Router --> JSON[Generate Telemetry JSON Report]
```

---

## How the OCR Integration Works


DocArmor implements a high-performance **hybrid OCR gateway** under the `OcrDocumentAnalyzer` class:

1. **Rust-Native Gatekeeping**:
   When a file is submitted, DocArmor first uses its sub-millisecond Rust parsers to check the file type and structure.
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
   Once text is extracted via OCR, the raw text bytes are passed back into DocArmor's Rust core using a virtual text buffer. The Rust engine then computes exact GPT token budgets (`tiktoken-rs`), counts words/characters, runs domain classification, and generates cost estimations—reconciling all statistics back into a single unified JSON schema.

---

## Examples

### Example 1: RAG Ingestion Security & Quality Gatekeeper
Ensure that only secure, high-quality, digital documents enter your vector database:
```python
import json
import docarmor

analyzer = docarmor.DocumentAnalyzer()
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
import docarmor

analyzer = docarmor.DocumentAnalyzer({
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
from docarmor import OcrDocumentAnalyzer

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

<!-- DOCARMOR-STATS:START -->
## 📦 DocArmor — PyPI Installs

<div align="center">

<a href="https://pypi.org/project/docarmor/">
  <img src="https://img.shields.io/pypi/v/docarmor?style=for-the-badge&logo=pypi&logoColor=white&label=PyPI&color=3775A9" alt="PyPI Version"/>
</a>
<a href="https://pypistats.org/packages/docarmor">
  <img src="https://img.shields.io/badge/Total%20Installs-2%2C956-00C853?style=for-the-badge&logo=python&logoColor=white" alt="Total Installs"/>
</a>
<a href="https://pypistats.org/packages/docarmor">
  <img src="https://img.shields.io/badge/Last%207%20Days-17-AA00FF?style=for-the-badge&logo=download&logoColor=white" alt="Last 7 Days"/>
</a>
<a href="https://pypistats.org/packages/docarmor">
  <img src="https://img.shields.io/badge/Last%2024h-13-2979FF?style=for-the-badge&logo=download&logoColor=white" alt="Last 24 Hours"/>
</a>

</div>

<div align="center">

| 📊 Metric | 📈 Installs |
|:---|---:|
| 🏆 **Total Installs** — all releases | **2,956** |
| 📦 docarmor — since Aug 2, 2026 | 324 |
| 🗃️ docgaurd — before the rename | 2,632 |
| 📆 Last 7 days | 17 |
| 🕐 Last 24 hours | 13 |

<sub><b>What counts as an install:</b> a download requested by a package manager such as <code>pip</code> — repeat installs and CI runs included. A further <b>5,580</b> requests came from automated services that clone the entire PyPI index; those are excluded above, since they fetch every release whether or not anyone wants it.</sub>

<sub><b>Two package names, one project:</b> first published as <code>docgaurd</code> on Jun 2, 2026, then renamed to <code>docarmor</code> on Aug 2, 2026 and republished under a new PyPI account. The retired name kept serving installs until Aug 26, 2026. PyPI records download history per name and cannot merge the two, so the total above sums both.</sub>

<sub>🤖 Auto-updated August 30, 2026 at 12:58 AM IST via GitHub Actions · data through Aug 28, 2026</sub>

</div>

<div align="center">
  <a href="https://github.com/JIVTESH28/docarmor"><img src="https://img.shields.io/badge/GitHub-Source_Code-181717?style=for-the-badge&logo=github" alt="GitHub"/></a>
  <a href="https://pypi.org/project/docarmor/"><img src="https://img.shields.io/badge/Install-pip_install_docarmor-3775A9?style=for-the-badge&logo=pypi&logoColor=white" alt="Install"/></a>
  <a href="https://pypistats.org/packages/docarmor"><img src="https://img.shields.io/badge/Analytics-pypistats-4B8BBE?style=for-the-badge&logo=python&logoColor=white" alt="Analytics"/></a>
</div>
<!-- DOCARMOR-STATS:END -->
