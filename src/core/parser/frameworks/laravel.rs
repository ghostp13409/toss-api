use crate::cli::args::Method;
use crate::core::collection::{
    Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody,
};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct LaravelParser;

impl SourceParser for LaravelParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (Laravel)",
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

        // Route::get('/path', ...)
        let route_regex =
            Regex::new(r#"Route::(get|post|put|patch|delete)\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

        for entry in WalkDir::new(project_path.join("routes"))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "php"))
        {
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

                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("{} {}", method_str.to_uppercase(), url_path),
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
