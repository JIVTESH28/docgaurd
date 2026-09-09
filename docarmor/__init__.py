import json
import os
from typing import Optional, Union, Dict, Any, List
from .docarmor import DocumentAnalyzer
from .ocr import OcrDocumentAnalyzer
from .parallel import ParallelToolExecutor, run_parallel_tools
from .mcp import run_mcp_server

_default_analyzer: Optional[DocumentAnalyzer] = None

def _get_default_analyzer() -> DocumentAnalyzer:
    global _default_analyzer
    if _default_analyzer is None:
        _default_analyzer = DocumentAnalyzer()
    return _default_analyzer

def convert_to_kb(
    target: Union[str, bytes],
    file_name: Optional[str] = None,
    target_model: str = "claude-3-5-sonnet",
    recursive: bool = True,
    mode: str = "full"
) -> Dict[str, Any]:
    """
    Converts a file path, raw bytes buffer, or project directory into a structured
    Knowledge Base Markdown document (.md) with hierarchical TOC and token savings telemetry.
    
    Args:
        target: Path to file (str), path to directory (str), or bytes buffer.
        file_name: Optional file name if target is bytes buffer.
        target_model: Target LLM model profile (default: "claude-3-5-sonnet").
        recursive: Whether to recursively process directories (default: True).
        mode: KB detail mode: 'full' (complete content), 'compact' (deduplicated), or 'outline' (headings/signatures).
        
    Returns:
        Dict containing "markdown" content string and "telemetry" metadata dict.
    """
    analyzer = _get_default_analyzer()
    if isinstance(target, bytes):
        name = file_name or "document.txt"
        res_str = analyzer.convert_bytes_to_kb(target, name, target_model, mode)
    elif isinstance(target, str):
        if os.path.isdir(target):
            res_str = analyzer.convert_directory_to_kb(target, recursive, target_model, mode)
        else:
            res_str = analyzer.convert_file_to_kb(target, target_model, mode)
    else:
        raise ValueError("Target must be a file path string, directory path string, or bytes buffer.")
    return json.loads(res_str)

to_knowledge_base = convert_to_kb

__all__ = [
    "DocumentAnalyzer",
    "OcrDocumentAnalyzer",
    "ParallelToolExecutor",
    "run_parallel_tools",
    "run_mcp_server",
    "convert_to_kb",
    "to_knowledge_base",
]
