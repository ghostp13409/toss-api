use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Environment {
    #[serde(default)]
    pub name: String,
    pub variables: HashMap<String, String>,
}

impl Environment {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&path)?;
        let env: Environment = if path.as_ref().extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.as_ref().extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::from_str(&content)?
        } else {
            serde_json::from_str(&content)?
        };
        Ok(env)
    }

    pub fn replace_vars(&self, input: &str) -> String {
        let mut output = input.to_string();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            output = output.replace(&placeholder, value);
        }
        output
    }
}
