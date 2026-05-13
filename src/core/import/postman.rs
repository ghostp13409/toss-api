use crate::core::collection::{Collection, CollectionItem, Request, Folder, KVParam, RequestBody, Auth};
use crate::cli::args::Method;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn import_postman<P: AsRef<Path>>(path: P) -> anyhow::Result<Collection> {
    let content = fs::read_to_string(path)?;
    import_postman_collection(&content)
}

pub fn import_postman_collection(json_str: &str) -> anyhow::Result<Collection> {
    let v: Value = serde_json::from_str(json_str)?;
    
    let name = v["info"]["name"].as_str().unwrap_or("Imported Collection").to_string();
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
                RequestBody::Raw {
                    content: body_val["raw"].as_str().unwrap_or("").to_string(),
                    content_type: body_val["options"]["raw"]["language"].as_str().unwrap_or("json").to_string(),
                }
            } else {
                RequestBody::None
            }
        } else {
            RequestBody::None
        };

        Some(CollectionItem::Request(Request {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            method,
            url,
            headers,
            params: Vec::new(),
            auth: Auth::None,
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
