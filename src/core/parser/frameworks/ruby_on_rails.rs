use crate::cli::args::Method;
use crate::core::collection::{Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;

pub struct RubyOnRailsParser;

impl SourceParser for RubyOnRailsParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (Ruby on Rails)",
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

        // get 'path', to: 'controller#action'
        let route_regex = Regex::new(r#"(get|post|put|patch|delete)\s+['"]([^'"]+)['"]"#).unwrap();

        let routes_path = project_path.join("config").join("routes.rb");
        if let Ok(content) = std::fs::read_to_string(routes_path) {
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
                let mut folder = Folder::new("routes.rb".to_string());
                folder.items = requests;
                collection.items.push(CollectionItem::Folder(folder));
            }
        }

        Ok(collection)
    }
}
