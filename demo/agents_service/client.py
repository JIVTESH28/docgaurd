"""A2A client: calls each agent service's Agent Card + JSON-RPC endpoint.

This is the "orchestrator" side of the protocol — it never imports the
agents' internals, only talks to them over HTTP, exactly as a client from
another team or another codebase would.
"""
import json
import uuid

import requests

GUARD_URL = "http://localhost:9101"
SUMMARIZER_URL = "http://localhost:9102"
QA_URL = "http://localhost:9103"


class AgentUnavailable(RuntimeError):
    pass


def fetch_agent_card(base_url: str, timeout: float = 1.5) -> dict:
    try:
        r = requests.get(f"{base_url}/.well-known/agent-card.json", timeout=timeout)
        r.raise_for_status()
        return r.json()
    except requests.RequestException as e:
        raise AgentUnavailable(f"{base_url} not reachable: {e}") from e


def send_message(base_url: str, payload: dict, timeout: float = 180.0) -> dict:
    """SendMessage per the A2A JSON-RPC binding. `payload` is JSON-encoded
    into a single TextPart; the returned Task's first artifact part is
    JSON-decoded and returned."""
    task_id = str(uuid.uuid4())
    request_body = {
        "jsonrpc": "2.0",
        "method": "SendMessage",
        "params": {
            "message": {
                "messageId": str(uuid.uuid4()),
                "taskId": task_id,
                "role": "ROLE_USER",
                "parts": [{"text": json.dumps(payload)}],
            }
        },
        "id": task_id,
    }
    try:
        r = requests.post(base_url + "/", json=request_body, timeout=timeout)
        r.raise_for_status()
    except requests.RequestException as e:
        raise AgentUnavailable(f"{base_url} request failed: {e}") from e

    body = r.json()
    if "error" in body:
        raise RuntimeError(f"{base_url}: {body['error']}")

    task = body["result"]
    if task["status"]["state"] != "TASK_STATE_COMPLETED":
        raise RuntimeError(f"{base_url} task failed: {task['artifacts'][0]['parts'][0]['text']}")

    return json.loads(task["artifacts"][0]["parts"][0]["text"])


def guard(content_b64: str, file_name: str, entities: list) -> dict:
    return send_message(GUARD_URL, {
        "content_b64": content_b64, "file_name": file_name, "entities": entities,
    })


def summarize(redacted_text: str, document_class: str, backend: str, model: str) -> dict:
    return send_message(SUMMARIZER_URL, {
        "redacted_text": redacted_text, "document_class": document_class,
        "backend": backend, "model": model,
    })


def qa(redacted_text: str, question: str, backend: str, model: str) -> dict:
    return send_message(QA_URL, {
        "redacted_text": redacted_text, "question": question,
        "backend": backend, "model": model,
    })


def all_agents_reachable() -> dict:
    """{"guard-agent": card_or_None, ...} — used for a preflight/status view."""
    status = {}
    for name, url in (("guard-agent", GUARD_URL), ("summarizer-agent", SUMMARIZER_URL), ("qa-agent", QA_URL)):
        try:
            status[name] = fetch_agent_card(url)
        except AgentUnavailable:
            status[name] = None
    return status
