#!/usr/bin/env python3
"""Single entrypoint: `python live.py`.

Preflights the environment, launches the three A2A agent services
(guard-agent, summarizer-agent, qa-agent) plus the Streamlit orchestrator
UI, and tears everything down together on Ctrl-C.
"""
import subprocess
import sys
import time
import webbrowser
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent
DEMO_DIR = REPO_ROOT / "demo"
APP_PATH = DEMO_DIR / "ui" / "app.py"
UI_PORT = 8765

AGENT_SERVICES = [
    ("guard-agent", "agents_service.guard_agent:app", 9101),
    ("summarizer-agent", "agents_service.summarizer_agent:app", 9102),
    ("qa-agent", "agents_service.qa_agent:app", 9103),
]

# This repo's own source tree contains a `docarmor/` package (the library
# being built here) which would otherwise shadow the installed PyPI
# package of the same name. Strip the repo root from sys.path so imports
# below always resolve to the installed, compiled `docarmor`.
sys.path = [p for p in sys.path if Path(p or ".").resolve() != REPO_ROOT]


def preflight():
    print("== DocArmor PreGuard :: preflight ==")

    try:
        import docarmor  # noqa: F401
        print("[ok] docarmor importable")
    except ImportError:
        print("[FAIL] docarmor not installed. Run: pip install -r demo/requirements.txt")
        sys.exit(1)

    for pkg in ("streamlit", "fastapi", "uvicorn", "langchain_ollama", "langchain_openai"):
        try:
            __import__(pkg)
            print(f"[ok] {pkg} importable")
        except ImportError:
            print(f"[FAIL] {pkg} not installed. Run: pip install -r demo/requirements.txt")
            sys.exit(1)

    sys.path.insert(0, str(DEMO_DIR))
    from preguard.llm_backends import available_backends

    backends = available_backends()
    if not backends:
        print(
            "[warn] No local LLM backend detected.\n"
            "        Start one of:\n"
            "          - `ollama serve` (and `ollama pull <model>`)\n"
            "          - LM Studio -> Developer tab -> Start Server\n"
            "        The UI will still load and auto-detect once one is up."
        )
    else:
        for name, models in backends.items():
            print(f"[ok] {name} reachable, {len(models)} model(s) loaded")


def launch_agent_services() -> list[subprocess.Popen]:
    from agents_service.client import fetch_agent_card, AgentUnavailable

    print("\n== Starting A2A agent services ==")
    procs = []
    for name, module_target, port in AGENT_SERVICES:
        proc = subprocess.Popen(
            [sys.executable, "-m", "uvicorn", module_target, "--port", str(port)],
            cwd=str(DEMO_DIR),
        )
        procs.append(proc)

    for name, _module_target, port in AGENT_SERVICES:
        url = f"http://localhost:{port}"
        for attempt in range(30):
            try:
                fetch_agent_card(url, timeout=1.0)
                print(f"[ok] {name} up at {url}")
                break
            except AgentUnavailable:
                time.sleep(0.5)
        else:
            print(f"[FAIL] {name} did not come up at {url}")
            for p in procs:
                p.terminate()
            sys.exit(1)

    return procs


def launch_ui():
    url = f"http://localhost:{UI_PORT}"
    print(f"\n== Launching DocArmor PreGuard UI at {url} ==\n")
    return subprocess.Popen(
        [
            sys.executable, "-m", "streamlit", "run", str(APP_PATH),
            "--server.port", str(UI_PORT),
        ],
        # `python -m streamlit` puts the *current working directory* (not
        # the script's directory) at sys.path[0]. Running from repo root
        # would shadow the installed `docarmor` package with this repo's
        # own `docarmor/` source tree, so run from demo/ instead.
        cwd=str(DEMO_DIR),
    )


if __name__ == "__main__":
    preflight()
    agent_procs = launch_agent_services()
    ui_proc = launch_ui()
    webbrowser.open(f"http://localhost:{UI_PORT}")

    try:
        ui_proc.wait()
    except KeyboardInterrupt:
        pass
    finally:
        print("\nShutting down...")
        ui_proc.terminate()
        for p in agent_procs:
            p.terminate()
