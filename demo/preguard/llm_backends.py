"""Local LLM backend discovery and LangChain chat-model construction.

Supports two interchangeable local backends:
  - Ollama       (native API,        default http://localhost:11434)
  - LM Studio    (OpenAI-compatible, default http://localhost:1234/v1)
"""
import requests
from langchain_ollama import ChatOllama
from langchain_openai import ChatOpenAI

from .config import settings


def detect_ollama():
    """Return list of loaded model names, or None if the server isn't up."""
    try:
        r = requests.get(f"{settings.ollama_url}/api/tags", timeout=settings.detect_timeout_s)
        r.raise_for_status()
        return [m["name"] for m in r.json().get("models", [])]
    except requests.RequestException:
        return None


def detect_lmstudio():
    try:
        r = requests.get(f"{settings.lmstudio_url}/v1/models", timeout=settings.detect_timeout_s)
        r.raise_for_status()
        ids = [m["id"] for m in r.json().get("data", [])]
        # LM Studio lists loaded embedding models alongside chat models with
        # no field to tell them apart; embedding models 404 on chat
        # completions, so filter them out by the standard naming convention.
        return [m for m in ids if "embed" not in m.lower()]
    except requests.RequestException:
        return None


def available_backends() -> dict:
    """{"Ollama": [...models], "LM Studio": [...models]} for whichever are up."""
    backends = {}
    ollama_models = detect_ollama()
    if ollama_models is not None:
        backends["Ollama"] = ollama_models
    lmstudio_models = detect_lmstudio()
    if lmstudio_models is not None:
        backends["LM Studio"] = lmstudio_models
    return backends


def build_chat_model(backend: str, model: str, temperature: float = 0.2):
    if backend == "Ollama":
        return ChatOllama(base_url=settings.ollama_url, model=model, temperature=temperature)
    if backend == "LM Studio":
        return ChatOpenAI(
            base_url=f"{settings.lmstudio_url}/v1",
            api_key="lm-studio",  # LM Studio ignores the key but the client requires one
            model=model,
            temperature=temperature,
        )
    raise ValueError(f"Unknown backend: {backend}")
