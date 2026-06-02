import json
import docgaurd

def test_basic():
    print("Testing docgaurd library...")
    analyzer = docgaurd.DocumentAnalyzer()
    
    text_data = b"This is a safety stock replenishment safety inventory demand planning SKU warehouse forecast logistics. " * 5
    res_str = analyzer.analyze_bytes(text_data, "supply.txt")
    res = json.loads(res_str)
    
    print("\nSingle Document Analysis Result:")
    print(json.dumps(res, indent=2))
    
    assert res["file_name"] == "supply.txt"
    assert res["file_type"] == "txt"
    assert res["document_class"] == "Supply Planning"
    assert res["recommended_agent"] == "SupplyPlanningAgent"
    assert res["security_risk"] == "low"
    assert res["fits_context"] is True
    assert res["rag_ready"] is True
    
    # Test duplicate detection via batch processing
    import tempfile
    import os
    
    with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as f1, \
         tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as f2:
        
        content = b"This is a legal contract agreement liability clause."
        f1.write(content)
        f1_path = f1.name
        
        f2.write(content)
        f2_path = f2.name
        
    try:
        batch_res_str = analyzer.analyze_batch([f1_path, f2_path])
        batch_res = json.loads(batch_res_str)
        print("\nBatch Analysis Result Summary:")
        print(json.dumps(batch_res["summary"], indent=2))
        
        results = batch_res["results"]
        assert len(results) == 2
        assert results[0]["duplicate"] is False
        assert results[1]["duplicate"] is True
        assert results[0]["document_class"] == "Legal"
        assert results[0]["recommended_agent"] == "LegalAgent"
        print("\nAll integration tests passed successfully!")
    finally:
        os.unlink(f1_path)
        os.unlink(f2_path)

if __name__ == "__main__":
    test_basic()
