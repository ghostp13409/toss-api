use crate::cli::args::Method;
use crate::core::collection::{Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody};
use crate::core::parser::models::{FieldType, Model, ModelField, ModelRegistry};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct GolangParser;

impl GolangParser {
    fn parse_go_type(type_str: &str) -> FieldType {
        let type_str = type_str.trim();
        if type_str.is_empty() {
            return FieldType::Unknown;
        }

        match type_str.to_lowercase().as_str() {
            "string" => FieldType::String,
            "int" | "int64" | "float32" | "float64" | "uint" => FieldType::Number,
            "bool" => FieldType::Boolean,
            t if t.starts_with("[]") => {
                FieldType::Array(Box::new(Self::parse_go_type(&t[2..])))
            }
            _ => FieldType::Object(type_str.to_string()),
        }
    }
}

impl SourceParser for GolangParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (Go)",
            project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));

        collection.env_vars.push(KVParam {
            key: "baseUrl".to_string(),
            value: "http://localhost:8080".to_string(),
            enabled: true,
            description: Some("Base URL for the service".to_string()),
        });

        let mut registry = ModelRegistry::new();

        // Pass 1: Model Discovery (Structs)
        let struct_regex = Regex::new(r"(?m)^type\s+([a-zA-Z0-9_]+)\s+struct\s*\{").unwrap();
        let field_regex = Regex::new(r#"^\s+([a-zA-Z0-9_]+)\s+([a-zA-Z0-9_\[\]\*]+)(?:\s+`json:"([a-zA-Z0-9_]+)".*`)?"#).unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "go"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let lines: Vec<&str> = content.lines().collect();
                let mut i = 0;
                while i < lines.len() {
                    if let Some(cap) = struct_regex.captures(lines[i]) {
                        let name = cap[1].to_string();
                        let mut fields = Vec::new();
                        i += 1;
                        while i < lines.len() && !lines[i].contains('}') {
                            if let Some(fcap) = field_regex.captures(lines[i]) {
                                let field_name = fcap.get(3).map_or(fcap[1].to_string(), |m| m.as_str().to_string());
                                fields.push(ModelField {
                                    name: field_name,
                                    field_type: Self::parse_go_type(&fcap[2].trim_start_matches('*')),
                                });
                            }
                            i += 1;
                        }
                        registry.add_model(Model {
                            name,
                            fields,
                        });
                    } else {
                        i += 1;
                    }
                }
            }
        }

        // Pass 2: Endpoint Extraction
        // router.GET("/path", ...) or http.HandleFunc("/path", ...)
        let route_regex = Regex::new(r#"\.(?:HandleFunc|GET|POST|PUT|PATCH|DELETE)\s*\(\s*['"]([^'"]+)['"]"#).unwrap();
        
        let bind_regex = Regex::new(r"(?:\.BindJSON|json\.Unmarshal|json\.NewDecoder).*?(&([a-zA-Z0-9_]+)\{\}|&([a-zA-Z0-9_]+))").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "go"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let mut requests = Vec::new();

                for cap in route_regex.captures_iter(&content) {
                    let url_path = &cap[1];
                    let method_match = Regex::new(r#"\.(GET|POST|PUT|PATCH|DELETE|HandleFunc)"#).unwrap().captures(cap.get(0).unwrap().as_str()).unwrap();
                    let method_str = &method_match[1];

                    let method = match method_str.to_uppercase().as_str() {
                        "POST" => Method::Post,
                        "PUT" => Method::Put,
                        "PATCH" => Method::Patch,
                        "DELETE" => Method::Delete,
                        _ => Method::Get,
                    };

                    let mut body = RequestBody::None;
                    
                    // Try to find binding pattern near the route
                    let pos = cap.get(0).unwrap().end();
                    let slice_end = std::cmp::min(content.len(), pos + 1000);
                    if let Some(bcap) = bind_regex.captures(&content[pos..slice_end]) {
                        let type_name = bcap.get(2).or(bcap.get(3)).map(|m| m.as_str()).unwrap_or("");
                        if let Some(json_body) = registry.generate_json(type_name) {
                            body = RequestBody::Raw {
                                content: json_body,
                                content_type: "application/json".to_string(),
                            };
                        }
                    }

                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("{} {}", if method_str == "HandleFunc" { "REQ" } else { method_str }, url_path),
                        method,
                        url: format!("{{{{baseUrl}}}}/{}", url_path.trim_start_matches('/')),
                        params: Vec::new(),
                        headers: Vec::new(),
                        auth: Auth::None,
                        body,
                        pre_request_script: None,
                        post_response_script: None,
                    }));
                }

                if !requests.is_empty() {
                    let file_name = entry.path().file_name().unwrap_or_default().to_string_lossy().to_string();
                    let mut folder = Folder::new(file_name);
                    folder.items = requests;
                    collection.items.push(CollectionItem::Folder(folder));
                }
            }
        }

        Ok(collection)
    }
}
