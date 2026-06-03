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

        # Test new single-metric APIs
        print("\nTesting new single-metric APIs...")
        words = analyzer.count_words_bytes(text_data, "supply.txt")
        tokens = analyzer.count_tokens_bytes(text_data, "supply.txt")
        chars = analyzer.count_chars_bytes(text_data, "supply.txt")
        print(f"Single-metric count results: words={words}, tokens={tokens}, chars={chars}")
        assert words == 70
        assert tokens == 81
        assert chars == 520

        with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as f:
            f.write(text_data)
            temp_path = f.name
        try:
            f_words = analyzer.count_words(temp_path)
            f_tokens = analyzer.count_tokens(temp_path)
            f_chars = analyzer.count_chars(temp_path)
            print(f"File single-metric count results: words={f_words}, tokens={f_tokens}, chars={f_chars}")
            assert f_words == 70
            assert f_tokens == 81
            assert f_chars == 520
        finally:
            os.unlink(temp_path)

        # Test PII Redaction API
        print("\nTesting PII redaction APIs...")
        pii_text = "Contact me at bob@example.com or call 123-456-7890. IP is 192.168.1.50."
        
        # Test full redaction
        redacted_all = analyzer.redact_pii(pii_text)
        print(f"Redacted all: {redacted_all}")
        assert "[EMAIL]" in redacted_all
        assert "[PHONE]" in redacted_all
        assert "[IP_ADDRESS]" in redacted_all
        assert "bob@example.com" not in redacted_all
        assert "123-456-7890" not in redacted_all
        assert "192.168.1.50" not in redacted_all

        # Test partial redaction (only email)
        redacted_email_only = analyzer.redact_pii(pii_text, ["email"])
        print(f"Redacted email only: {redacted_email_only}")
        assert "[EMAIL]" in redacted_email_only
        assert "[PHONE]" not in redacted_email_only
        assert "123-456-7890" in redacted_email_only

        # Test analysis output contains pii fields
        res_pii = json.loads(analyzer.analyze_bytes(pii_text.encode("utf-8"), "pii_doc.txt"))
        print("\nPII Document Analysis Result:")
        print(json.dumps(res_pii, indent=2))
        assert res_pii["contains_pii"] is True
        assert "email" in res_pii["pii_categories_found"]
        assert "phone" in res_pii["pii_categories_found"]
        assert "ip" in res_pii["pii_categories_found"]
        
        # Test batch summary contains pii_files count
        with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as f_pii:
            f_pii.write(pii_text.encode("utf-8"))
            f_pii_path = f_pii.name
        try:
            batch_pii_res = json.loads(analyzer.analyze_batch([f1_path, f_pii_path]))
            print("\nBatch PII Analysis Result Summary:")
            print(json.dumps(batch_pii_res["summary"], indent=2))
            assert batch_pii_res["summary"]["pii_files"] == 1
        finally:
            os.unlink(f_pii_path)

        print("\nAll integration tests passed successfully!")
    finally:
        os.unlink(f1_path)
        os.unlink(f2_path)

if __name__ == "__main__":
    test_basic()
