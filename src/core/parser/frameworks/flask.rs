use crate::cli::args::Method;
use crate::core::collection::{Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct FlaskParser;

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

        // @app.route('/path', methods=['GET', 'POST'])
        let route_regex = Regex::new(r#"@(?:[a-zA-Z0-9_]+)\.route\s*\(\s*['"]([^'"]+)['"](?:\s*,\s*methods\s*=\s*\[([^\]]+)\])?"#).unwrap();

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
                        methods_str.split(',').map(|s| s.trim().trim_matches('\'').trim_matches('"')).collect::<Vec<_>>()
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

                        requests.push(CollectionItem::Request(Request {
                            id: uuid::Uuid::new_v4().to_string(),
                            name: format!("{} {}", m.to_uppercase(), url_path),
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
