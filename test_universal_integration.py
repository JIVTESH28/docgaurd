"""
Test suite for DocArmor's Rust-backed Universal Agent Integration Layer.
Tests compatibility across OpenAI, Anthropic, MCP, LangChain, CrewAI, LlamaIndex, and AutoGen.
"""

import json
import docarmor
from docarmor.integrations import (
    get_tools,
    get_tool,
    register_tool,
    to_openai_tools,
    to_anthropic_tools,
    to_mcp_tools,
    to_langchain_tools,
    get_langchain_tools,
    get_langchain_tool,
    to_crewai_tools,
    get_crewai_tools,
    get_crewai_tool,
    to_llamaindex_tools,
    get_llamaindex_tools,
    get_llamaindex_tool,
    register_with_autogen,
    UniversalToolAdapter,
)


def test_registry_and_tool_discovery():
    print("\n========================================================")
    print("1. Testing Universal Tool Discovery & Registry")
    print("========================================================")

    tools = get_tools()
    tool_names = [t.name for t in tools]
    print(f"Registered universal tools ({len(tools)}): {tool_names}")

    expected_tools = [
        "docarmor_scan",
        "docarmor_to_kb",
        "docarmor_redact_pii",
        "docarmor_token_budget",
        "docarmor_repo_digest",
        "docarmor_parallel_tools",
    ]

    for expected in expected_tools:
        assert expected in tool_names, f"Missing expected tool: {expected}"
        tool = get_tool(expected)
        assert tool is not None, f"get_tool('{expected}') returned None"
        assert tool.name == expected
        assert len(tool.description) > 0
        assert "properties" in tool.parameters

    print("✓ Tool registry and discovery passed!")


def test_schema_exports():
    print("\n========================================================")
    print("2. Testing Multi-Framework Schema Exporters")
    print("========================================================")

    # 1. OpenAI format
    openai_tools = to_openai_tools()
    assert len(openai_tools) >= 6
    for t in openai_tools:
        assert t["type"] == "function"
        fn = t["function"]
        assert "name" in fn and isinstance(fn["name"], str)
        assert "description" in fn and isinstance(fn["description"], str)
        assert "parameters" in fn and isinstance(fn["parameters"], dict)
        assert fn["parameters"].get("type") == "object"
    print(f"✓ Exported {len(openai_tools)} valid OpenAI function calling schemas.")

    # 2. Anthropic format
    anthropic_tools = to_anthropic_tools()
    assert len(anthropic_tools) >= 6
    for t in anthropic_tools:
        assert "name" in t and isinstance(t["name"], str)
        assert "description" in t and isinstance(t["description"], str)
        assert "input_schema" in t and isinstance(t["input_schema"], dict)
        assert t["input_schema"].get("type") == "object"
    print(f"✓ Exported {len(anthropic_tools)} valid Anthropic tool use schemas.")

    # 3. MCP format
    mcp_tools = to_mcp_tools()
    assert len(mcp_tools) >= 6
    for t in mcp_tools:
        assert "name" in t and isinstance(t["name"], str)
        assert "description" in t and isinstance(t["description"], str)
        assert "inputSchema" in t and isinstance(t["inputSchema"], dict)
        assert t["inputSchema"].get("type") == "object"
    print(f"✓ Exported {len(mcp_tools)} valid Model Context Protocol (MCP) schemas.")


def test_direct_tool_execution():
    print("\n========================================================")
    print("3. Testing Direct Tool Execution via Universal Adapter")
    print("========================================================")

    # Test PII Redaction
    redact_tool = get_tool("docarmor_redact_pii")
    assert redact_tool is not None

    res1 = redact_tool.run(text="Confidential agent contact: agent007@mi6.gov.uk call +1-800-555-0199")
    print("PII Redaction run() output:", res1)
    assert res1.get("contains_pii") is True
    assert "[EMAIL]" in res1.get("redacted_text", "")

    # Test callable syntax __call__
    res2 = redact_tool(text="Another email john.doe@example.com")
    assert "[EMAIL]" in res2.get("redacted_text", "")

    # Test Token Budget
    budget_tool = get_tool("docarmor_token_budget")
    assert budget_tool is not None
    budget_res = budget_tool(
        text="Analyzing complex multi-agent execution pathways across distributed nodes.",
        target_model="claude-3-5-sonnet",
    )
    print("Token Budget output:", budget_res)
    assert "token_count" in budget_res
    assert budget_res["token_count"] > 0
    assert "estimated_cost_usd" in budget_res
    assert "rate_per_million_usd" in budget_res

    # Test KB Generation
    kb_tool = get_tool("docarmor_to_kb")
    assert kb_tool is not None
    kb_res = kb_tool(content="# System Architecture\nCore engine handles routing.\n## Subsystem\nActive.", mode="compact")
    assert "markdown" in kb_res
    assert "# Knowledge Base:" in kb_res["markdown"]
    assert "telemetry" in kb_res

    print("✓ Direct tool execution passed across multiple operations!")


