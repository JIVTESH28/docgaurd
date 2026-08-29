"""Summarizer agent: A2A service that summarizes already-redacted text via
a local LLM (Ollama / LM Studio).

Run standalone:
    uvicorn agents_service.summarizer_agent:app --port 9102
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from preguard.config import settings
from preguard.llm_backends import build_chat_model

from .protocol import build_a2a_app, make_agent_card

PORT = 9102


def _handle(message: dict) -> str:
    payload = json.loads(message["parts"][0]["text"])
    model = build_chat_model(payload["backend"], payload["model"])
    prompt = (
        f"{settings.system_prompt}\n\n"
        f"Document class: {payload['document_class']}\n\n"
        f"Document:\n{payload['redacted_text']}\n\n"
        "Summarize this document in 3-5 bullet points."
    )
    summary = model.invoke(prompt).content
    return json.dumps({"summary": summary})


agent_card = make_agent_card(
    agent_id="summarizer-agent",
    name="DocArmor Summarizer Agent",
    description="Summarizes redacted document text using a local LLM.",
    url=f"http://localhost:{PORT}/",
)
app = build_a2a_app(agent_card, _handle)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=PORT)
