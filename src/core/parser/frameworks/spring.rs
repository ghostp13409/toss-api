use crate::cli::args::Method;
use crate::core::collection::{Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody};
use crate::core::parser::models::{FieldType, Model, ModelField, ModelRegistry};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct SpringParser;

impl SpringParser {
    fn parse_java_type(type_str: &str) -> FieldType {
        let type_str = type_str.trim();
        if type_str.is_empty() {
            return FieldType::Unknown;
        }

        match type_str.to_lowercase().as_str() {
            "string" => FieldType::String,
            "int" | "integer" | "long" | "double" | "float" | "bigdecimal" => FieldType::Number,
            "boolean" | "bool" => FieldType::Boolean,
            t if t.contains("list") || t.contains("set") || t.contains("iterable") => {
                FieldType::Array(Box::new(FieldType::Unknown))
            }
            _ => FieldType::Object(type_str.to_string()),
        }
    }
}

impl SourceParser for SpringParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (Spring Boot)",
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

        // Pass 1: Model Discovery (DTOs and Entities)
        let class_regex = Regex::new(r"(?m)^(?:public\s+)?(?:class|record)\s+([a-zA-Z0-9_]+)").unwrap();
        let field_regex = Regex::new(r"^\s+(?:private|public|protected)?\s+([a-zA-Z0-9_<>\?]+)\s+([a-zA-Z0-9_]+)\s*(?:;|=)").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "java" || ext == "kt"))
        {
            let path_str = entry.path().to_string_lossy();
            if path_str.contains("target") || path_str.contains(".git") {
                continue;
            }

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
                            
                            if let Some(fcap) = field_regex.captures(lines[i]) {
                                fields.push(ModelField {
                                    name: fcap[2].to_string(),
                                    field_type: Self::parse_java_type(&fcap[1]),
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
        // Matches @GetMapping("/path"), @PostMapping("/path"), etc.
        let mapping_regex = Regex::new(
            r#"@(Get|Post|Put|Delete|Patch)Mapping\s*\(\s*(?:value\s*=\s*)?['"]([^'"]+)['"]"#,
        )
        .unwrap();

        // Matches @RequestMapping(value = "/path", method = RequestMethod.GET)
        let request_mapping_regex = Regex::new(
            r#"@RequestMapping\s*\(\s*(?:value\s*=\s*)?['"]([^'"]+)['"](?:.*method\s*=\s*RequestMethod\.(GET|POST|PUT|DELETE|PATCH))?"#,
        )
        .unwrap();

        let request_body_regex = Regex::new(r"@RequestBody\s+([a-zA-Z0-9_<>]+)\s+([a-zA-Z0-9_]+)").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "java" || ext == "kt"))
        {
            let path_str = entry.path().to_string_lossy();
            if path_str.contains("target") || path_str.contains(".git") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                // Skip Feign Clients and only include actual Controllers
                if content.contains("@FeignClient") {
                    continue;
                }
                
                if !content.contains("@RestController") && 
                   !content.contains("@Controller") && 
                   !content.contains("@RequestMapping") {
                    continue;
                }

                let mut requests = Vec::new();

                // Helper to find body for a match
                let find_body = |pos: usize| -> RequestBody {
                    let slice_end = std::cmp::min(content.len(), pos + 500);
                    if let Some(bcap) = request_body_regex.captures(&content[pos..slice_end]) {
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

                // Check for @XMapping
                for cap in mapping_regex.captures_iter(&content) {
                    let method_prefix = &cap[1];
                    let url_path = &cap[2];

                    let method = match method_prefix.to_lowercase().as_str() {
                        "post" => Method::Post,
                        "put" => Method::Put,
                        "patch" => Method::Patch,
                        "delete" => Method::Delete,
                        _ => Method::Get,
                    };

                    let body = find_body(cap.get(0).unwrap().end());

                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("{} {}", method_prefix.to_uppercase(), url_path),
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

                // Check for @RequestMapping
                for cap in request_mapping_regex.captures_iter(&content) {
                    let url_path = &cap[1];
                    let method_str = cap.get(2).map(|m| m.as_str()).unwrap_or("GET");

                    let method = match method_str.to_uppercase().as_str() {
                        "POST" => Method::Post,
                        "PUT" => Method::Put,
                        "PATCH" => Method::Patch,
                        "DELETE" => Method::Delete,
                        _ => Method::Get,
                    };

                    let body = find_body(cap.get(0).unwrap().end());

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
