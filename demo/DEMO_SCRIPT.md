# Demo script — DocArmor PreGuard

## 1. What is DocArmor? (30 sec)

"DocArmor is a Rust-native document intelligence and PII-redaction engine,
published on PyPI as `docarmor`. Give it a PDF, PPTX, DOCX, or TXT file and
it extracts the text, classifies the document, scores its quality and
security risk, estimates token count and LLM cost, and detects/redacts PII —
emails, phone numbers, SSNs, credit cards, IP addresses. It's compiled Rust
under a Python API, so it's fast, and it's a real published package
(`pip install docarmor`), not a script."

## 2. What is this project? (30 sec)

"This is DocArmor PreGuard — a gate that sits between any document and any
LLM. The core idea: an LLM — local or cloud — should never see a raw
document. Every document goes through DocArmor first: extract, classify,
redact. Only the redacted text goes downstream.

Architecture-wise it's three independent agent services — guard-agent,
summarizer-agent, qa-agent — each with its own A2A agent card and JSON-RPC
endpoint, talking over real HTTP, not just function calls. An orchestrator
UI calls them in sequence. `python live.py` boots all of it — three agent
services plus the UI — with one command."

## 3. The PII story (1 min) — live demo

1. Upload a doc with obvious PII in it (email, phone, SSN, card number).
2. Point at the **Guard report**: security risk, PII categories found,
   document class — this all happens before any LLM is involved.
3. Point at the **redacted text panel**: "this is the only thing that ever
   reaches an LLM. The original never leaves this layer."

## 4. The token story — with vs without (1–2 min) — the part you asked about

Two separate, honest claims — don't conflate them:

**a) Dollar cost (if you used a cloud model):**
"DocArmor estimates what this document would cost if sent to GPT-4 —
$X for Y tokens. Since we're running the LLM locally via Ollama/LM Studio,
that cost is $0. That's the chart in section 2."

**b) Token count — this is the one that matters even fully local:**
"Running locally doesn't mean tokens are free of *cost* — a local model
still has a fixed context window (4K–32K tokens depending on the model)
and still takes real time per token. Redacting PII before it hits the model
measurably shrinks the token count, because DocArmor replaces a long email
address or a 16-digit card number with a short tag like `[EMAIL]` or
`[CREDIT_CARD]`.

On a PII-dense paragraph I tested: **103 tokens before → 62 tokens after,
a 39.8% reduction.** That's the chart in section 2b, computed live for
whatever you upload — not a canned number."

**Why that matters even locally, concretely:**
- More of the model's fixed context window is left for the actual question,
  chat history, or retrieved context (RAG) instead of PII payload.
- Faster generation — local inference time scales with token count.
- The reduction is proportional to how PII-dense the document is; a
  contract full of names/emails/phone numbers shrinks a lot more than a
  clean technical spec.

## 5. Multi-agent pipeline (1 min) — live demo

1. Type a question, click **Run agents**.
2. Point at the **live console**: each line is a real `SendMessage` JSON-RPC
   call to a separate process (summarizer-agent, then qa-agent), with
   latency shown.
3. Point out: if the model is a reasoning model (e.g. DeepSeek-R1), its
   `<think>` trace is separated into a collapsible "reasoning trace" instead
   of cluttering the answer.

## 6. Closing line

"So: DocArmor is the redaction/analysis engine, published and pip-installable.
This project wraps it as a PreGuard layer with a real multi-agent, A2A-style
service architecture — and the numbers on screen aren't slides, they're
computed live off whatever document you hand it."

---

## Quick facts to have ready if asked

- **Package**: `pip install docarmor`, PyPI, currently v0.1.14.
- **PII types detected**: email, phone, SSN, IP address, credit card.
- **Supported inputs**: PDF, PPTX, DOCX, TXT, CSV, MD.
- **Local LLM backends supported**: Ollama and LM Studio, auto-detected.
- **Known limitation** (say it proactively, it lands well): phone-number
  regex is US-format-biased — an international number like
  `+91-98765-43210` slipped past redaction in testing. Good "next roadmap
  item" if asked about limitations.
- **What's simplified vs the full A2A spec**: synchronous only — no
  streaming transport, no push notifications, no auth between agents. Good
  answer if asked "is this production-ready": "this is a POC proving the
  architecture; hardening the transport is the next step."
