use schemars::{schema_for, JsonSchema};
use serde_json::Value;

/// Generates JSON schema from a Rust type
pub fn generate_schema<T: JsonSchema>() -> Value {
    let schema = schema_for!(T);
    serde_json::to_value(schema).unwrap_or(Value::Null)
}

/// Tool schema builder for manual schema creation
#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub type_: String,
    pub properties: serde_json::Map<String, Value>,
    pub required: Vec<String>,
}

impl ToolSchema {
    pub fn new() -> Self {
        Self {
            type_: "object".to_string(),
            properties: serde_json::Map::new(),
            required: Vec::new(),
        }
    }

    pub fn property(
        mut self,
        name: impl Into<String>,
        type_: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let mut prop = serde_json::Map::new();
        prop.insert("type".to_string(), Value::String(type_.into()));
        prop.insert("description".to_string(), Value::String(description.into()));

        self.properties.insert(name.into(), Value::Object(prop));
        self
    }

    pub fn required(mut self, name: impl Into<String>) -> Self {
        self.required.push(name.into());
        self
    }

    pub fn build(self) -> Value {
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), Value::String(self.type_));
        schema.insert("properties".to_string(), Value::Object(self.properties));
        schema.insert(
            "required".to_string(),
            Value::Array(self.required.into_iter().map(Value::String).collect()),
        );

        Value::Object(schema)
    }
}

impl Default for ToolSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct TestParams {
        name: String,
        age: u32,
    }

    #[test]
    fn test_generate_schema() {
        let schema = generate_schema::<TestParams>();
        assert!(schema.is_object());

        let obj = schema.as_object().unwrap();
        assert!(obj.contains_key("$schema"));
    }

    #[test]
    fn test_tool_schema_builder() {
        let schema = ToolSchema::new()
            .property("expression", "string", "Math expression to calculate")
            .required("expression")
            .build();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert_eq!(
            schema["required"],
            Value::Array(vec![Value::String("expression".to_string())])
        );
    }
}
