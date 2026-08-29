"""Guard agent: A2A service wrapping DocArmor extraction + PII redaction.

Run standalone:
    uvicorn agents_service.guard_agent:app --port 9101
"""
import base64
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from preguard.guard import run_guard

from .protocol import build_a2a_app, make_agent_card

PORT = 9101


def _handle(message: dict) -> str:
    payload = json.loads(message["parts"][0]["text"])
    raw = base64.b64decode(payload["content_b64"])
    result = run_guard(raw, payload["file_name"], payload.get("entities", ()))
    return json.dumps({
        "security_risk": result.security_risk,
        "contains_pii": result.contains_pii,
        "pii_categories_found": result.pii_categories_found,
        "document_class": result.document_class,
        "token_count": result.token_count,
        "redacted_token_count": result.redacted_token_count,
        "rag_ready": result.rag_ready,
        "raw_text": result.raw_text,
        "redacted_text": result.redacted_text,
        "ocr_used": result.ocr_used,
        "full_report": result.full_report,
    })


agent_card = make_agent_card(
    agent_id="guard-agent",
    name="DocArmor Guard Agent",
    description="Extracts document text and redacts PII before any other agent sees it.",
    url=f"http://localhost:{PORT}/",
)
app = build_a2a_app(agent_card, _handle)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=PORT)
