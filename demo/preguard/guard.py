"""The PreGuard step: DocArmor-backed extraction, risk analysis and redaction.

This is the one part of the pipeline that never touches an LLM — it is
pure, deterministic, rule-based scanning so that raw PII is guaranteed to
never leave this layer.
"""
import json
from dataclasses import dataclass
from typing import Sequence

import docarmor

from .config import settings
from .extraction import extract_text
from .ocr_fallback import is_image_file, ocr_extract_text

_analyzer = docarmor.DocumentAnalyzer()


@dataclass
class GuardReport:
    file_name: str
    security_risk: str
    contains_pii: bool
    pii_categories_found: list
    document_class: str
    token_count: int
    redacted_token_count: int
    rag_ready: bool
    raw_text: str
    redacted_text: str
    ocr_used: bool
    full_report: dict


def run_guard(raw: bytes, file_name: str, entities: Sequence[str] = ()) -> GuardReport:
    """Extract, analyze and redact a document. Returns everything downstream
    stages need — but downstream stages must only ever forward `redacted_text`."""
    report = json.loads(_analyzer.analyze_bytes(raw, file_name))

    ocr_used = is_image_file(file_name) or report.get("requires_ocr", False)
    if ocr_used:
        text = ocr_extract_text(raw, file_name)
        # Re-score against the actual OCR'd text, under a plain .txt name —
        # analyze_bytes dispatches on file extension, and reusing the
        # original image/scan filename here makes it try to parse plain
        # text as e.g. PNG bytes and silently return "unsupported format".
        report = json.loads(_analyzer.analyze_bytes(text.encode("utf-8"), "ocr_text.txt"))
    else:
        text = extract_text(raw, file_name)

    redacted = _analyzer.redact_pii(text, list(entities) or list(settings.pii_entities))
    # `redacted` is always plain text regardless of the source format, so it
    # must be counted under a plain-text name — passing e.g. the original
    # "report.pdf"/"slides.pptx" name here would make analyze dispatch on
    # that extension, try to parse plain text as PDF/PPTX bytes, and
    # silently report 0 tokens (only .txt survives that by accident).
    redacted_token_count = _analyzer.count_tokens_bytes(redacted.encode("utf-8"), "redacted.txt")

    return GuardReport(
        file_name=file_name,
        security_risk=report["security_risk"],
        contains_pii=report["contains_pii"],
        pii_categories_found=report["pii_categories_found"],
        document_class=report["document_class"],
        token_count=report["token_count"],
        redacted_token_count=redacted_token_count,
        rag_ready=report["rag_ready"],
        raw_text=text,
        redacted_text=redacted,
        ocr_used=ocr_used,
        full_report=report,
    )
