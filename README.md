# DocArmor

**Parallel AI Ingestion & Tool Execution Layer for LLM Systems**

DocArmor is a high-performance, Rust-powered engine that **validates, secures, analyzes, and executes document workflows before they reach your LLM or agent system**.

It acts as a **pre-LLM firewall + execution runtime**, enabling safe, cost-efficient, and parallel AI pipelines.

---

## 🚀 Why DocArmor?

Modern AI systems fail at the first step:

* ❌ Sending raw, unvalidated data to LLMs
* ❌ Unknown token cost until API call
* ❌ No PII protection
* ❌ Sequential tool execution (slow agents)

DocArmor fixes this by introducing a **control layer before AI execution**.

> **Protect, optimize, and execute AI workflows before they reach your LLM.**

---

## 🧠 Core Architecture

```
Raw Input (PDFs, Docs, APIs)
        ↓
🛡️ Security Layer (validation, zip bomb protection)
        ↓
📄 Parsing Layer (multi-format extraction)
        ↓
📊 Analysis Layer (tokens, cost, quality, PII)
        ↓
🧠 Decision Layer (routing, chunking, agent hints)
        ↓
⚡ Execution Layer (parallel tools via Rayon)
        ↓
🔌 MCP Interface / LangChain Integration
        ↓
LLM / Agent System
```

---

## ⚙️ Key Capabilities

### 🛡️ 1. LLM Ingestion Firewall

* Zip bomb protection
* File validation & corruption detection
* Oversized document handling
* Safe ingestion pipeline

---

### 💰 2. Deterministic Token & Cost Control

* Exact token counting (tiktoken-rs)
* Pre-LLM cost estimation
* Embedding cost prediction

> Know your cost **before** calling any model.

---

### 🔒 3. PII Detection & Masking

* Emails, phone numbers, IPs, sensitive data
* Automatic redaction before LLM exposure

---

### 📂 4. Multi-Format Document Processing

* PDF, DOCX, XLSX, TXT, JSON, HTML
* OCR routing (only when required)

---

### ⚡ 5. True Parallel Execution Engine

* Rust + PyO3 (GIL released)
* Rayon work-stealing scheduler
* Multi-core batch processing

```python
analyzer.analyze_batch(documents)
```

✔ Real CPU parallelism
✔ High throughput ingestion

---

### 🔌 6. MCP Tool Runtime (Parallel Execution)

DocArmor exposes tools via **MCP (Model Context Protocol)** and enables:

* Parallel tool execution
* Batched operations
* Structured tool responses

#### Traditional Agent Flow:

```
tool → wait → tool → wait
```

#### DocArmor Flow:

```
tool graph → parallel execution → aggregated result
```

---

### 📚 7. Knowledge Base (KB) Generation

* Repository digestion
* Structured knowledge extraction
* File-level summaries & classification

```python
repo_digest(path)
```

✔ Converts raw code/docs → structured knowledge
✔ Ready for downstream AI systems

---

### 🧠 8. Intelligent Routing Layer

DocArmor provides:

* Document classification
* Quality scoring
* RAG readiness detection
* Chunking recommendations

---

### 📊 9. Telemetry & Observability

Every document produces structured output:

```json
{
  "token_count": 1240,
  "estimated_cost": 0.0021,
  "quality_score": 0.87,
  "rag_ready": true,
  "pii_detected": false,
  "processing_time": 0.12
}
```

---

## 🔗 Integrations

### LangChain

Use DocArmor as a preprocessing layer:

```text
Documents → DocArmor → LangChain → LLM
```

---

### MCP (Model Context Protocol)

Run as an MCP server:

```bash
python -m docarmor.mcp
```

Allows LLMs/agents to:

* call tools
* run parallel workflows
* process documents at scale

---

## ⚡ Parallel Tool Execution Example

```python
tasks = [
    {"type": "scan", "input": doc1},
    {"type": "redact_pii", "input": doc2},
    {"type": "token_budget", "input": doc3},
    {"type": "to_kb", "input": doc4}
]

results = analyzer.execute_parallel_tasks(tasks)
```

✔ Executes across multiple CPU cores
✔ Returns aggregated structured output

---

## 📈 Real-World Use Cases

### 1. RAG Pipeline Optimization

```
Docs → DocArmor → Vector DB
```

* Filter bad documents
* Reduce token usage
* Improve retrieval quality

---

### 2. AI SaaS Upload Pipeline

```
User Upload → DocArmor → LLM
```

* Block unsafe inputs
* Mask PII
* Estimate cost

---

### 3. Enterprise Document Processing

```
1000+ Docs → Parallel Processing → Clean Output
```

* High throughput
* Secure ingestion
* Structured analytics

---

## 🆚 What Makes DocArmor Different

| Feature            | Typical Systems | DocArmor         |
| ------------------ | --------------- | ---------------- |
| Token counting     | Approximate     | Exact            |
| Cost estimation    | After API call  | Before API call  |
| PII protection     | Optional        | Built-in         |
| Execution          | Sequential      | Parallel (Rayon) |
| Tool orchestration | Linear          | Parallel MCP     |
| Performance        | Python-bound    | Rust-powered     |

---

## 🧭 Positioning

DocArmor is NOT:

* ❌ a vector database
* ❌ a full RAG framework
* ❌ an LLM

DocArmor IS:

> **The control layer that ensures only safe, optimized, and structured data reaches your AI system.**

---

## 🚀 Installation

```bash
pip install docarmor
```

---

## 🔥 One-Line Summary

> **DocArmor protects, optimizes, and executes AI workflows before they reach your LLM.**

---

## 📌 Future Scope

* Advanced agent orchestration
* Async + distributed execution
* Deeper LangChain & LlamaIndex integrations
* Semantic retrieval extensions

---

## 👨‍💻 Author

Built with focus on:

* performance
* scalability
* real-world AI system needs

---

## ⭐ Final Note

DocArmor is designed for developers who want:

* control over LLM cost
* secure document pipelines
* high-performance processing
* scalable AI workflows

---

**Stop sending raw data to LLMs.
Start controlling your AI pipeline.**
