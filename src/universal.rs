use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::batch::AnalysisConfig;
use super::mcp::{get_tools_list, handle_tool_call};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

impl UniversalToolDefinition {
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }

    pub fn to_openai(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters
            }
        })
    }

    pub fn to_anthropic(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "input_schema": self.parameters
        })
    }

    pub fn to_mcp(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.parameters
        })
    }

    pub fn to_universal(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters
        })
    }
}

#[derive(Debug, Clone)]
pub struct UniversalToolRegistry {
    tools: HashMap<String, UniversalToolDefinition>,
    order: Vec<String>,
}

impl Default for UniversalToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            order: Vec::new(),
        };
        registry.register_defaults();
        registry
    }

    fn register_defaults(&mut self) {
        let mcp_tools = get_tools_list();
        if let Some(tools_arr) = mcp_tools.as_array() {
            for tool_val in tools_arr {
                let name = tool_val["name"].as_str().unwrap_or("").to_string();
                let description = tool_val["description"].as_str().unwrap_or("").to_string();
                let parameters = tool_val["inputSchema"].clone();

                if !name.is_empty() {
                    self.register(UniversalToolDefinition::new(name, description, parameters));
                }
            }
        }
    }

    pub fn register(&mut self, tool: UniversalToolDefinition) {
        let name = tool.name.clone();
        if !self.tools.contains_key(&name) {
            self.order.push(name.clone());
        }
        self.tools.insert(name, tool);
    }

    pub fn get(&self, name: &str) -> Option<&UniversalToolDefinition> {
        self.tools.get(name)
    }

    pub fn list_names(&self) -> Vec<String> {
        self.order.clone()
    }

    pub fn export_schemas(&self, format: &str) -> Value {
        let mut list = Vec::new();
        for name in &self.order {
            if let Some(tool) = self.tools.get(name) {
                let exported = match format.to_lowercase().as_str() {
                    "openai" => tool.to_openai(),
                    "anthropic" => tool.to_anthropic(),
                    "mcp" => tool.to_mcp(),
                    _ => tool.to_universal(),
                };
                list.push(exported);
            }
        }
        Value::Array(list)
    }

    pub fn execute(&self, name: &str, args: &Value, config: &AnalysisConfig) -> Result<Value, String> {
        if self.tools.contains_key(name) {
            let (res, is_err) = handle_tool_call(name, args, config);
            if is_err {
                let err_msg = res.get("error").and_then(|v| v.as_str()).unwrap_or("Tool execution failed");
                Err(err_msg.to_string())
            } else {
                Ok(res)
            }
        } else {
            Err(format!("Tool '{}' is not registered in UniversalToolRegistry", name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_registry_defaults() {
        let registry = UniversalToolRegistry::new();
        let names = registry.list_names();
        assert!(names.contains(&"docarmor_scan".to_string()));
        assert!(names.contains(&"docarmor_to_kb".to_string()));
        assert!(names.contains(&"docarmor_redact_pii".to_string()));
        assert!(names.contains(&"docarmor_token_budget".to_string()));
        assert!(names.contains(&"docarmor_repo_digest".to_string()));
        assert!(names.contains(&"docarmor_parallel_tools".to_string()));
    }

    #[test]
    fn test_schema_exports() {
        let registry = UniversalToolRegistry::new();

        let openai = registry.export_schemas("openai");
        assert!(openai.is_array());
        let first_openai = &openai[0];
        assert_eq!(first_openai["type"], "function");
        assert!(first_openai["function"]["name"].is_string());
        assert!(first_openai["function"]["parameters"].is_object());

        let anthropic = registry.export_schemas("anthropic");
        assert!(anthropic.is_array());
        let first_anthropic = &anthropic[0];
        assert!(first_anthropic["name"].is_string());
        assert!(first_anthropic["input_schema"].is_object());

        let mcp = registry.export_schemas("mcp");
        assert!(mcp.is_array());
        let first_mcp = &mcp[0];
        assert!(first_mcp["name"].is_string());
        assert!(first_mcp["inputSchema"].is_object());
    }

    #[test]
    fn test_custom_tool_registration() {
        let mut registry = UniversalToolRegistry::new();
        let custom = UniversalToolDefinition::new(
            "custom_verifier",
            "Custom verification tool",
            json!({
                "type": "object",
                "properties": {
                    "token": { "type": "string" }
                },
                "required": ["token"]
            }),
        );
        registry.register(custom);
        assert!(registry.list_names().contains(&"custom_verifier".to_string()));

        let openai = registry.export_schemas("openai");
        let found = openai.as_array().unwrap().iter().any(|item| {
            item["function"]["name"] == "custom_verifier"
        });
        assert!(found);
    }
}
