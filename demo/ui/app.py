"""DocArmor PreGuard — Streamlit UI.

DocArmor is the primary layer here: every upload is extracted, scanned,
redacted, and costed out *before* anything optionally reaches a local LLM
(Ollama / LM Studio) for a secondary summarize/Q&A pass.
"""
import base64
import re
import sys
import time
from pathlib import Path

import altair as alt
import pandas as pd
import streamlit as st

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from agents_service import client as a2a
from preguard.config import settings
from preguard.llm_backends import available_backends

# Fixed categorical assignment (dataviz palette, slots 1 & 2 in order):
COLOR_CLOUD = "#2a78d6"   # blue  — first series encountered: cloud/no-guard baseline
COLOR_LOCAL = "#eb6834"   # orange — second series: PreGuard + local inference

THINK_RE = re.compile(r"<think>.*?</think>", re.DOTALL)


def split_reasoning(text: str) -> tuple[str, str]:
    """Split a reasoning model's <think>...</think> preamble from its answer."""
    reasoning = "".join(THINK_RE.findall(text))
    final = THINK_RE.sub("", text).strip()
    return final, reasoning.strip()


st.set_page_config(page_title="DocArmor PreGuard", page_icon="🛡️", layout="wide")
st.title("🛡️ DocArmor PreGuard")
st.caption(
    "The gate between your documents and any LLM: DocArmor extracts, "
    "classifies, costs out, and redacts PII first — an LLM never sees a "
    "raw document, local or otherwise."
)

if "processed_hashes" not in st.session_state:
    st.session_state.processed_hashes = set()
    st.session_state.docs_guarded = 0
    st.session_state.tokens_guarded = 0
    st.session_state.cloud_cost_avoided = 0.0

with st.sidebar:
    st.header("A2A agent services")
    agent_status = a2a.all_agents_reachable()
    if all(agent_status.values()):
        st.success("guard-agent / summarizer-agent / qa-agent all reachable")
    else:
        down = [name for name, card in agent_status.items() if card is None]
        st.error(
            f"Not reachable: {', '.join(down)}\n\n"
            "Start them (or use `python live.py` from repo root):\n"
            "```\ncd demo\n"
            "uvicorn agents_service.guard_agent:app --port 9101 &\n"
            "uvicorn agents_service.summarizer_agent:app --port 9102 &\n"
            "uvicorn agents_service.qa_agent:app --port 9103 &\n```"
        )
    with st.expander("Agent cards"):
        st.json({k: v for k, v in agent_status.items() if v})

    st.divider()
    st.header("Local LLM backend")
    backends = available_backends()

    backend = model = None
    if backends:
        for name, models in backends.items():
            st.success(f"{name} detected ({len(models)} model(s))")
        backend = st.selectbox("Backend", list(backends.keys()))
        models = backends[backend]
        model = st.selectbox("Model", models) if models else None
        if not models:
            st.info("No models loaded in that backend yet.")
    else:
        st.error(
            "No local LLM server found.\n\n"
            "Start one of:\n"
            "- `ollama serve` (and `ollama pull <model>`)\n"
            "- LM Studio -> Developer tab -> Start Server"
        )

    st.divider()
    st.header("PII categories to redact")
    selected_entities = [
        key for key in settings.pii_entities
        if st.checkbox(settings.pii_labels[key], value=True, key=f"ent_{key}")
    ]

    st.divider()
    st.header("This session")
    # Filled at the bottom of the script, after any upload this run has
    # already updated st.session_state — a plain call here would render
    # last run's totals, one interaction behind.
    session_stats_slot = st.container()

uploaded = st.file_uploader(
    "Upload a document", type=list(settings.supported_extensions)
)

