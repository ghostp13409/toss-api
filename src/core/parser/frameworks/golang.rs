use crate::cli::args::Method;
use crate::core::collection::{Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct GolangParser;

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

        // router.GET("/path", ...) or http.HandleFunc("/path", ...)
        let route_regex = Regex::new(r#"\.(?:HandleFunc|GET|POST|PUT|PATCH|DELETE)\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

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

                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("{} {}", if method_str == "HandleFunc" { "REQ" } else { method_str }, url_path),
                        method,
                        url: format!("{{{{baseUrl}}}}/{}", url_path.trim_start_matches('/')),
                        params: Vec::new(),
                        headers: Vec::new(),
                        auth: Auth::None,
                        body: RequestBody::None,
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
