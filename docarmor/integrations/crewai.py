"""
CrewAI integration for DocArmor.

Provides seamless conversion of DocArmor tools into CrewAI tools:
- BaseTool subclasses compatible with Agent(tools=[...])
- Multi-agent collaboration with native DocArmor security, KB, and PII protection
"""

from typing import Any, List, Optional
from .universal import get_default_registry


def get_crewai_tools() -> List[Any]:
    """
    Returns all DocArmor tools wrapped as CrewAI BaseTool instances.
    Compatible with Agent(tools=get_crewai_tools()).
    """
    registry = get_default_registry()
    return registry.to_crewai()


def get_crewai_tool(name: str) -> Optional[Any]:
    """
    Returns a specific DocArmor tool by name wrapped for CrewAI.
    """
    tool = get_default_registry().get_tool(name)
    if tool:
        return tool.as_crewai_tool()
    return None


to_crewai_tools = get_crewai_tools
