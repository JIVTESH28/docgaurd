import json
from typing import List, Dict, Any, Optional, Union
from .docarmor import DocumentAnalyzer

class ParallelToolExecutor:
    """
    High-performance Rayon-powered parallel execution engine for agent tool calls,
    document inspections, PII scrubbing, and Knowledge Base generation.
    
    Bypasses the Python Global Interpreter Lock (GIL) by dispatching workloads
    directly into Rust's concurrent work-stealing thread pool.
    """
    
    def __init__(self, analyzer: Optional[DocumentAnalyzer] = None):
        self.analyzer = analyzer or DocumentAnalyzer()
        self._tasks: List[Dict[str, Any]] = []

    def add_task(
        self,
        task_type: str,
        file_path: Optional[str] = None,
        content: Optional[str] = None,
        file_name: Optional[str] = None,
        target_model: Optional[str] = None,
        mode: Optional[str] = None,
        entities: Optional[List[str]] = None,
        task_id: Optional[str] = None,
        recursive: Optional[bool] = None,
    ) -> "ParallelToolExecutor":
        """Queues a task for parallel execution."""
        task = {
            "id": task_id,
            "task_type": task_type,
            "file_path": file_path,
            "content": content,
            "file_name": file_name,
            "target_model": target_model,
            "mode": mode,
            "entities": entities,
            "recursive": recursive,
        }
        self._tasks.append(task)
        return self

    def clear(self) -> None:
        """Clears all queued tasks."""
        self._tasks.clear()

    def run(self) -> List[Dict[str, Any]]:
        """
        Executes all queued tasks in parallel across all CPU cores using Rust's Rayon.
        Releases the Python GIL during execution.
        """
        if not self._tasks:
            return []
        tasks_json = json.dumps(self._tasks)
        self._tasks.clear()
        res_str = self.analyzer.execute_parallel_tasks(tasks_json)
        return json.loads(res_str)

    def scan_files(self, file_paths: List[str]) -> List[Dict[str, Any]]:
        """Scans multiple files in parallel."""
        res_str = self.analyzer.parallel_scan(file_paths)
        return json.loads(res_str)

    def convert_files_to_kb(
        self,
        file_paths: List[str],
        target_model: str = "claude-3-5-sonnet",
        mode: str = "full"
    ) -> List[Dict[str, Any]]:
        """Converts multiple files to Knowledge Base markdown in parallel."""
        res_str = self.analyzer.parallel_convert_to_kb(file_paths, target_model, mode)
        return json.loads(res_str)

    def redact_pii_batch(
        self,
        texts: List[str],
        entities: Optional[List[str]] = None
    ) -> List[str]:
        """Redacts PII from a batch of strings in parallel."""
        return self.analyzer.parallel_redact_pii(texts, entities or [])

def run_parallel_tools(
    tasks: List[Dict[str, Any]],
    analyzer: Optional[DocumentAnalyzer] = None
) -> List[Dict[str, Any]]:
    """
    Convenience function: executes a list of task dicts simultaneously using Rayon.
    """
    executor = ParallelToolExecutor(analyzer)
    for t in tasks:
        executor.add_task(
            task_type=t["task_type"],
            file_path=t.get("file_path"),
            content=t.get("content"),
            file_name=t.get("file_name"),
            target_model=t.get("target_model"),
            mode=t.get("mode"),
            entities=t.get("entities"),
            task_id=t.get("id"),
            recursive=t.get("recursive"),
        )
    return executor.run()
