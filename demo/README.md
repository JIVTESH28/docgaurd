# DocArmor PreGuard demo

A small interactive UI showing DocArmor acting as a PII pre-guard in front
of a local LLM: upload a PDF/PPTX/DOCX/TXT, DocArmor scans and redacts PII,
and only the redacted text is ever sent to the model.

## Setup

```bash
cd demo
python3 -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
```

Start a local LLM backend (either works, the app auto-detects both):

- **Ollama**: `ollama serve` (default `http://localhost:11434`), then
  `ollama pull llama3` (or any model) so it shows up in the dropdown.
- **LM Studio**: open LM Studio → Developer tab → Start Server
  (default `http://localhost:1234`).

## Run

```bash
streamlit run app.py
```

Open the printed local URL, upload a document, review the redaction
report, then ask the local model a question about the (redacted) document.
