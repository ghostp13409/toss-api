use crate::cli::args::Method;
use crate::core::collection::{
    Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody,
};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn import_postman<P: AsRef<Path>>(path: P) -> anyhow::Result<Collection> {
    let content = fs::read_to_string(path)?;
    import_postman_collection(&content)
}

pub fn import_postman_collection(json_str: &str) -> anyhow::Result<Collection> {
    let v: Value = serde_json::from_str(json_str)?;

    let name = v["info"]["name"]
        .as_str()
        .unwrap_or("Imported Collection")
        .to_string();
    let mut collection = Collection::new(name);

    if let Some(items) = v["item"].as_array() {
        for item in items {
            if let Some(c_item) = parse_item(item) {
                collection.items.push(c_item);
            }
        }
    }

    Ok(collection)
}

fn parse_item(item: &Value) -> Option<CollectionItem> {
    let name = item["name"].as_str().unwrap_or("Unnamed").to_string();

    if let Some(request) = item.get("request") {
        let method_str = request["method"].as_str().unwrap_or("GET");
        let method = match method_str.to_uppercase().as_str() {
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            _ => Method::Get,
        };

        // Extract params from 'query' array if it exists
        let mut query_params = Vec::new();
        if let Some(url_val) = request.get("url") {
            if let Some(query_array) = url_val.get("query")
                && let Some(query_list) = query_array.as_array()
            {
                for q in query_list {
                    query_params.push(KVParam {
                        key: q["key"].as_str().unwrap_or("").to_string(),
                        value: q["value"].as_str().unwrap_or("").to_string(),
                        enabled: !q["disabled"].as_bool().unwrap_or(false),
                        description: q["description"].as_str().map(|s| s.to_string()),
                    });
                }
            }
        }

        let url = if let Some(url_val) = request.get("url") {
            if let Some(raw) = url_val.get("raw") {
                raw.as_str().unwrap_or("").to_string()
            } else if let Some(url_str) = url_val.as_str() {
                url_str.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut params = Vec::new();
        // Parse the URL for parameters
        if !url.is_empty() {
            if let Some(q_pos) = url.find('?') {
                let query_str = &url[q_pos + 1..];
                for s in query_str.split('&').filter(|s| !s.is_empty()) {
                    let (k, v) = s.split_once('=').unwrap_or((s, ""));
                    let mut p = KVParam {
                        key: k.to_string(),
                        value: v.to_string(),
                        enabled: true,
                        description: None,
                    };

                    // Enrich with info from Postman's query array if available
                    if let Some(existing) = query_params.iter().find(|qp| qp.key == p.key) {
                        p.enabled = existing.enabled;
                        p.description = existing.description.clone();
                    }
                    params.push(p);
                }
            }
        }

        // If URL parsing didn't find any params but query_params has some, use those
        if params.is_empty() && !query_params.is_empty() {
            params = query_params;
        }

        let mut headers = Vec::new();
        if let Some(header_array) = request.get("header")
            && let Some(headers_list) = header_array.as_array()
        {
            for h in headers_list {
                headers.push(KVParam {
                    key: h["key"].as_str().unwrap_or("").to_string(),
                    value: h["value"].as_str().unwrap_or("").to_string(),
                    enabled: !h["disabled"].as_bool().unwrap_or(false),
                    description: h["description"].as_str().map(|s| s.to_string()),
                });
            }
        }

        let body = if let Some(body_val) = request.get("body") {
            if body_val["mode"] == "raw" {
                RequestBody::raw(
                    body_val["raw"].as_str().unwrap_or("").to_string(),
                    body_val["options"]["raw"]["language"]
                        .as_str()
                        .unwrap_or("json")
                        .to_string(),
                )
            } else {
                RequestBody::default()
            }
        } else {
            RequestBody::default()
        };


        Some(CollectionItem::Request(Request {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            method,
            url,
            headers,
            params,
            auth: Auth::default(),
            body,
            pre_request_script: None,
            post_response_script: None,
        }))
    } else if let Some(items) = item.get("item") {
        let mut folder = Folder::new(name);
        if let Some(item_array) = items.as_array() {
            for it in item_array {
                if let Some(c_item) = parse_item(it) {
                    folder.items.push(c_item);
                }
            }
        }
        Some(CollectionItem::Folder(folder))
    } else {
        None
    }
}
