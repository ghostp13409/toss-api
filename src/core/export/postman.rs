use crate::core::collection::{
    AuthType, Collection, CollectionItem, Folder, Request,
};
use serde_json::{json, Value};

pub fn export_postman(col: &Collection) -> anyhow::Result<String> {
    let mut items = Vec::new();
    for item in &col.items {
        items.push(convert_item(item));
    }

    let doc = json!({
        "info": {
            "_postman_id": col.id,
            "name": col.name,
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": items
    });

    Ok(serde_json::to_string_pretty(&doc)?)
}

fn convert_item(item: &CollectionItem) -> Value {
    match item {
        CollectionItem::Folder(folder) => convert_folder(folder),
        CollectionItem::Request(req) => convert_request(req),
    }
}

fn convert_folder(folder: &Folder) -> Value {
    let sub_items: Vec<Value> = folder.items.iter().map(convert_item).collect();
    json!({
        "name": folder.name,
        "item": sub_items
    })
}

fn convert_request(req: &Request) -> Value {
    let method_str = format!("{:?}", req.method).to_uppercase();

    // Headers
    let headers: Vec<Value> = req
        .headers
        .iter()
        .map(|h| {
            json!({
                "key": h.key,
                "value": h.value,
                "disabled": !h.enabled,
                "description": h.description
            })
        })
        .collect();

    // Query params & URL
    let query_params: Vec<Value> = req
        .params
        .iter()
        .map(|p| {
            json!({
                "key": p.key,
                "value": p.value,
                "disabled": !p.enabled,
                "description": p.description
            })
        })
        .collect();

    let url_obj = json!({
        "raw": req.url,
        "query": query_params
    });

    // Auth
    let auth_obj = match req.auth.selected {
        AuthType::Bearer => json!({
            "type": "bearer",
            "bearer": [{ "key": "token", "value": req.auth.bearer.token, "type": "string" }]
        }),
        AuthType::Basic => json!({
            "type": "basic",
            "basic": [
                { "key": "username", "value": req.auth.basic.username, "type": "string" },
                { "key": "password", "value": req.auth.basic.password, "type": "string" }
            ]
        }),
        AuthType::ApiKey => json!({
            "type": "apikey",
            "apikey": [
                { "key": "key", "value": req.auth.api_key.key, "type": "string" },
                { "key": "value", "value": req.auth.api_key.value, "type": "string" },
                { "key": "in", "value": if req.auth.api_key.in_header { "header" } else { "query" }, "type": "string" }
            ]
        }),
        AuthType::None => Value::Null,
    };

    // Body
    let body_obj = match req.body.selected {
        crate::core::collection::BodyType::Raw => json!({
            "mode": "raw",
            "raw": req.body.raw.content,
            "options": {
                "raw": {
                    "language": if req.body.raw.content_type.contains("json") { "json" } else { "text" }
                }
            }
        }),
        crate::core::collection::BodyType::FormData => {
            let formdata: Vec<Value> = req
                .body
                .form_data
                .items
                .iter()
                .map(|item| {
                    json!({
                        "key": item.key,
                        "value": item.value,
                        "type": "text",
                        "disabled": !item.enabled
                    })
                })
                .collect();
            json!({
                "mode": "formdata",
                "formdata": formdata
            })
        }
        crate::core::collection::BodyType::XWwwFormUrlEncoded => {
            let urlencoded: Vec<Value> = req
                .body
                .x_www_form_urlencoded
                .items
                .iter()
                .map(|item| {
                    json!({
                        "key": item.key,
                        "value": item.value,
                        "disabled": !item.enabled
                    })
                })
                .collect();
            json!({
                "mode": "urlencoded",
                "urlencoded": urlencoded
            })
        }
        crate::core::collection::BodyType::None => Value::Null,
    };

    let mut req_map = serde_json::Map::new();
    req_map.insert("method".to_string(), json!(method_str));
    req_map.insert("header".to_string(), json!(headers));
    req_map.insert("url".to_string(), url_obj);
    if !auth_obj.is_null() {
        req_map.insert("auth".to_string(), auth_obj);
    }
    if !body_obj.is_null() {
        req_map.insert("body".to_string(), body_obj);
    }

    let mut events = Vec::new();
    if let Some(pre) = &req.pre_request_script {
        if !pre.is_empty() {
            events.push(json!({
                "listen": "prerequest",
                "script": {
                    "type": "text/javascript",
                    "exec": pre.lines().collect::<Vec<&str>>()
                }
            }));
        }
    }
    if let Some(post) = &req.post_response_script {
        if !post.is_empty() {
            events.push(json!({
                "listen": "test",
                "script": {
                    "type": "text/javascript",
                    "exec": post.lines().collect::<Vec<&str>>()
                }
            }));
        }
    }

    let mut item_obj = serde_json::Map::new();
    item_obj.insert("name".to_string(), json!(req.name));
    item_obj.insert("request".to_string(), json!(req_map));
    if !events.is_empty() {
        item_obj.insert("event".to_string(), json!(events));
    }

    Value::Object(item_obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::Method;
    use crate::core::collection::{
        Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody,
    };

    #[test]
    fn test_export_postman_basic() {
        let mut col = Collection::new("Test Collection".to_string());
        let req = Request::new(
            "Get Test".to_string(),
            Method::Get,
            "https://httpbin.org/get".to_string(),
        );
        col.items.push(CollectionItem::Request(req));

        let json_str = export_postman(&col).expect("export should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["info"]["name"], "Test Collection");
        assert_eq!(
            v["info"]["schema"],
            "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        );
        assert_eq!(v["item"][0]["name"], "Get Test");
        assert_eq!(v["item"][0]["request"]["method"], "GET");
    }

    #[test]
    fn test_export_postman_folder_headers_params_auth_body() {
        let mut col = Collection::new("Full Collection".to_string());

        let mut folder = Folder::new("Users Folder".to_string());

        let mut req = Request::new(
            "Create User".to_string(),
            Method::Post,
            "https://api.example.com/users".to_string(),
        );
        req.headers.push(KVParam {
            key: "Content-Type".to_string(),
            value: "application/json".to_string(),
            enabled: true,
            description: Some("Header desc".to_string()),
        });
        req.params.push(KVParam {
            key: "verbose".to_string(),
            value: "true".to_string(),
            enabled: true,
            description: None,
        });

        req.auth = Auth::bearer("my-secret-token".to_string());
        req.body = RequestBody::raw(
            r#"{"name": "Alice"}"#.to_string(),
            "application/json".to_string(),
        );

        folder.items.push(CollectionItem::Request(req));
        col.items.push(CollectionItem::Folder(folder));

        let json_str = export_postman(&col).expect("export should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(v["info"]["name"], "Full Collection");
        let folder_val = &v["item"][0];
        assert_eq!(folder_val["name"], "Users Folder");

        let req_val = &folder_val["item"][0];
        assert_eq!(req_val["name"], "Create User");
        assert_eq!(req_val["request"]["method"], "POST");

        // Headers
        assert_eq!(req_val["request"]["header"][0]["key"], "Content-Type");
        assert_eq!(req_val["request"]["header"][0]["value"], "application/json");

        // Query params
        assert_eq!(req_val["request"]["url"]["query"][0]["key"], "verbose");

        // Auth
        assert_eq!(req_val["request"]["auth"]["type"], "bearer");
        assert_eq!(
            req_val["request"]["auth"]["bearer"][0]["value"],
            "my-secret-token"
        );

        // Body
        assert_eq!(req_val["request"]["body"]["mode"], "raw");
        assert_eq!(
            req_val["request"]["body"]["raw"],
            r#"{"name": "Alice"}"#
        );
        assert_eq!(
            req_val["request"]["body"]["options"]["raw"]["language"],
            "json"
        );
    }

    #[test]
    fn test_export_postman_auth_variants() {
        let mut col = Collection::new("Auth Test".to_string());

        let mut req_basic = Request::new(
            "Basic Auth Req".to_string(),
            Method::Get,
            "https://httpbin.org/basic-auth".to_string(),
        );
        req_basic.auth = Auth::basic("admin".to_string(), "pass123".to_string());

        let mut req_api_key = Request::new(
            "ApiKey Auth Req".to_string(),
            Method::Get,
            "https://httpbin.org/get".to_string(),
        );
        req_api_key.auth = Auth::api_key("X-API-Key".to_string(), "key123".to_string(), true);

        col.items.push(CollectionItem::Request(req_basic));
        col.items.push(CollectionItem::Request(req_api_key));

        let json_str = export_postman(&col).expect("export should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(v["item"][0]["request"]["auth"]["type"], "basic");
        assert_eq!(v["item"][1]["request"]["auth"]["type"], "apikey");
    }

    #[test]
    fn test_export_postman_body_variants() {
        let mut col = Collection::new("Body Test".to_string());

        let mut req_form = Request::new(
            "Form Data Req".to_string(),
            Method::Post,
            "https://httpbin.org/post".to_string(),
        );
        req_form.body = RequestBody::form_data(vec![KVParam {
            key: "field1".to_string(),
            value: "val1".to_string(),
            enabled: true,
            description: None,
        }]);

        let mut req_urlencoded = Request::new(
            "Urlencoded Req".to_string(),
            Method::Post,
            "https://httpbin.org/post".to_string(),
        );
        req_urlencoded.body = RequestBody::x_www_form_urlencoded(vec![KVParam {
            key: "param1".to_string(),
            value: "val1".to_string(),
            enabled: true,
            description: None,
        }]);

        col.items.push(CollectionItem::Request(req_form));
        col.items.push(CollectionItem::Request(req_urlencoded));

        let json_str = export_postman(&col).expect("export should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(v["item"][0]["request"]["body"]["mode"], "formdata");
        assert_eq!(v["item"][1]["request"]["body"]["mode"], "urlencoded");
    }

    #[test]
    fn test_export_postman_scripts() {
        let mut col = Collection::new("Script Export Test".to_string());
        let mut req = Request::new(
            "Script Req".to_string(),
            Method::Get,
            "https://httpbin.org/get".to_string(),
        );
        req.pre_request_script = Some("console.log('pre');".to_string());
        req.post_response_script = Some("client.test('status', function() {});".to_string());

        col.items.push(CollectionItem::Request(req));

        let json_str = export_postman(&col).expect("export should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(v["item"][0]["event"][0]["listen"], "prerequest");
        assert_eq!(v["item"][0]["event"][0]["script"]["exec"][0], "console.log('pre');");
        assert_eq!(v["item"][0]["event"][1]["listen"], "test");
        assert_eq!(v["item"][0]["event"][1]["script"]["exec"][0], "client.test('status', function() {});");
    }
}
