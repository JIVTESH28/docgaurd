"""Lazy OCR fallback for scanned/image documents, via DocArmor's
`OcrDocumentAnalyzer` (EasyOCR under the hood, GPU-accelerated on
Metal/CUDA when available). Only touched when DocArmor's own analysis says
`requires_ocr`, or the upload is a raw image — plain digital PDFs/DOCX/etc.
never pay the EasyOCR startup cost.
"""
import json
import tempfile
from pathlib import Path

IMAGE_EXTENSIONS = {"png", "jpg", "jpeg", "tiff", "bmp", "webp", "gif"}

_ocr_analyzer = None


def _get_ocr_analyzer():
    global _ocr_analyzer
    if _ocr_analyzer is None:
        from docarmor.ocr import OcrDocumentAnalyzer
        _ocr_analyzer = OcrDocumentAnalyzer()
    return _ocr_analyzer


def is_image_file(file_name: str) -> bool:
    ext = file_name.rsplit(".", 1)[-1].lower() if "." in file_name else ""
    return ext in IMAGE_EXTENSIONS


def ocr_extract_text(raw: bytes, file_name: str) -> str:
    """Run DocArmor's OCR path on this file and return its extracted text."""
    suffix = Path(file_name).suffix or ".bin"
    with tempfile.NamedTemporaryFile(suffix=suffix) as tmp:
        tmp.write(raw)
        tmp.flush()
        report = json.loads(_get_ocr_analyzer().analyze_file(tmp.name, force_ocr=True))
    return report.get("text", "")
