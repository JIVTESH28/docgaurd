"""
DocArmor Universal Tool Adapter & Registry.

Provides native, dynamic integration between DocArmor's Rust-powered tool engine
and modern agent frameworks:
- OpenAI Function Calling
- Anthropic Tool Use
- Model Context Protocol (MCP)
- LangChain / LangGraph
- CrewAI
- LlamaIndex
- AutoGen

Designed for zero hardcoding and zero mandatory heavy dependencies:
All parameter schemas are generated natively from the Rust core and dynamically
translated into framework-native tool objects when installed, or into high-fidelity
duck-typed tool objects when running in lightweight environments.
"""

from __future__ import annotations
import inspect
import json
from typing import Any, Callable, Dict, List, Optional, Union

import docarmor


class UniversalToolAdapter:
    """
    Universal Tool representation capable of acting as a native tool in:
    - LangChain / LangGraph (implements .name, .description, .args_schema, .run, ._run, .invoke)
    - CrewAI (implements .name, .description, ._run, .run)
    - LlamaIndex (implements .metadata, .fn, .call)
    - AutoGen (supports register_for_llm, register_for_execution, register_function)
    - OpenAI Function Calling (to_openai())
    - Anthropic Tool Use (to_anthropic())
    - MCP Tool Protocol (to_mcp())
    """

    def __init__(
        self,
        name: str,
        description: str,
        parameters: Dict[str, Any],
        handler: Optional[Callable[..., Any]] = None,
    ):
        self.name = name
        self.description = description
        self.parameters = parameters
        self.args_schema = parameters
        self.args = parameters.get("properties", {})
        self._handler = handler or self._default_rust_handler

    def _default_rust_handler(self, **kwargs: Any) -> Any:
        analyzer = docarmor._get_default_analyzer()
        # Clean null values
        cleaned = {k: v for k, v in kwargs.items() if v is not None}
        args_json = json.dumps(cleaned)
        res_json = analyzer.execute_tool(self.name, args_json)
        return json.loads(res_json)

    def __call__(self, *args: Any, **kwargs: Any) -> Any:
        return self.run(*args, **kwargs)

    def run(self, *args: Any, **kwargs: Any) -> Any:
        """Executes the tool with the provided arguments."""
        if args and not kwargs:
            # Handle positional dict or single arg
            if len(args) == 1 and isinstance(args[0], dict):
                return self._handler(**args[0])
            elif len(args) == 1 and "path" in self.args:
                return self._handler(path=args[0])
            elif len(args) == 1 and "text" in self.args:
                return self._handler(text=args[0])
        return self._handler(**kwargs)

    def _run(self, *args: Any, **kwargs: Any) -> Any:
        """Compatibility method for LangChain BaseTool and CrewAI BaseTool."""
        return self.run(*args, **kwargs)

    def invoke(self, input_data: Union[Dict[str, Any], str], **kwargs: Any) -> Any:
        """Compatibility method for LangChain Runnable interface."""
        if isinstance(input_data, dict):
            return self.run(**input_data)
        elif isinstance(input_data, str):
            if "text" in self.args:
                return self.run(text=input_data)
            elif "path" in self.args:
                return self.run(path=input_data)
        return self.run(input=input_data, **kwargs)

    # -------------------------------------------------------------------------
    # Standard LLM Schemas
    # -------------------------------------------------------------------------

    def to_openai(self) -> Dict[str, Any]:
        """Export as OpenAI Function Calling schema."""
        return {
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters,
            },
        }

    def to_anthropic(self) -> Dict[str, Any]:
        """Export as Anthropic Tool Use schema."""
        return {
            "name": self.name,
            "description": self.description,
            "input_schema": self.parameters,
        }

    def to_mcp(self) -> Dict[str, Any]:
        """Export as Model Context Protocol (MCP) Tool specification."""
        return {
            "name": self.name,
            "description": self.description,
            "inputSchema": self.parameters,
        }

    def to_json_schema(self) -> Dict[str, Any]:
        """Export as standard OpenAPI/JSON Schema."""
        return {
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters,
        }

    # -------------------------------------------------------------------------
    # Framework Native Adapters
    # -------------------------------------------------------------------------

    def as_langchain_tool(self) -> Any:
        """
        Export as a first-class LangChain StructuredTool if langchain_core is installed,
        otherwise returns self (which duck-types LangChain BaseTool).
        """
        try:
            from langchain_core.tools import StructuredTool  # type: ignore

            def tool_fn(**kwargs: Any) -> Any:
                return self.run(**kwargs)

            tool_fn.__name__ = self.name
            tool_fn.__doc__ = self.description

            return StructuredTool.from_function(
                func=tool_fn,
                name=self.name,
                description=self.description,
                args_schema=None,
            )
        except ImportError:
            try:
                from langchain.tools import StructuredTool  # type: ignore

                def tool_fn(**kwargs: Any) -> Any:
                    return self.run(**kwargs)

                tool_fn.__name__ = self.name
                tool_fn.__doc__ = self.description

                return StructuredTool.from_function(
                    func=tool_fn,
                    name=self.name,
                    description=self.description,
                )
            except ImportError:
                return self

    def as_crewai_tool(self) -> Any:
        """
        Export as a first-class CrewAI BaseTool if crewai is installed,
        otherwise returns self (which duck-types CrewAI BaseTool).
        """
        try:
            from crewai.tools import BaseTool  # type: ignore

            class _CrewAITool(BaseTool):  # type: ignore
                name: str = self.name
                description: str = self.description

                def _run(self, *args: Any, **kwargs: Any) -> Any:
                    return self.run(*args, **kwargs)

            return _CrewAITool()
        except Exception:
            return self

    def as_llamaindex_tool(self) -> Any:
        """
        Export as a first-class LlamaIndex FunctionTool if llama_index is installed,
        otherwise returns self (which duck-types LlamaIndex BaseTool).
        """
        try:
            from llama_index.core.tools import FunctionTool  # type: ignore

            def tool_fn(**kwargs: Any) -> Any:
                return self.run(**kwargs)

            tool_fn.__name__ = self.name
            tool_fn.__doc__ = self.description

            return FunctionTool.from_defaults(
                fn=tool_fn,
                name=self.name,
                description=self.description,
            )
        except ImportError:
            # Add metadata attribute for duck-typing LlamaIndex tool
            class _LlamaIndexDuckTool:
                def __init__(inner_self, adapter: UniversalToolAdapter):
                    inner_self.adapter = adapter
                    inner_self.metadata = type(
                        "ToolMetadata",
                        (),
                        {
                            "name": adapter.name,
                            "description": adapter.description,
                            "get_parameters_dict": lambda: adapter.parameters,
                        },
                    )()

                def __call__(inner_self, *args: Any, **kwargs: Any) -> Any:
                    return inner_self.adapter.run(*args, **kwargs)

                def call(inner_self, *args: Any, **kwargs: Any) -> Any:
                    return inner_self.adapter.run(*args, **kwargs)

            return _LlamaIndexDuckTool(self)

    def register_with_autogen(self, caller: Any, executor: Optional[Any] = None) -> None:
        """
        Register this tool with Microsoft AutoGen ConversableAgent or AssistantAgent.
        Supports both modern autogen-agentchat and legacy autogen.
        """
        exec_agent = executor or caller

        # If register_for_llm and register_for_execution are available
        if hasattr(caller, "register_for_llm") and hasattr(exec_agent, "register_for_execution"):
            decorator_llm = caller.register_for_llm(name=self.name, description=self.description)
            decorator_exec = exec_agent.register_for_execution(name=self.name)
            decorated = decorator_llm(decorator_exec(self.run))
            return

        # AutoGen legacy function registration
        if hasattr(caller, "register_function"):
            caller.register_function(
                function_map={self.name: self.run},
            )


