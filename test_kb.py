import json
import os
import tempfile
import docarmor

def test_kb_features():
    print("==================================================")
    print("Testing DocArmor 0.2.0 Pre-Ingestion KB Engine...")
    print("==================================================")
    
    analyzer = docarmor.DocumentAnalyzer()

    # 1. Test single byte document conversion
    sample_text = (
        "DocArmor Supply Chain Agreement\n"
        "Section 1: Executive Overview of Inventory and Logistics Safety Stock Replenishment.\n"
        "Section 2: Vendor Liability, Warranty, and Risk Governance Clauses.\n"
        "Contact support@docarmor.io or call 123-456-7890 for disputes.\n"
    ) * 10
    
    res = analyzer.convert_bytes_to_kb(sample_text.encode('utf-8'), "supply_chain.txt", "claude-3-5-sonnet")
    kb_data = json.loads(res) if isinstance(res, str) else res
    
    print("\n--- Single File KB Conversion Result ---")
    print(f"File Name      : {kb_data['telemetry']['file_name']}")
    print(f"Target Model   : {kb_data['telemetry']['target_model']}")
    print(f"Raw Tokens     : {kb_data['telemetry']['raw_tokens']}")
    print(f"KB Tokens      : {kb_data['telemetry']['kb_tokens']}")
    print(f"Tokens Saved   : {kb_data['telemetry']['tokens_saved']}")
    print(f"Reduction %    : {kb_data['telemetry']['reduction_percentage']}%")
    print(f"Cost Savings   : ${kb_data['telemetry']['cost_savings_usd']}")

    assert kb_data["telemetry"]["file_name"] == "supply_chain.txt"
    assert kb_data["telemetry"]["target_model"] == "claude-3-5-sonnet"
    assert kb_data["telemetry"]["contains_pii"] is True
    assert "email" in kb_data["telemetry"]["pii_categories_found"]
    assert "# Knowledge Base: supply_chain.txt" in kb_data["markdown"]
    assert "Table of Contents" in kb_data["markdown"]
    assert "[↑ Back to Table of Contents]" in kb_data["markdown"]

    # 2. Test top-level wrapper docarmor.convert_to_kb / to_knowledge_base
    with tempfile.NamedTemporaryFile(suffix=".md", delete=False, mode="w", encoding="utf-8") as f:
        f.write("# Project Spec\n\nSection 1: Architecture of neural gateway.\n\nSection 2: Security guardrails.")
        temp_file_path = f.name

    try:
        top_res = docarmor.convert_to_kb(temp_file_path, target_model="gpt-4o")
        print("\n--- Top-Level convert_to_kb Result (GPT-4o) ---")
        print(f"Target Model : {top_res['telemetry']['target_model']}")
        print(f"Raw Tokens   : {top_res['telemetry']['raw_tokens']}")
        print(f"KB Tokens    : {top_res['telemetry']['kb_tokens']}")
        assert top_res["telemetry"]["target_model"] == "gpt-4o"
        assert "Executive Summary" in top_res["markdown"]
    finally:
        os.unlink(temp_file_path)

    # 3. Test multi-file Project / Directory KB Conversion ("one brain")
    with tempfile.TemporaryDirectory() as temp_dir:
        f1_path = os.path.join(temp_dir, "main.rs")
        f2_path = os.path.join(temp_dir, "config.py")
        
        with open(f1_path, "w", encoding="utf-8") as f1:
            f1.write(
                "pub fn main() {\n"
                "    println!(\"Starting DocArmor Engine...\");\n"
                "}\n"
                "pub struct Config {\n"
                "    pub model: String,\n"
                "}\n"
            )
            
        with open(f2_path, "w", encoding="utf-8") as f2:
            f2.write(
                "class PipelineConfig:\n"
                "    def __init__(self):\n"
                "        self.target_model = 'gemini-1.5-pro'\n"
                "    def load(self):\n"
                "        pass\n"
            )

        proj_res_str = analyzer.convert_directory_to_kb(temp_dir, recursive=True, target_model="gemini-1.5-pro")
        proj_res = json.loads(proj_res_str)

        print("\n--- Project Directory KB Conversion Result (Gemini 1.5 Pro) ---")
        print(f"Total Files  : {proj_res['telemetry']['total_files']}")
        print(f"Raw Tokens   : {proj_res['telemetry']['raw_tokens']}")
        print(f"KB Tokens    : {proj_res['telemetry']['kb_tokens']}")
        print(f"Reduction %  : {proj_res['telemetry']['reduction_percentage']}%")

        assert proj_res["telemetry"]["total_files"] == 2
        assert "# Project Knowledge Base:" in proj_res["markdown"]
        assert "Project File Directory Tree & Module Index" in proj_res["markdown"]
        assert "[↑ Back to Project Index]" in proj_res["markdown"]

    print("\nAll Pre-Ingestion KB Engine integration tests passed successfully!")

if __name__ == "__main__":
    test_kb_features()