if uploaded is not None:
    raw = uploaded.getvalue()

    with st.spinner("Calling guard-agent (A2A) to extract, classify, and redact PII..."):
        try:
            result = a2a.guard(base64.b64encode(raw).decode(), uploaded.name, selected_entities)
        except a2a.AgentUnavailable as e:
            st.error(f"guard-agent unreachable: {e}")
            st.stop()

    doc_hash = result["full_report"]["sha256"]
    if doc_hash not in st.session_state.processed_hashes:
        st.session_state.processed_hashes.add(doc_hash)
        st.session_state.docs_guarded += 1
        st.session_state.tokens_guarded += result["token_count"]
        st.session_state.cloud_cost_avoided += result["full_report"]["estimated_llm_cost"]

    st.subheader("1. Guard report")
    cols = st.columns(6)
    cols[0].metric("Security risk", result["security_risk"])
    cols[1].metric("Contains PII", "Yes" if result["contains_pii"] else "No")
    cols[2].metric("Document class", result["document_class"])
    cols[3].metric("Token count", result["token_count"])
    cols[4].metric("RAG ready", "Yes" if result["rag_ready"] else "No")
    cols[5].metric("OCR used", "Yes" if result.get("ocr_used") else "No")

    if result.get("ocr_used"):
        st.info("Scanned/image document detected — DocArmor ran local GPU-accelerated OCR (EasyOCR) before analysis.")

    if result["pii_categories_found"]:
        st.warning("PII categories detected: " + ", ".join(result["pii_categories_found"]))
    else:
        st.success("No PII detected in this document.")

    st.subheader("2. Cost & privacy impact of this layer")
    st.caption(
        "Baseline cost is DocArmor's own estimate for sending this document's "
        f"{result['token_count']:,} tokens to a cloud model (gpt-4 @ $5.00/M "
        "input tokens). PreGuard routes the redacted text to a local model "
        "instead — $0 API cost, and raw PII never leaves this machine."
    )

    cost_df = pd.DataFrame([
        {"path": "Without PreGuard (cloud LLM)", "cost": result["full_report"]["estimated_llm_cost"]},
        {"path": "With PreGuard (local LLM)", "cost": 0.0},
    ])
    chart = (
        alt.Chart(cost_df)
        .mark_bar(size=48, cornerRadiusTopLeft=4, cornerRadiusTopRight=4)
        .encode(
            x=alt.X("path:N", title=None, sort=None, axis=alt.Axis(labelAngle=0)),
            y=alt.Y("cost:Q", title="Estimated cost per run (USD)"),
            color=alt.Color(
                "path:N",
                scale=alt.Scale(domain=[
                    "Without PreGuard (cloud LLM)", "With PreGuard (local LLM)",
                ], range=[COLOR_CLOUD, COLOR_LOCAL]),
                legend=None,
            ),
        )
    )
    labels = chart.mark_text(dy=-10, fontWeight="bold").encode(
        text=alt.Text("cost:Q", format="$.4f")
    )
    st.altair_chart(chart + labels, width="stretch")

    st.subheader("2b. Token footprint on the local model")
    before_tok = result["token_count"]
    after_tok = result["redacted_token_count"]
    reduction_pct = (100 * (before_tok - after_tok) / before_tok) if before_tok else 0.0
    st.caption(
        "Even with $0 API cost, fewer tokens per request still matters locally: "
        "less to fit in the model's context window, faster generation, more "
        "room left for chat history / retrieved context in the same window."
    )
    tcols = st.columns(3)
    tcols[0].metric("Tokens before redaction", f"{before_tok:,}")
    tcols[1].metric("Tokens after redaction", f"{after_tok:,}")
    tcols[2].metric("Reduction", f"{reduction_pct:.1f}%")

    token_df = pd.DataFrame([
        {"path": "Without PreGuard (raw text)", "tokens": before_tok},
        {"path": "With PreGuard (redacted text)", "tokens": after_tok},
    ])
    token_chart = (
        alt.Chart(token_df)
        .mark_bar(size=48, cornerRadiusTopLeft=4, cornerRadiusTopRight=4)
        .encode(
            x=alt.X("path:N", title=None, sort=None, axis=alt.Axis(labelAngle=0)),
            y=alt.Y("tokens:Q", title="Tokens sent to the local model"),
            color=alt.Color(
                "path:N",
                scale=alt.Scale(domain=[
                    "Without PreGuard (raw text)", "With PreGuard (redacted text)",
                ], range=[COLOR_CLOUD, COLOR_LOCAL]),
                legend=None,
            ),
        )
    )
    token_labels = token_chart.mark_text(dy=-10, fontWeight="bold").encode(
        text=alt.Text("tokens:Q", format=",")
    )
    st.altair_chart(token_chart + token_labels, width="stretch")

    with st.expander("Full DocArmor report (JSON)"):
        st.json(result["full_report"])

    st.subheader("3. Redaction — what actually changes")
    col_a, col_b = st.columns(2)
    with col_a:
        st.markdown("**Original extracted text**")
        st.text_area("original", result["raw_text"], height=250, label_visibility="collapsed")
    with col_b:
        st.markdown("**Redacted text — this is what any downstream agent sees**")
        st.text_area("redacted", result["redacted_text"], height=250, label_visibility="collapsed")

    st.divider()
    st.subheader("4. Optional: run the A2A multi-agent pipeline")
    st.caption(
        "Secondary step — each call below is a real HTTP request to an "
        "independent agent process (summarizer-agent, qa-agent), discovered "
        "via its A2A agent card and invoked over JSON-RPC `SendMessage`."
    )
    question = st.text_input(
        "Optional question", placeholder="e.g. What risks does this document raise?"
    )

    agents_up = agent_status["summarizer-agent"] and agent_status["qa-agent"]
    if st.button("Run agents", disabled=backend is None or model is None or not agents_up):
        out_col, log_col = st.columns([3, 2])

        with out_col:
            st.markdown("**Summary**")
            summary_box = st.empty()
            summary_reasoning = st.container()
            st.markdown("**Answer**")
            answer_box = st.empty()
            answer_reasoning = st.container()

        with log_col:
            st.markdown("**🔴 Live agent console**")
            console_box = st.empty()

        log_lines = []

        def render_console():
            console_box.code("\n".join(log_lines) or "...", language="text")

        def log(line: str):
            log_lines.append(f"[{time.strftime('%H:%M:%S')}] {line}")
            render_console()

        try:
            log(f"model selected: {backend} / {model}")

            log("SendMessage -> summarizer-agent: started")
            t0 = time.monotonic()
            summary_resp = a2a.summarize(result["redacted_text"], result["document_class"], backend, model)
            elapsed_ms = (time.monotonic() - t0) * 1000
            final, reasoning = split_reasoning(summary_resp["summary"])
            summary_box.write(final)
            if reasoning:
                with summary_reasoning.expander("Model reasoning trace"):
                    st.text(reasoning)
            log(f"summarizer-agent: task completed ({elapsed_ms:.0f} ms)")

            if question:
                log("SendMessage -> qa-agent: started")
                t0 = time.monotonic()
                qa_resp = a2a.qa(result["redacted_text"], question, backend, model)
                elapsed_ms = (time.monotonic() - t0) * 1000
                final, reasoning = split_reasoning(qa_resp["answer"])
                answer_box.write(final)
                if reasoning:
                    with answer_reasoning.expander("Model reasoning trace"):
                        st.text(reasoning)
                log(f"qa-agent: task completed ({elapsed_ms:.0f} ms)")
            else:
                answer_box.write("_(no question asked — summary only)_")

            log("pipeline complete")
        except Exception as e:
            log(f"ERROR: {e}")
            st.error(f"Pipeline failed: {e}")
else:
    st.info("Upload a PDF, PPTX, DOCX, or TXT file to get started.")

with session_stats_slot:
    st.metric("Documents guarded", st.session_state.docs_guarded)
    st.metric("Tokens kept off the cloud", f"{st.session_state.tokens_guarded:,}")
    st.metric("Cloud-LLM cost avoided (est.)", f"${st.session_state.cloud_cost_avoided:,.4f}")
