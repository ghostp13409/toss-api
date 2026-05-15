use crate::cli::args::Method;
use crate::core::collection::{
    Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody,
};
use crate::core::parser::SourceParser;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct DjangoParser;

impl SourceParser for DjangoParser {
    fn parse(&self, project_path: &Path) -> anyhow::Result<Collection> {
        let mut collection = Collection::new(format!(
            "{} (Django)",
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

        // path('admin/', admin.site.urls),
        let path_regex = Regex::new(r#"path\s*\(\s*['"]([^'"]*)['"]"#).unwrap();
        // re_path(r'^articles/(?P<year>[0-9]{4})/$', views.year_archive),
        let re_path_regex = Regex::new(r#"re_path\s*\(\s*r?['"]([^'"]*)['"]"#).unwrap();

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().file_name().map_or(false, |name| name == "urls.py"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let mut requests = Vec::new();

                for cap in path_regex.captures_iter(&content) {
                    let url_path = &cap[1];
                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("GET /{}", url_path),
                        method: Method::Get,
                        url: format!("{{{{baseUrl}}}}/{}", url_path),
                        params: Vec::new(),
                        headers: Vec::new(),
                        auth: Auth::None,
                        body: RequestBody::None,
                        pre_request_script: None,
                        post_response_script: None,
                    }));
                }

                for cap in re_path_regex.captures_iter(&content) {
                    let url_path = &cap[1];
                    requests.push(CollectionItem::Request(Request {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: format!("GET /{}", url_path),
                        method: Method::Get,
                        url: format!("{{{{baseUrl}}}}/{}", url_path),
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
                        .parent()
                        .and_then(|p| p.file_name())
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let mut folder = Folder::new(format!("{}/urls.py", file_name));
                    folder.items = requests;
                    collection.items.push(CollectionItem::Folder(folder));
                }
            }
        }

        Ok(collection)
    }
}
