use crate::cli::args::Method;
use crate::core::collection::{
    Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody,
};
use crate::core::parser::SourceParser;
use crate::core::parser::models::{FieldType, Model, ModelField, ModelRegistry};
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct FlaskParser;

impl FlaskParser {
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
                FieldType::Array(Box::new(FieldType::Unknown))
            }
            _ => FieldType::Object(type_str.to_string()),
        }
    }
}

impl SourceParser for FlaskParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (Flask)",
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

        // Pass 1: Model Discovery
        let class_regex = Regex::new(r"(?m)^class\s+([a-zA-Z0-9_]+)(?:\s*\(.*\))?:").unwrap();
        let field_regex = Regex::new(r"^\s+([a-zA-Z0-9_]+)\s*:\s*([a-zA-Z0-9_\[\]]+)").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "py"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let lines: Vec<&str> = content.lines().collect();
                let mut i = 0;
                while i < lines.len() {
                    if let Some(cap) = class_regex.captures(lines[i]) {
                        let name = cap[1].to_string();
                        let mut fields = Vec::new();
                        i += 1;
                        while i < lines.len()
                            && (lines[i].starts_with(' ')
                                || lines[i].starts_with('\t')
                                || lines[i].is_empty())
                        {
                            if let Some(fcap) = field_regex.captures(lines[i]) {
                                fields.push(ModelField {
                                    name: fcap[1].to_string(),
                                    field_type: Self::parse_python_type(&fcap[2]),
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

        // Pass 2: Endpoint Extraction
        // @app.route('/path', methods=['GET', 'POST'])
        let route_regex = Regex::new(r#"@(?:[a-zA-Z0-9_]+)\.route\s*\(\s*['"]([^'"]+)['"](?:\s*,\s*methods\s*=\s*\[([^\]]+)\])?"#).unwrap();
        let json_load_regex = Regex::new(r"([a-zA-Z0-9_]+)\s*=\s*request\.get_json\(\)").unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "py"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let mut requests = Vec::new();

                for cap in route_regex.captures_iter(&content) {
                    let url_path = &cap[1];
                    let methods_str = cap.get(2).map_or("GET", |m| m.as_str());

                    let methods = if methods_str.contains(',') {
                        methods_str
                            .split(',')
                            .map(|s| s.trim().trim_matches('\'').trim_matches('"'))
                            .collect::<Vec<_>>()
                    } else {
                        vec![methods_str.trim().trim_matches('\'').trim_matches('"')]
                    };

                    for m in methods {
                        let method = match m.to_uppercase().as_str() {
                            "POST" => Method::Post,
                            "PUT" => Method::Put,
                            "PATCH" => Method::Patch,
                            "DELETE" => Method::Delete,
                            _ => Method::Get,
                        };

                        let mut body = RequestBody::None;
                        if method == Method::Post
                            || method == Method::Put
                            || method == Method::Patch
                        {
                            let pos = cap.get(0).unwrap().end();
                            let slice_end = std::cmp::min(content.len(), pos + 1000);
                            if let Some(jcap) = json_load_regex.captures(&content[pos..slice_end]) {
                                let var_name = &jcap[1];
                                // Check if variable name matches a model (heuristic)
                                for model_name in registry.models.keys() {
                                    if var_name.to_lowercase().contains(&model_name.to_lowercase())
                                        || model_name.to_lowercase().contains(var_name)
                                    {
                                        if let Some(json_body) = registry.generate_json(model_name)
                                        {
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
                            name: format!("{} {}", m.to_uppercase(), url_path),
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
