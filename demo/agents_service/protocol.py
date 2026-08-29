"""Minimal, spec-faithful A2A (Agent2Agent) protocol primitives.

Implements just enough of https://a2a-protocol.org (Agent Card discovery +
JSON-RPC 2.0 `SendMessage`/`GetTask`) for each pipeline stage to run as an
independently deployable, network-reachable agent — rather than a plain
in-process function call. Synchronous only: no streaming/push-notification
transport, no auth. Each agent completes a task in a single request/response
and stores it in memory for `GetTask` lookups.
"""
import time
import uuid
from typing import Callable

from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel

TASK_STATE_COMPLETED = "TASK_STATE_COMPLETED"
TASK_STATE_FAILED = "TASK_STATE_FAILED"


class AgentCard(BaseModel):
    id: str
    name: str
    description: str
    provider: dict
    capabilities: dict
    interfaces: list[dict]


def make_agent_card(agent_id: str, name: str, description: str, url: str) -> AgentCard:
    return AgentCard(
        id=agent_id,
        name=name,
        description=description,
        provider={"organization": "DocArmor PreGuard demo"},
        capabilities={"streaming": False, "pushNotifications": False},
        interfaces=[{"url": url, "transport": "JSONRPC"}],
    )


def _text_part(text: str) -> dict:
    return {"text": text}


def build_task(task_id: str, result_text: str) -> dict:
    return {
        "id": task_id,
        "status": {"state": TASK_STATE_COMPLETED, "timestamp": time.time()},
        "artifacts": [{"parts": [_text_part(result_text)]}],
    }


def build_failed_task(task_id: str, error: str) -> dict:
    return {
        "id": task_id,
        "status": {"state": TASK_STATE_FAILED, "timestamp": time.time()},
        "artifacts": [{"parts": [_text_part(error)]}],
    }


def build_a2a_app(agent_card: AgentCard, handle_message: Callable[[dict], str]) -> FastAPI:
    """`handle_message` receives the first text Part's content (as a raw
    string, expected to be JSON-encoded by the caller) and returns the
    result text (also JSON-encoded) to place in the completed Task's
    artifact."""
    app = FastAPI(title=agent_card.name)
    tasks: dict[str, dict] = {}

    @app.get("/.well-known/agent-card.json")
    def get_agent_card():
        return agent_card.model_dump()

    @app.post("/")
    async def jsonrpc_endpoint(request: Request):
        body = await request.json()
        method = body.get("method")
        rpc_id = body.get("id")

        if method == "SendMessage":
            message = body["params"]["message"]
            task_id = message.get("taskId") or str(uuid.uuid4())
            try:
                output_text = handle_message(message)
                task = build_task(task_id, output_text)
            except Exception as e:  # noqa: BLE001 - surfaced to the caller as a failed task
                task = build_failed_task(task_id, str(e))
            tasks[task_id] = task
            return JSONResponse({"jsonrpc": "2.0", "result": task, "id": rpc_id})

        if method == "GetTask":
            task_id = body["params"]["id"]
            task = tasks.get(task_id)
            if task is None:
                return JSONResponse(
                    {"jsonrpc": "2.0", "error": {"code": -32001, "message": "task not found"}, "id": rpc_id},
                    status_code=404,
                )
            return JSONResponse({"jsonrpc": "2.0", "result": task, "id": rpc_id})

        return JSONResponse(
            {"jsonrpc": "2.0", "error": {"code": -32601, "message": f"unknown method {method}"}, "id": rpc_id},
            status_code=400,
        )

    return app
