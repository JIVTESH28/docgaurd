#!/usr/bin/env python3

import json
import time
import os
import tempfile
import psutil
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

BATCH_SIZE = 100  # Number of concurrent files to process
TEST_FILE = "README.md"

def load_test_files():
    # Create copies of our test file in a temp directory to simulate a large corpus
    temp_dir = tempfile.TemporaryDirectory()
    paths = []
    with open(TEST_FILE, "r", encoding="utf-8") as f:
        content = f.read()
        
    for i in range(BATCH_SIZE):
        temp_path = os.path.join(temp_dir.name, f"doc_{i}.md")
        with open(temp_path, "w", encoding="utf-8") as out:
            out.write(content)
        paths.append(temp_path)
        
    return temp_dir, paths


def run_doctok_concurrently(file_paths):
    """
    To run DocTok concurrently in Python, developers must manually set up 
    a ThreadPoolExecutor. However, because CPU-bound tasks are blocked by the 
    Python GIL, execution is serialized.
    """
    try:
        from token_calculator import count_file
    except ImportError:
        return "Not Installed"
        
    start = time.perf_counter()
    
    # Process files concurrently using Python threads
    with ThreadPoolExecutor() as executor:
        results = list(executor.map(count_file, file_paths))
        
    elapsed = time.perf_counter() - start
    return elapsed


def run_docarmor_concurrently(file_paths):
    """
    DocArmor handles concurrency natively inside Rust via Rayon.
    It releases Python's Global Interpreter Lock (GIL) and processes all files 
    concurrently across physical CPU cores.
    """
    from docarmor import DocumentAnalyzer
    analyzer = DocumentAnalyzer()
    
    start = time.perf_counter()
    
    # Process files natively in Rust parallel thread pool
    raw = analyzer.analyze_batch(file_paths)
    data = json.loads(raw)
    
    elapsed = time.perf_counter() - start
    return elapsed


def main():
    print("\n" + "=" * 80)
    print(f"CONCURRENCY & MULTI-THREADING BENCHMARK ({BATCH_SIZE} Files)")
    print("=" * 80)
    
    print(f"CPU Cores : {psutil.cpu_count(logical=True)}")
    print(f"Memory GB : {round(psutil.virtual_memory().total/1024**3,2)}")
    
    if not Path(TEST_FILE).exists():
        print(f"Missing: {TEST_FILE}. Run the script from the project root.")
        return
        
    print("\nGenerating batch corpus in temporary folder...")
    temp_dir, paths = load_test_files()
    print(f"Generated {len(paths)} files successfully.")

    # 1. Benchmark DocTok (Python Threads)
    print("\n1. Running DocTok (Python ThreadPoolExecutor)...")
    doctok_time = run_doctok_concurrently(paths)
    if isinstance(doctok_time, str):
        print("   DocTok not available for test.")
        doctok_time = 0.0
    else:
        print(f"   Completed in: {doctok_time:.4f} seconds.")

    # 2. Benchmark DocArmor (Rust Native Parallelism)
    print("\n2. Running DocArmor (Rust Native Rayon ThreadPool)...")
    docarmor_time = run_docarmor_concurrently(paths)
    print(f"   Completed in: {docarmor_time:.4f} seconds.")

    # 3. Calculate Speedup
    if doctok_time > 0 and docarmor_time > 0:
        speedup = doctok_time / docarmor_time
        print("\n" + "=" * 80)
        print("CONCURRENCY BENCHMARK RESULTS")
        print("=" * 80)
        print(f"DocTok Concurrency   : {doctok_time:.4f}s")
        print(f"DocArmor Concurrency : {docarmor_time:.4f}s")
        print(f"Relative Speedup     : {speedup:.2f}x faster with DocArmor")
        print("-" * 80)
        print("Architectural Insight:")
        print("DocArmor releases the Python GIL and utilizes a Rust work-stealing thread")
        print("pool (Rayon). While Python threads block on CPU tasks due to the GIL,")
        print("DocArmor achieves true multi-core hardware scaling.")
        print("=" * 80)
    
    # Cleanup temp directory
    temp_dir.cleanup()


if __name__ == "__main__":
    main()
