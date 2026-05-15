use crate::cli::args::Method;
use crate::core::collection::{
    Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody,
};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct QuarkusParser;

impl SourceParser for QuarkusParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (Quarkus)",
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

        // @Path("/path")
        let path_regex = Regex::new(r#"@Path\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
        // @GET, @POST, etc.
        let method_regex = Regex::new(r#"@(GET|POST|PUT|PATCH|DELETE)"#).unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "java" || ext == "kotlin")
            })
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let mut requests = Vec::new();

                // Find class level path if any
                let class_path = path_regex
                    .captures(&content)
                    .map(|c| c[1].to_string())
                    .unwrap_or_default();

                for cap in path_regex.captures_iter(&content) {
                    let url_path = &cap[1];
                    if url_path == &class_path {
                        continue;
                    } // Skip class level path as it's handled in the final URL

                    // Look for the method annotation right before or after the path annotation
                    // This is naive, but works for basic cases
                    let start_idx = cap.get(0).unwrap().start();
                    let context = if start_idx > 50 {
                        &content[start_idx - 50..start_idx + url_path.len() + 50]
                    } else {
                        &content[0..start_idx + url_path.len() + 50]
                    };

                    let method_str = method_regex
                        .captures(context)
                        .map(|m| m[1].to_string())
                        .unwrap_or("GET".to_string());

                    let method = match method_str.to_uppercase().as_str() {
                        "POST" => Method::Post,
                        "PUT" => Method::Put,
                        "PATCH" => Method::Patch,
                        "DELETE" => Method::Delete,
                        _ => Method::Get,
                    };

                    let full_path = format!(
                        "{}/{}",
                        class_path.trim_end_matches('/'),
                        url_path.trim_start_matches('/')
                    );

                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("{} {}", method_str.to_uppercase(), full_path),
                        method,
                        url: format!("{{{{baseUrl}}}}/{}", full_path.trim_start_matches('/')),
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
