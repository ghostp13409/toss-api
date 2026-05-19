use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array(Box<FieldType>),
    Object(String), // Reference to another model
    DateTime,
    Enum(Vec<String>),
    Map(Box<FieldType>, Box<FieldType>),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ModelField {
    pub name: String,
    pub field_type: FieldType,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub name: String,
    pub fields: Vec<ModelField>,
}

pub struct ModelRegistry {
    pub models: HashMap<String, Model>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.insert(model.name.clone(), model);
    }

    pub fn generate_json(&self, model_name: &str) -> Option<String> {
        self.generate_value(model_name, &mut Vec::new())
            .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
    }

    fn generate_value(&self, model_name: &str, stack: &mut Vec<String>) -> Option<Value> {
        // Prevent infinite recursion for self-referencing models
        if stack.contains(&model_name.to_string()) {
            return Some(json!(null));
        }
        stack.push(model_name.to_string());

        let model = self.models.get(model_name)?;
        let mut obj = serde_json::Map::new();

        for field in &model.fields {
            obj.insert(
                field.name.clone(),
                self.generate_field_value(&field.field_type, stack),
            );
        }

        stack.pop();
        Some(Value::Object(obj))
    }

    fn generate_field_value(&self, field_type: &FieldType, stack: &mut Vec<String>) -> Value {
        match field_type {
            FieldType::String => json!("string"),
            FieldType::Number => json!(0),
            FieldType::Boolean => json!(true),
            FieldType::DateTime => json!("2024-01-01T00:00:00Z"),
            FieldType::Enum(values) => {
                if let Some(first) = values.first() {
                    json!(first)
                } else {
                    json!("ENUM_VALUE")
                }
            }
            FieldType::Map(_, inner) => {
                let mut map = serde_json::Map::new();
                map.insert("key".to_string(), self.generate_field_value(inner, stack));
                Value::Object(map)
            }
            FieldType::Array(inner) => json!([self.generate_field_value(inner, stack)]),
            FieldType::Object(name) => self.generate_value(name, stack).unwrap_or(json!({})),
            FieldType::Unknown => json!(null),
        }
    }
}
