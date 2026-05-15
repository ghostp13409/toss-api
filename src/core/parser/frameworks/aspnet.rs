use crate::cli::args::Method;
use crate::core::collection::{Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody};
use crate::core::parser::models::{FieldType, Model, ModelField, ModelRegistry};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct AspNetParser;

impl AspNetParser {
    fn parse_cs_type(type_str: &str) -> FieldType {
        let type_str = type_str.trim().trim_end_matches('?');
        if type_str.is_empty() {
            return FieldType::Unknown;
        }

        match type_str.to_lowercase().as_str() {
            "string" | "guid" | "datetime" => FieldType::String,
            "int" | "long" | "float" | "double" | "decimal" | "short" | "byte" => FieldType::Number,
            "bool" => FieldType::Boolean,
            t if t.contains("List<") || t.contains("IEnumerable<") || t.contains("[]") => {
                FieldType::Array(Box::new(FieldType::Unknown))
            }
            _ => FieldType::Object(type_str.to_string()),
        }
    }
}

impl SourceParser for AspNetParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (ASP.NET)",
            project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));

        collection.env_vars.push(KVParam {
            key: "baseUrl".to_string(),
            value: "http://localhost:5000".to_string(),
            enabled: true,
            description: Some("Base URL for the service".to_string()),
        });

        let mut registry = ModelRegistry::new();

        // Pass 1: Model Discovery (Classes, Records, Structs)
        let class_regex = Regex::new(r"(?m)^(?:public|internal)?\s+(?:class|record|struct)\s+([a-zA-Z0-9_]+)").unwrap();
        let prop_regex = Regex::new(r"(?m)^\s+public\s+([a-zA-Z0-9_<>\[\]\?]+)\s+([a-zA-Z0-9_]+)\s*\{\s*get;").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "cs"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let lines: Vec<&str> = content.lines().collect();
                let mut i = 0;
                while i < lines.len() {
                    if let Some(cap) = class_regex.captures(lines[i]) {
                        let name = cap[1].to_string();
                        let mut fields = Vec::new();
                        i += 1;
                        let mut brace_count = if lines[i-1].contains('{') { 1 } else { 0 };
                        while i < lines.len() {
                            if lines[i].contains('{') { brace_count += 1; }
                            if lines[i].contains('}') { brace_count -= 1; }
                            
                            if let Some(pcap) = prop_regex.captures(lines[i]) {
                                fields.push(ModelField {
                                    name: pcap[2].to_string(),
                                    field_type: Self::parse_cs_type(&pcap[1]),
                                });
                            }
                            
                            if brace_count == 0 && lines[i].contains('}') {
                                break;
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
        // Attributes like [HttpGet("path")] or [HttpPost("path")]
        let attr_regex = Regex::new(r#"\[Http(Get|Post|Put|Patch|Delete)\s*(?:\(\s*["']([^"']*)["']\s*\))?\]"#).unwrap();
        // Minimal APIs like app.MapGet("path", ...)
        let map_regex = Regex::new(r#"app\.Map(Get|Post|Put|Patch|Delete)\s*\(\s*["']([^'"]+)['"]"#).unwrap();
        
        let from_body_regex = Regex::new(r"\[FromBody\]\s*([a-zA-Z0-9_<>]+)\s+([a-zA-Z0-9_]+)").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "cs"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let mut requests = Vec::new();

                let find_body = |pos: usize| -> RequestBody {
                    let slice_end = std::cmp::min(content.len(), pos + 500);
                    if let Some(bcap) = from_body_regex.captures(&content[pos..slice_end]) {
                        let type_name = &bcap[1];
                        if let Some(json_body) = registry.generate_json(type_name) {
                            return RequestBody::Raw {
                                content: json_body,
                                content_type: "application/json".to_string(),
                            };
                        }
                    }
                    RequestBody::None
                };

                for cap in attr_regex.captures_iter(&content) {
                    let method_str = &cap[1];
                    let url_path = cap.get(2).map_or("", |m| m.as_str());

                    let method = match method_str.to_lowercase().as_str() {
                        "post" => Method::Post,
                        "put" => Method::Put,
                        "patch" => Method::Patch,
                        "delete" => Method::Delete,
                        _ => Method::Get,
                    };

                    let body = find_body(cap.get(0).unwrap().end());

                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("{} {}", method_str.to_uppercase(), url_path),
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

                for cap in map_regex.captures_iter(&content) {
                    let method_str = &cap[1];
                    let url_path = &cap[2];

                    let method = match method_str.to_lowercase().as_str() {
                        "post" => Method::Post,
                        "put" => Method::Put,
                        "patch" => Method::Patch,
                        "delete" => Method::Delete,
                        _ => Method::Get,
                    };

                    let body = find_body(cap.get(0).unwrap().end());

                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("{} {}", method_str.to_uppercase(), url_path),
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
