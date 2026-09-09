"""
LlamaIndex integration for DocArmor.

Provides seamless conversion of DocArmor tools into LlamaIndex FunctionTool instances:
- Compatible with ReActAgent, FunctionCallingAgent, and AgentRunner
"""

from typing import Any, List, Optional
from .universal import get_default_registry


def get_llamaindex_tools() -> List[Any]:
    """
    Returns all DocArmor tools wrapped as LlamaIndex FunctionTool instances.
    Compatible with ReActAgent.from_tools(get_llamaindex_tools()).
    """
    registry = get_default_registry()
    return registry.to_llamaindex()


def get_llamaindex_tool(name: str) -> Optional[Any]:
    """
    Returns a specific DocArmor tool by name wrapped for LlamaIndex.
    """
    tool = get_default_registry().get_tool(name)
    if tool:
        return tool.as_llamaindex_tool()
    return None


to_llamaindex_tools = get_llamaindex_tools
