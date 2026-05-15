use crate::cli::args::Method;
use crate::core::collection::{
    Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody,
};
use crate::core::parser::SourceParser;
use crate::core::parser::models::{FieldType, Model, ModelField, ModelRegistry};
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct ExpressParser;

impl ExpressParser {
    fn parse_ts_type(type_str: &str) -> FieldType {
        let type_str = type_str.trim();
        if type_str.is_empty() {
            return FieldType::Unknown;
        }

        match type_str.to_lowercase().as_str() {
            "string" => FieldType::String,
            "number" => FieldType::Number,
            "boolean" => FieldType::Boolean,
            t if t.ends_with("[]") => {
                let inner = &t[..t.len() - 2];
                FieldType::Array(Box::new(Self::parse_ts_type(inner)))
            }
            t if t.starts_with("Array<") && t.ends_with('>') => {
                let inner = &t[6..t.len() - 1];
                FieldType::Array(Box::new(Self::parse_ts_type(inner)))
            }
            _ => FieldType::Object(type_str.to_string()),
        }
    }
}

impl SourceParser for ExpressParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (Express)",
            project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));

        collection.env_vars.push(KVParam {
            key: "baseUrl".to_string(),
            value: "http://localhost:3000".to_string(),
            enabled: true,
            description: Some("Base URL for the service".to_string()),
        });

        let mut registry = ModelRegistry::new();

        // Pass 1: Model Discovery (Interfaces and Types)
        let interface_regex =
            Regex::new(r"(?m)^(?:export\s+)?(?:interface|type)\s+([a-zA-Z0-9_]+)\s*(?:=)?\s*\{")
                .unwrap();
        let field_regex = Regex::new(r"^\s*([a-zA-Z0-9_]+)\s*\??\s*:\s*([^;,\n]+)").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "ts"))
        {
            let path_str = entry.path().to_string_lossy();
            if path_str.contains("node_modules")
                || path_str.contains("dist")
                || path_str.contains(".next")
            {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let lines: Vec<&str> = content.lines().collect();
                let mut i = 0;
                while i < lines.len() {
                    if let Some(cap) = interface_regex.captures(lines[i]) {
                        let name = cap[1].to_string();
                        let mut fields = Vec::new();
                        i += 1;
                        while i < lines.len() && !lines[i].contains('}') {
                            if let Some(fcap) = field_regex.captures(lines[i]) {
                                fields.push(ModelField {
                                    name: fcap[1].to_string(),
                                    field_type: Self::parse_ts_type(&fcap[2]),
                                });
                            }
                            i += 1;
                        }
                        registry.add_model(Model { name, fields });
                    } else {
                        i += 1;
                    }
                }
            }
        }

        let route_regex = Regex::new(
            r#"(?:app|router|auth)\.(get|post|put|patch|delete)\s*(?:<[^>]+>)?\s*\(\s*['"]([^'"]+)['"]"#,
        )
        .unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "js" || ext == "ts")
            })
        {
            let path_str = entry.path().to_string_lossy();
            if path_str.contains("node_modules")
                || path_str.contains("dist")
                || path_str.contains(".next")
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

                    // Try to extract body type from Request<..., ReqBody>
                    let start_pos = cap.get(0).unwrap().start();
                    let end_pos = cap.get(0).unwrap().end();

                    let slice_start = if start_pos > 100 { start_pos - 100 } else { 0 };
                    let slice_end = std::cmp::min(content.len(), end_pos + 100);
                    let search_area = &content[slice_start..slice_end];

                    let req_type_regex =
                        Regex::new(r"Request\s*<[^,]+,\s*[^,]+,\s*([a-zA-Z0-9_]+)>").unwrap();
                    if let Some(tcap) = req_type_regex.captures(search_area) {
                        let body_type = &tcap[1];
                        if let Some(json_body) = registry.generate_json(body_type) {
                            body = RequestBody::Raw {
                                content: json_body,
                                content_type: "application/json".to_string(),
                            };
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
