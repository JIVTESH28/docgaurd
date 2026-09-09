"""
DocArmor Universal Agent Integrations.

Provides seamless, native adapters for all major LLM and Agent frameworks:
- OpenAI Function Calling
- Anthropic Tool Use
- Model Context Protocol (MCP)
- LangChain / LangGraph
- CrewAI
- LlamaIndex
- AutoGen
"""

from __future__ import annotations
from typing import Any, Callable, Dict, List, Optional, Union

from .universal import (
    UniversalToolAdapter,
    UniversalToolRegistry,
    get_default_registry,
)
from .langchain import get_langchain_tools, get_langchain_tool, to_langchain_tools
from .crewai import get_crewai_tools, get_crewai_tool, to_crewai_tools
from .llamaindex import get_llamaindex_tools, get_llamaindex_tool, to_llamaindex_tools
from .autogen import register_with_autogen


def get_tools() -> List[UniversalToolAdapter]:
    """Returns all registered tools as UniversalToolAdapters."""
    return get_default_registry().get_tools()


def get_tool(name: str) -> Optional[UniversalToolAdapter]:
    """Returns a specific tool by name."""
    return get_default_registry().get_tool(name)


def register_tool(
    name: str,
    description: str,
    parameters: dict,
    handler=None,
) -> UniversalToolAdapter:
    """Dynamically registers a custom tool into DocArmor's Rust registry."""
    return get_default_registry().register_tool(
        name=name,
        description=description,
        parameters=parameters,
        handler=handler,
    )


def to_openai_tools() -> List[Dict[str, Any]]:
    """Exports all tools to OpenAI function calling specifications."""
    return get_default_registry().to_openai()


def to_anthropic_tools() -> List[Dict[str, Any]]:
    """Exports all tools to Anthropic tool use specifications."""
    return get_default_registry().to_anthropic()


def to_mcp_tools() -> List[Dict[str, Any]]:
    """Exports all tools to Model Context Protocol (MCP) tool specifications."""
    return get_default_registry().to_mcp()


__all__ = [
    "UniversalToolAdapter",
    "UniversalToolRegistry",
    "get_default_registry",
    "get_tools",
    "get_tool",
    "register_tool",
    "to_openai_tools",
    "to_anthropic_tools",
    "to_mcp_tools",
    "to_langchain_tools",
    "get_langchain_tools",
    "get_langchain_tool",
    "to_crewai_tools",
    "get_crewai_tools",
    "get_crewai_tool",
    "to_llamaindex_tools",
    "get_llamaindex_tools",
    "get_llamaindex_tool",
    "register_with_autogen",
]