class UniversalToolRegistry:
    """
    Central Dynamic Tool Registry backed by DocArmor's Rust engine.
    """

    def __init__(self, analyzer: Optional[docarmor.DocumentAnalyzer] = None):
        self._analyzer = analyzer or docarmor._get_default_analyzer()
        self._custom_tools: Dict[str, UniversalToolAdapter] = {}

    def list_tools(self) -> List[str]:
        """Returns list of registered tool names."""
        names = list(self._analyzer.list_tools())
        for custom_name in self._custom_tools:
            if custom_name not in names:
                names.append(custom_name)
        return names

    def get_tool(self, name: str) -> Optional[UniversalToolAdapter]:
        """Retrieve a UniversalToolAdapter by name."""
        if name in self._custom_tools:
            return self._custom_tools[name]

        # Check Rust tools
        schemas = json.loads(self._analyzer.get_tool_definitions("universal"))
        for item in schemas:
            if item.get("name") == name:
                return UniversalToolAdapter(
                    name=item["name"],
                    description=item.get("description", ""),
                    parameters=item.get("parameters", {}),
                )
        return None

    def get_tools(self) -> List[UniversalToolAdapter]:
        """Retrieve all registered tools as UniversalToolAdapters."""
        tools: List[UniversalToolAdapter] = []
        schemas = json.loads(self._analyzer.get_tool_definitions("universal"))
        for item in schemas:
            name = item.get("name", "")
            if name in self._custom_tools:
                tools.append(self._custom_tools[name])
            else:
                tools.append(
                    UniversalToolAdapter(
                        name=name,
                        description=item.get("description", ""),
                        parameters=item.get("parameters", {}),
                    )
                )
        for custom_name, custom_tool in self._custom_tools.items():
            if not any(t.name == custom_name for t in tools):
                tools.append(custom_tool)
        return tools

    def register_tool(
        self,
        name: str,
        description: str,
        parameters: Dict[str, Any],
        handler: Optional[Callable[..., Any]] = None,
    ) -> UniversalToolAdapter:
        """
        Dynamically register a custom tool into DocArmor.
        Also registers its schema in Rust for universal export.
        """
        self._analyzer.register_tool(name, description, json.dumps(parameters))
        adapter = UniversalToolAdapter(
            name=name,
            description=description,
            parameters=parameters,
            handler=handler,
        )
        self._custom_tools[name] = adapter
        return adapter

    # -------------------------------------------------------------------------
    # Batch Framework Exporters
    # -------------------------------------------------------------------------

    def to_openai(self) -> List[Dict[str, Any]]:
        """Export all tools to OpenAI function calling specifications."""
        return [t.to_openai() for t in self.get_tools()]

    def to_anthropic(self) -> List[Dict[str, Any]]:
        """Export all tools to Anthropic tool use specifications."""
        return [t.to_anthropic() for t in self.get_tools()]

    def to_mcp(self) -> List[Dict[str, Any]]:
        """Export all tools to MCP tool specifications."""
        return [t.to_mcp() for t in self.get_tools()]

    def to_langchain(self) -> List[Any]:
        """Export all tools as LangChain tools."""
        return [t.as_langchain_tool() for t in self.get_tools()]

    def to_crewai(self) -> List[Any]:
        """Export all tools as CrewAI tools."""
        return [t.as_crewai_tool() for t in self.get_tools()]

    def to_llamaindex(self) -> List[Any]:
        """Export all tools as LlamaIndex tools."""
        return [t.as_llamaindex_tool() for t in self.get_tools()]

    def register_with_autogen(self, caller: Any, executor: Optional[Any] = None) -> None:
        """Register all tools with an AutoGen agent."""
        for t in self.get_tools():
            t.register_with_autogen(caller, executor)


_default_registry: Optional[UniversalToolRegistry] = None


def get_default_registry() -> UniversalToolRegistry:
    global _default_registry
    if _default_registry is None:
        _default_registry = UniversalToolRegistry()
    return _default_registry
