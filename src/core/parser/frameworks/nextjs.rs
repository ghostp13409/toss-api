use crate::cli::args::Method;
use crate::core::collection::{Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct NextJsParser;

impl SourceParser for NextJsParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (Next.js)",
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

        // App Router: app/api/**/route.ts|js
        let app_router_regex = Regex::new(r#"export\s+(?:async\s+)?function\s+(GET|POST|PUT|PATCH|DELETE)"#).unwrap();
        
        let app_dir = project_path.join("app");
        if app_dir.exists() {
            for entry in WalkDir::new(app_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().file_name().map_or(false, |name| name == "route.ts" || name == "route.js"))
            {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let mut requests = Vec::new();
                    let relative_path = entry.path().strip_prefix(project_path).unwrap_or(entry.path());
                    let url_path = relative_path.parent().unwrap_or(Path::new("")).to_string_lossy().to_string();
                    let url_path = url_path.replace("app", "").replace("\\", "/"); // Basic normalization

                    for cap in app_router_regex.captures_iter(&content) {
                        let method_str = &cap[1];
                        let method = match method_str.to_uppercase().as_str() {
                            "POST" => Method::Post,
                            "PUT" => Method::Put,
                            "PATCH" => Method::Patch,
                            "DELETE" => Method::Delete,
                            _ => Method::Get,
                        };

                        requests.push(CollectionItem::Request(Request {
                            id: uuid::Uuid::new_v4().to_string(),
                            name: format!("{} {}", method_str, url_path),
                            method,
                            url: format!("{{{{baseUrl}}}}{}", url_path),
                            params: Vec::new(),
                            headers: Vec::new(),
                            auth: Auth::None,
                            body: RequestBody::None,
                            pre_request_script: None,
                            post_response_script: None,
                        }));
                    }

                    if !requests.is_empty() {
                        let mut folder = Folder::new(url_path);
                        folder.items = requests;
                        collection.items.push(CollectionItem::Folder(folder));
                    }
                }
            }
        }

        // Pages Router: pages/api/**.ts|js
        let pages_api_dir = project_path.join("pages").join("api");
        if pages_api_dir.exists() {
            for entry in WalkDir::new(pages_api_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "ts" || ext == "js"))
            {
                let relative_path = entry.path().strip_prefix(project_path).unwrap_or(entry.path());
                let url_path = format!("/{}", relative_path.with_extension("").to_string_lossy().replace("\\", "/"));

                let req = CollectionItem::Request(Request {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: format!("API {}", url_path),
                    method: Method::Get, // Pages router often handles multiple methods in one handler
                    url: format!("{{{{baseUrl}}}}{}", url_path),
                    params: Vec::new(),
                    headers: Vec::new(),
                    auth: Auth::None,
                    body: RequestBody::None,
                    pre_request_script: None,
                    post_response_script: None,
                });

                let mut folder = Folder::new(entry.path().file_name().unwrap_or_default().to_string_lossy().to_string());
                folder.items = vec![req];
                collection.items.push(CollectionItem::Folder(folder));
            }
        }

        Ok(collection)
    }
}
