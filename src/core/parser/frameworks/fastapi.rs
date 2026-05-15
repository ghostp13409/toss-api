use crate::cli::args::Method;
use crate::core::collection::{Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody};
use crate::core::parser::models::{FieldType, Model, ModelField, ModelRegistry};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct FastApiParser;

impl FastApiParser {
    fn parse_python_type(type_str: &str) -> FieldType {
        let type_str = type_str.trim();
        if type_str.is_empty() {
            return FieldType::Unknown;
        }

        match type_str.to_lowercase().as_str() {
            "str" => FieldType::String,
            "int" | "float" => FieldType::Number,
            "bool" => FieldType::Boolean,
            t if t.starts_with("list[") || t.starts_with("list") => {
                // Simplified list parsing
                FieldType::Array(Box::new(FieldType::Unknown))
            }
            _ => FieldType::Object(type_str.to_string()),
        }
    }
}

impl SourceParser for FastApiParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (FastAPI)",
            project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));

        collection.env_vars.push(KVParam {
            key: "baseUrl".to_string(),
            value: "http://localhost:8000".to_string(),
            enabled: true,
            description: Some("Base URL for the service".to_string()),
        });

        let mut registry = ModelRegistry::new();

        // Pass 1: Model Discovery
        let class_regex = Regex::new(r"(?m)^class\s+([a-zA-Z0-9_]+)(?:\s*\(.*\))?:").unwrap();
        let field_regex = Regex::new(r"^\s+([a-zA-Z0-9_]+)\s*:\s*([a-zA-Z0-9_\[\]]+)").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "py"))
        {
            let path_str = entry.path().to_string_lossy();
            if path_str.contains("venv") || path_str.contains("__pycache__") || path_str.contains(".git") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let lines: Vec<&str> = content.lines().collect();
                let mut i = 0;
                while i < lines.len() {
                    if let Some(cap) = class_regex.captures(lines[i]) {
                        let class_name = cap[1].to_string();
                        let mut fields = Vec::new();
                        i += 1;
                        while i < lines.len() && (lines[i].starts_with(' ') || lines[i].starts_with('\t') || lines[i].is_empty()) {
                            if let Some(fcap) = field_regex.captures(lines[i]) {
                                fields.push(ModelField {
                                    name: fcap[1].to_string(),
                                    field_type: Self::parse_python_type(&fcap[2]),
                                });
                            }
                            i += 1;
                        }
                        registry.add_model(Model {
                            name: class_name,
                            fields,
                        });
                    } else {
                        i += 1;
                    }
                }
            }
        }

        // Pass 2: Endpoint Extraction
        let route_regex =
            Regex::new(r#"@(?:app|router)\.(get|post|put|patch|delete)\s*\(\s*['"]([^'"]+)['"]"#)
                .unwrap();
        // Capture function signature to find typed parameters
        let func_regex = Regex::new(r"def\s+[a-zA-Z0-9_]+\s*\(([^)]*)\)").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "py"))
        {
            let path_str = entry.path().to_string_lossy();
            if path_str.contains("venv")
                || path_str.contains("__pycache__")
                || path_str.contains(".git")
            {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let mut requests = Vec::new();

                for cap in route_regex.captures_iter(&content) {
                    let method_str = &cap[1];
                    let url_path = &cap[2];

                    let method = match method_str.to_lowercase().as_str() {
                        "post" => Method::Post,
                        "put" => Method::Put,
                        "patch" => Method::Patch,
                        "delete" => Method::Delete,
                        _ => Method::Get,
                    };

                    let mut body = RequestBody::None;
                    
                    // Look for the function definition immediately following the decorator
                    let start_pos = cap.get(0).unwrap().end();
                    if let Some(func_cap) = func_regex.captures(&content[start_pos..]) {
                        let params = &func_cap[1];
                        for param in params.split(',') {
                            if let Some((_, type_hint)) = param.split_once(':') {
                                let type_hint = type_hint.trim();
                                if registry.models.contains_key(type_hint) {
                                    if let Some(json_body) = registry.generate_json(type_hint) {
                                        body = RequestBody::Raw {
                                            content: json_body,
                                            content_type: "application/json".to_string(),
                                        };
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("{} {}", method_str.to_uppercase(), url_path),
                        method,
                        url: format!("{{{{baseUrl}}}}{}", url_path),
                        params: Vec::new(),
                        headers: Vec::new(),
                        auth: Auth::None,
                        body,
                        pre_request_script: None,
                        post_response_script: None,
                    }));
                }

                if !requests.is_empty() {
                    let file_name = entry
                        .path()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let mut folder = Folder::new(file_name);
                    folder.items = requests;
                    collection.items.push(CollectionItem::Folder(folder));
                }
            }
        }

        Ok(collection)
    }
}
