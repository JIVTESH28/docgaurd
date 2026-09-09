import json
import subprocess
import time
import os
import docarmor
from docarmor.parallel import ParallelToolExecutor, run_parallel_tools

def test_kb_engine_fidelity():
    print("\n========================================================")
    print("1. Testing Knowledge Base Engine (TOC & Markdown Fidelity)")
    print("========================================================")
    
    res = docarmor.convert_to_kb("README.md", mode="full")
    md = res["markdown"]
    telemetry = res["telemetry"]
    
    assert telemetry["file_name"] == "README.md"
    assert telemetry["mode"] == "full"
    
    lines = md.splitlines()
    print(f"Parsed README.md into {len(lines)} lines ({len(md)} chars).")
    
    # Assert no truncation: original README is ~500 lines, generated KB with TOC & metadata is > 650 lines
    assert len(lines) > 500, f"Expected > 500 lines, got {len(lines)} (truncation detected!)"
    
    # Verify TOC presence and nesting
    toc_entries = [l for l in lines if l.strip().startswith("- [") and "](#" in l]
    print(f"Detected {len(toc_entries)} TOC entries in Table of Contents.")
    assert len(toc_entries) > 20, "Table of Contents should capture all major sections without 8-section limit"
    
    # Verify code fence balance (no broken fences)
    code_fences = [l for l in lines if l.strip().startswith("```")]
    print(f"Detected {len(code_fences)} code block delimiters.")
    assert len(code_fences) % 2 == 0, f"Unbalanced code fences found! Count: {len(code_fences)}"
    
    # Verify outline mode
    outline_res = docarmor.convert_to_kb("README.md", mode="outline")
    outline_md = outline_res["markdown"]
    print(f"Outline mode generated {len(outline_md.splitlines())} lines ({len(outline_md)} chars).")
    assert len(outline_md) < len(md), "Outline mode should produce a significantly more compact document"
    
    print("✓ Knowledge Base TOC and Markdown fidelity tests passed!")

def test_rayon_parallel_concurrency():
    print("\n========================================================")
    print("2. Testing Rayon Parallel Tool Concurrency Layer")
    print("========================================================")
    
    executor = ParallelToolExecutor()
    num_tasks = 100
    
    for i in range(num_tasks):
        if i % 3 == 0:
            executor.add_task(
                task_type="token_budget",
                content=f"Parallel task index {i}: assessing token requirements for downstream agent execution.",
                target_model="claude-3-5-sonnet",
                task_id=f"budget-{i}"
            )
        elif i % 3 == 1:
            executor.add_task(
                task_type="redact_pii",
                content=f"Agent confidential contact: support_{i}@docarmor.io or call 800-555-01{i:02d}",
                task_id=f"pii-{i}"
            )
        else:
            executor.add_task(
                task_type="to_kb",
                content=f"# Module {i}\n\nSection 1: Agent logic.\n\nSection 2: Security guardrails.",
                file_name=f"module_{i}.txt",
                mode="compact",
                task_id=f"kb-{i}"
            )
            
    t0 = time.perf_counter()
    results = executor.run()
    elapsed = (time.perf_counter() - t0) * 1000.0
    
    print(f"Executed {len(results)} concurrent tasks via Rust Rayon in {elapsed:.2f} ms")
    print(f"Throughput: {len(results) / (elapsed / 1000.0):.1f} tasks/second")
    
    assert len(results) == num_tasks
    assert all(r["success"] for r in results), "All parallel tasks should succeed"
    
    # Test batch scan files
    scan_results = executor.scan_files(["Cargo.toml", "pyproject.toml", "README.md"])
    assert len(scan_results) == 3
    print(f"✓ Parallel scan of 3 files succeeded in {scan_results[0].get('processing_time_ms', 0):.2f} ms")
    
    print("✓ Rayon Parallel Concurrency tests passed!")

def test_mcp_server_protocol():
    print("\n========================================================")
    print("3. Testing Native Rust Model Context Protocol (MCP) Server")
    print("========================================================")
    
    proc = subprocess.Popen(
        ["/usr/bin/python3", "-m", "docarmor.mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    try:
        # 1. Initialize
        init_req = {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05"}}
        proc.stdin.write(json.dumps(init_req) + "\n")
        proc.stdin.flush()
        init_resp = json.loads(proc.stdout.readline())
        assert init_resp["result"]["serverInfo"]["name"] == "docarmor-mcp"
        print("✓ MCP 'initialize' handshake verified:", init_resp["result"]["serverInfo"])
        
        # 2. List tools
        list_req = {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}
        proc.stdin.write(json.dumps(list_req) + "\n")
        proc.stdin.flush()
        list_resp = json.loads(proc.stdout.readline())
        tool_names = [t["name"] for t in list_resp["result"]["tools"]]
        print(f"✓ MCP 'tools/list' exposed {len(tool_names)} tools: {tool_names}")
        assert "docarmor_scan" in tool_names
        assert "docarmor_to_kb" in tool_names
        assert "docarmor_redact_pii" in tool_names
        assert "docarmor_parallel_tools" in tool_names
        
        # 3. Call docarmor_scan tool
        scan_req = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "docarmor_scan",
                "arguments": {"path": "Cargo.toml"}
            }
        }
        proc.stdin.write(json.dumps(scan_req) + "\n")
        proc.stdin.flush()
        scan_resp = json.loads(proc.stdout.readline())
        scan_output = json.loads(scan_resp["result"]["content"][0]["text"])
        assert scan_output["file_name"] == "Cargo.toml"
        assert scan_output["security_risk"] in ("low", "none")
        print("✓ MCP 'docarmor_scan' tool call verified on Cargo.toml")
        
        # 4. Call docarmor_parallel_tools over MCP
        par_req = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "docarmor_parallel_tools",
                "arguments": {
                    "tasks": [
                        {"task_type": "token_budget", "content": "Short string."},
                        {"task_type": "redact_pii", "content": "email: test@mcp.io"}
                    ]
                }
            }
        }
        proc.stdin.write(json.dumps(par_req) + "\n")
        proc.stdin.flush()
        par_resp = json.loads(proc.stdout.readline())
        par_output = json.loads(par_resp["result"]["content"][0]["text"])
        assert par_output["total_executed"] == 2
        print("✓ MCP 'docarmor_parallel_tools' executed 2 tasks via Rayon over MCP")
        
    finally:
        proc.stdin.close()
        proc.terminate()
        
    print("✓ Model Context Protocol (MCP) server tests passed!")

if __name__ == "__main__":
    test_kb_engine_fidelity()
    test_rayon_parallel_concurrency()
    test_mcp_server_protocol()
    print("\n========================================================")
    print("ALL TESTS COMPLETED SUCCESSFULLY!")
    print("========================================================")