def test_langchain_compatibility():
    print("\n========================================================")
    print("4. Testing LangChain / LangGraph Compatibility")
    print("========================================================")

    lc_tools = to_langchain_tools()
    assert len(lc_tools) >= 6

    # Verify each tool satisfies LangChain tool contract
    for tool in lc_tools:
        assert hasattr(tool, "name")
        assert hasattr(tool, "description")
        assert hasattr(tool, "args_schema") or hasattr(tool, "args")
        assert hasattr(tool, "run")
        assert hasattr(tool, "invoke")
        assert hasattr(tool, "_run")

    scan_tool = get_langchain_tool("docarmor_scan")
    assert scan_tool is not None

    # Test invoke interface (LangChain standard)
    invoke_res = scan_tool.invoke({"content": "Safe agent instructions without vulnerabilities."})
    assert invoke_res.get("security_risk") in ["low", "medium", "high"]
    assert "quality_score" in invoke_res

    print("✓ LangChain protocol compatibility verified successfully!")


def test_crewai_compatibility():
    print("\n========================================================")
    print("5. Testing CrewAI Compatibility")
    print("========================================================")

    crew_tools = to_crewai_tools()
    assert len(crew_tools) >= 6

    # Verify each tool satisfies CrewAI BaseTool contract
    for tool in crew_tools:
        assert hasattr(tool, "name")
        assert hasattr(tool, "description")
        assert hasattr(tool, "_run")
        assert hasattr(tool, "run")

    redact_tool = get_crewai_tool("docarmor_redact_pii")
    assert redact_tool is not None

    # Test _run interface (CrewAI BaseTool standard)
    crew_res = redact_tool._run(text="User phone: 123-456-7890")
    assert crew_res.get("contains_pii") is True
    assert "[PHONE]" in crew_res.get("redacted_text", "")

    print("✓ CrewAI BaseTool protocol verified successfully!")


def test_llamaindex_compatibility():
    print("\n========================================================")
    print("6. Testing LlamaIndex Compatibility")
    print("========================================================")

    llama_tools = to_llamaindex_tools()
    assert len(llama_tools) >= 6

    for tool in llama_tools:
        assert hasattr(tool, "metadata")
        assert hasattr(tool.metadata, "name")
        assert hasattr(tool.metadata, "description")
        assert hasattr(tool, "call") or hasattr(tool, "__call__")

    budget_tool = get_llamaindex_tool("docarmor_token_budget")
    assert budget_tool is not None

    call_res = budget_tool.call(text="LlamaIndex index query document sample.")
    assert "token_count" in call_res
    assert call_res["token_count"] > 0

    print("✓ LlamaIndex FunctionTool protocol verified successfully!")


def test_autogen_compatibility():
    print("\n========================================================")
    print("7. Testing AutoGen Compatibility")
    print("========================================================")

    # Create mock AutoGen ConversableAgent
    class MockAutoGenAgent:
        def __init__(self, name="assistant"):
            self.name = name
            self.registered_tools = {}

        def register_for_llm(self, name, description):
            def decorator(fn):
                self.registered_tools[name] = {"type": "llm", "description": description, "fn": fn}
                return fn
            return decorator

        def register_for_execution(self, name):
            def decorator(fn):
                self.registered_tools[name] = {"type": "execution", "fn": fn}
                return fn
            return decorator

    agent = MockAutoGenAgent()
    register_with_autogen(agent)

    print(f"Registered {len(agent.registered_tools)} tools with AutoGen agent.")
    assert len(agent.registered_tools) >= 6
    assert "docarmor_scan" in agent.registered_tools
    assert "docarmor_to_kb" in agent.registered_tools

    # Test calling registered tool function
    registered_fn = agent.registered_tools["docarmor_scan"]["fn"]
    out = registered_fn(content="Test document for AutoGen agent execution.")
    assert out.get("security_risk") in ["low", "medium", "high"]

    print("✓ AutoGen integration verified successfully!")


def test_dynamic_custom_tool_registration():
    print("\n========================================================")
    print("8. Testing Dynamic Custom Tool Registration")
    print("========================================================")

    def custom_evaluator(query: str, threshold: float = 0.8) -> dict:
        return {
            "query": query,
            "threshold": threshold,
            "approved": len(query) > 5,
            "score": 0.95,
        }

    schema = {
        "type": "object",
        "properties": {
            "query": {"type": "string", "description": "Evaluation query"},
            "threshold": {"type": "number", "description": "Approval threshold"},
        },
        "required": ["query"],
    }

    custom_tool = register_tool(
        name="custom_evaluator",
        description="Evaluates agent prompt safety and criteria threshold.",
        parameters=schema,
        handler=custom_evaluator,
    )

    assert custom_tool.name == "custom_evaluator"

    # Verify presence in registry
    assert "custom_evaluator" in [t.name for t in get_tools()]

    # Verify export in OpenAI
    openai_schemas = to_openai_tools()
    assert any(s["function"]["name"] == "custom_evaluator" for s in openai_schemas)

    # Verify execution
    eval_res = custom_tool.run(query="Is this agent safe?", threshold=0.85)
    print("Custom tool execution result:", eval_res)
    assert eval_res["approved"] is True
    assert eval_res["score"] == 0.95

    print("✓ Dynamic custom tool registration and execution passed!")


def main():
    print("Starting DocArmor Universal Agent Integration Verification Suite...")
    test_registry_and_tool_discovery()
    test_schema_exports()
    test_direct_tool_execution()
    test_langchain_compatibility()
    test_crewai_compatibility()
    test_llamaindex_compatibility()
    test_autogen_compatibility()
    test_dynamic_custom_tool_registration()

    print("\n" + "=" * 56)
    print(" ALL UNIVERSAL INTEGRATION TESTS PASSED (8/8) ")
    print("=" * 56 + "\n")


if __name__ == "__main__":
    main()
