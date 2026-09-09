"""
LangChain and LangGraph integration for DocArmor.

Provides seamless conversion of DocArmor tools into LangChain tools:
- StructuredTool / BaseTool instances
- Bindable to ChatOpenAI, ChatAnthropic, ChatGoogleGenerativeAI, etc.
- Compatible with LangGraph ToolNode and create_react_agent
"""

from typing import Any, List, Optional
from .universal import get_default_registry, UniversalToolAdapter


def get_langchain_tools() -> List[Any]:
    """
    Returns all DocArmor tools wrapped as LangChain tools.
    If langchain / langchain_core is installed, returns StructuredTool instances.
    Otherwise returns protocol-compliant duck-typed tools.
    """
    registry = get_default_registry()
    return registry.to_langchain()


def get_langchain_tool(name: str) -> Optional[Any]:
    """
    Returns a specific DocArmor tool by name wrapped for LangChain.
    """
    tool = get_default_registry().get_tool(name)
    if tool:
        return tool.as_langchain_tool()
    return None


to_langchain_tools = get_langchain_tools
