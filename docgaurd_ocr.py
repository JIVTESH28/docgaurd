import json
import torch
import docgaurd
from typing import List, Optional

class OcrDocumentAnalyzer:
    """
    Unified Document Intelligence Gateway with Hardware-Accelerated OCR.
    
    This class wraps docgaurd's high-speed Rust-native analyzer with PyTorch-powered
    OCR (EasyOCR) to handle both digital and scanned documents seamlessly.
    
    It automatically routes OCR workloads to:
      - Apple Silicon Metal (MPS) on macOS
      - Nvidia CUDA on GPU systems
      - Multi-core CPU fallback
    """
    
    def __init__(self, config: Optional[dict] = None, languages: List[str] = ['en']):
        # Initialize the native Rust DocumentAnalyzer with customizable configurations
        self.analyzer = docgaurd.DocumentAnalyzer(config)
        self.languages = languages
        self.ocr_reader = None
        
        # Determine the best hardware accelerator available
        if torch.backends.mps.is_available():
            self.device = "mps"      # macOS Metal Performance Shaders
        elif torch.cuda.is_available():
            self.device = "cuda"     # Nvidia GPU
        else:
            self.device = "cpu"      # Fallback CPU

    def _init_ocr(self):
        """Lazy load EasyOCR to keep initial startup time sub-millisecond for clean digital documents."""
        if self.ocr_reader is None:
            try:
                import easyocr
            except ImportError:
                raise ImportError(
                    "EasyOCR is required for OCR functionality. "
                    "Install it using: pip install easyocr torch torchvision"
                )
            
            # Initialize EasyOCR reader bound to the accelerated device
            self.ocr_reader = easyocr.Reader(self.languages, gpu=(self.device in ["cuda", "mps"]))

    def analyze_file(self, file_path: str, force_ocr: bool = False) -> str:
        """
        Analyzes a file on disk. If the file is scanned/image-only (requires OCR) 
        or if `force_ocr` is enabled, it automatically executes hardware-accelerated 
        OCR and merges the results back into a single unified telemetry schema.
        """
        # 1. Execute high-speed native DocGaurd validation
        raw_report = self.analyzer.analyze_file(file_path)
        report = json.loads(raw_report)
        
        # 2. Check if the document requires OCR
        needs_ocr = report.get("requires_ocr", False)
        
        if not needs_ocr and not force_ocr:
            # Document is already clean and readable; return the report immediately
            return json.dumps(report, indent=2)
            
        # 3. Boot OCR on the accelerated device (Metal, CUDA, or CPU)
        self._init_ocr()
        
        # 4. Extract text via OCR
        results = self.ocr_reader.readtext(file_path, detail=0)
        ocr_text = " ".join(results)
        ocr_bytes = ocr_text.encode('utf-8')
        
        # 5. Use DocGaurd's ultra-fast raw helpers to calculate precise metrics
        word_count = self.analyzer.count_words_bytes(ocr_bytes, file_path)
        token_count = self.analyzer.count_tokens_bytes(ocr_bytes, file_path)
        char_count = self.analyzer.count_chars_bytes(ocr_bytes, file_path)
        
        # 6. Re-run analysis on the OCR'd text bytes to get exact classification and recommendations
        bytes_report_str = self.analyzer.analyze_bytes(ocr_bytes, file_path)
        bytes_report = json.loads(bytes_report_str)
        
        # 7. Merge the OCR results and Rust-native telemetry into a single unified JSON schema
        report["text"] = ocr_text
        report["word_count"] = word_count
        report["token_count"] = token_count
        report["character_count"] = char_count
        report["requires_ocr"] = False
        report["rag_ready"] = (report.get("security_risk") == "low" and token_count > 20)
        
        # Update RAG recommendations, classifier domains, chunking strategies and cost estimation
        report["document_class"] = bytes_report.get("document_class")
        report["recommended_agent"] = bytes_report.get("recommended_agent")
        report["recommended_chunking"] = bytes_report.get("recommended_chunking")
        report["estimated_embedding_cost"] = bytes_report.get("estimated_embedding_cost")
        report["estimated_llm_cost"] = bytes_report.get("estimated_llm_cost")
        report["fits_context"] = bytes_report.get("fits_context")
        
        return json.dumps(report, indent=2)


if __name__ == "__main__":
    # Self-testing/demo interface
    import tempfile
    import os
    
    print("Initializing OcrDocumentAnalyzer...")
    gateway = OcrDocumentAnalyzer()
    print(f"Unified Gateway running on: {gateway.device.upper()}")
    
    # Create a dummy scanned document simulation
    with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as f:
        # A file with low character count but pages is detected as requiring OCR
        f.write(b"Scanned Doc Sim Content")
        temp_path = f.name
        
    try:
        print("\nProcessing Simulated Document...")
        report_json = gateway.analyze_file(temp_path)
        print(report_json)
    finally:
        os.unlink(temp_path)
