#!/usr/bin/env python3
import json
import time
import os
import tempfile
import docarmor

def run_kb_benchmark():
    print("\n" + "=" * 80)
    print("DOCARMOR 0.2.0 PRE-INGESTION KNOWLEDGE BASE BENCHMARK")
    print("=" * 80)

    analyzer = docarmor.DocumentAnalyzer()

    # 1. Large Document / PDF Benchmark
    print("\n1. Benchmarking Large Document Knowledge Base Conversion...")
    large_doc = (
        "SECTION 1: FINANCIAL AND LEGAL PROCUREMENT GOVERNANCE CLAUSES\n"
        "This master agreement governs all procurement order logistics safety stock replenishment\n"
        "and inventory demand forecasting between entity A and entity B under state jurisdiction.\n"
        "All disputes subject to binding arbitration within 30 days of invoice issuance.\n"
        "Contact legal-compliance@docarmor.org or call 1800-555-0199 for notices.\n\n"
        "SECTION 2: TECHNICAL SPECIFICATIONS AND CLOUD INFRASTRUCTURE\n"
        "System architecture relies on high performance Rust engine with Rayon multithreading\n"
        "and PyO3 bindings releasing Python GIL for concurrent ingestion of documents.\n"
        "Vector embedding index storage is chunked semantically to minimize hallucination.\n\n"
    ) * 150  # ~50,000 characters

    start = time.perf_counter()
    kb_res_str = analyzer.convert_bytes_to_kb(large_doc.encode("utf-8"), "large_procurement_spec.pdf", "claude-3-5-sonnet")
    elapsed_ms = (time.perf_counter() - start) * 1000.0
    kb_res = json.loads(kb_res_str)

    telemetry = kb_res["telemetry"]
    print(f"   [Done in {elapsed_ms:.2f} ms]")
    print(f"   Raw Input Tokens : {telemetry['raw_tokens']:,}")
    print(f"   KB Output Tokens : {telemetry['kb_tokens']:,}")
    print(f"   Tokens Saved     : {telemetry['tokens_saved']:,}")
    print(f"   Token Reduction  : {telemetry['reduction_percentage']}%")
    print(f"   Cost Savings     : ${telemetry['cost_savings_usd']:.5f}")

    # 2. Multi-File Project Repository Ingestion ("One Brain")
    print("\n2. Benchmarking Multi-File Code Base Ingestion ('One Brain')...")
    with tempfile.TemporaryDirectory() as temp_dir:
        # Generate 25 source files across subdirectories
        os.makedirs(os.path.join(temp_dir, "src", "parsers"), exist_ok=True)
        os.makedirs(os.path.join(temp_dir, "src", "engine"), exist_ok=True)

        for i in range(10):
            with open(os.path.join(temp_dir, "src", "parsers", f"parser_{i}.rs"), "w", encoding="utf-8") as f:
                f.write(
                    f"pub fn parse_module_{i}(data: &[u8]) -> usize {{\n"
                    f"    // High performance Rust parser implementation {i}\n"
                    f"    let size = data.len();\n"
                    f"    size * 2\n"
                    f"}}\n"
                    f"pub struct ParserConfig{i} {{\n"
                    f"    pub buffer_capacity: usize,\n"
                    f"}}\n" * 10
                )

        for i in range(10):
            with open(os.path.join(temp_dir, "src", "engine", f"service_{i}.py"), "w", encoding="utf-8") as f:
                f.write(
                    f"class ServiceEngine{i}:\n"
                    f"    def __init__(self):\n"
                    f"        self.name = 'Engine_{i}'\n"
                    f"    def execute(self, payload: bytes):\n"
                    f"        return len(payload)\n" * 10
                )

        start_proj = time.perf_counter()
        proj_res_str = analyzer.convert_directory_to_kb(temp_dir, recursive=True, target_model="claude-3-5-sonnet")
        elapsed_proj_ms = (time.perf_counter() - start_proj) * 1000.0
        proj_res = json.loads(proj_res_str)

        p_telemetry = proj_res["telemetry"]
        print(f"   [Done in {elapsed_proj_ms:.2f} ms]")
        print(f"   Total Files      : {p_telemetry['total_files']}")
        print(f"   Raw Project Toks : {p_telemetry['raw_tokens']:,}")
        print(f"   KB Project Toks  : {p_telemetry['kb_tokens']:,}")
        print(f"   Token Reduction  : {p_telemetry['reduction_percentage']}%")

    print("\n" + "=" * 80)
    print("BENCHMARK SUMMARY")
    print("=" * 80)
    print(f"Single Document Processing Latency : {elapsed_ms:.2f} ms")
    print(f"Project Repository Latency         : {elapsed_proj_ms:.2f} ms")
    print(f"Raw Input Tokens                   : {telemetry['raw_tokens']:,}")
    print(f"Knowledge Base Output Tokens       : {telemetry['kb_tokens']:,}")
    print(f"Token Savings                      : {telemetry['reduction_percentage']}% Reduction")
    print("=" * 80 + "\n")

if __name__ == "__main__":
    run_kb_benchmark()
