"""
AutoGen integration for DocArmor.

Provides automated registration of DocArmor tools with Microsoft AutoGen agents:
- ConversableAgent / UserProxyAgent
- AssistantAgent
"""

from typing import Any, Optional
from .universal import get_default_registry


def register_with_autogen(caller: Any, executor: Optional[Any] = None) -> None:
    """
    Register all DocArmor tools with AutoGen agents.
    
    Example:
        assistant = AssistantAgent(name="assistant", ...)
        user_proxy = UserProxyAgent(name="user_proxy", ...)
        register_with_autogen(caller=assistant, executor=user_proxy)
    """
    registry = get_default_registry()
    registry.register_with_autogen(caller, executor)
