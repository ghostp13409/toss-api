use crate::core::collection::{
    AuthType, BodyType, Collection, CollectionItem, KVParam, Request,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub fn export_openapi(col: &Collection) -> anyhow::Result<String> {
    let mut requests = Vec::new();
    collect_requests(&col.items, &mut requests);

    let mut paths_map: BTreeMap<String, Map<String, Value>> = BTreeMap::new();
    let mut has_bearer = false;
    let mut has_basic = false;
    let mut has_apikey = false;

    for req in requests {
        let (path_str, method_str, op_val) = convert_request_to_operation(req);

        match req.auth.selected {
            AuthType::Bearer => has_bearer = true,
            AuthType::Basic => has_basic = true,
            AuthType::ApiKey => has_apikey = true,
            AuthType::None => {}
        }

        paths_map
            .entry(path_str)
            .or_default()
            .insert(method_str, op_val);
    }

    let mut paths_json = Map::new();
    for (path, ops) in paths_map {
        paths_json.insert(path, Value::Object(ops));
    }

    let mut security_schemes = Map::new();
    if has_bearer {
        security_schemes.insert(
            "bearerAuth".to_string(),
            json!({
                "type": "http",
                "scheme": "bearer"
            }),
        );
    }
    if has_basic {
        security_schemes.insert(
            "basicAuth".to_string(),
            json!({
                "type": "http",
                "scheme": "basic"
            }),
        );
    }
    if has_apikey {
        security_schemes.insert(
            "apiKeyAuth".to_string(),
            json!({
                "type": "apiKey",
                "name": "api_key",
                "in": "header"
            }),
        );
    }

    let doc = json!({
        "openapi": "3.0.3",
        "info": {
            "title": col.name,
            "version": "1.0.0"
        },
        "paths": paths_json,
        "components": {
            "securitySchemes": security_schemes
        }
    });

    Ok(serde_json::to_string_pretty(&doc)?)
}

fn collect_requests<'a>(items: &'a [CollectionItem], acc: &mut Vec<&'a Request>) {
    for item in items {
        match item {
            CollectionItem::Folder(folder) => collect_requests(&folder.items, acc),
            CollectionItem::Request(req) => acc.push(req),
        }
    }
}

fn convert_request_to_operation(req: &Request) -> (String, String, Value) {
    let method = format!("{:?}", req.method).to_lowercase();
    let (path, url_params) = extract_path_and_query(&req.url);

    let mut parameters = Vec::new();

    // Add URL query params from req.url + req.params
    for p in url_params.iter().chain(req.params.iter()) {
        if p.key.is_empty() {
            continue;
        }
        parameters.push(json!({
            "name": p.key,
            "in": "query",
            "required": false,
            "schema": { "type": "string" },
            "description": p.description
        }));
    }

    // Add header params
    for h in &req.headers {
        if h.key.is_empty() {
            continue;
        }
        parameters.push(json!({
            "name": h.key,
            "in": "header",
            "required": false,
            "schema": { "type": "string" },
            "description": h.description
        }));
    }

    // Security
    let security = match req.auth.selected {
        AuthType::Bearer => vec![json!({ "bearerAuth": [] })],
        AuthType::Basic => vec![json!({ "basicAuth": [] })],
        AuthType::ApiKey => vec![json!({ "apiKeyAuth": [] })],
        AuthType::None => vec![],
    };

    let mut op_map = Map::new();
    op_map.insert("summary".to_string(), json!(req.name));
    if !parameters.is_empty() {
        op_map.insert("parameters".to_string(), json!(parameters));
    }
    if !security.is_empty() {
        op_map.insert("security".to_string(), json!(security));
    }

    // Request body
    match req.body.selected {
        BodyType::Raw => {
            let ct = if req.body.raw.content_type.is_empty() {
                "application/json"
            } else {
                &req.body.raw.content_type
            };
            op_map.insert(
                "requestBody".to_string(),
                json!({
                    "content": {
                        ct: {
                            "schema": { "type": "string" },
                            "example": req.body.raw.content
                        }
                    }
                }),
            );
        }
        BodyType::FormData => {
            op_map.insert(
                "requestBody".to_string(),
                json!({
                    "content": {
                        "multipart/form-data": {
                            "schema": { "type": "object" }
                        }
                    }
                }),
            );
        }
        BodyType::XWwwFormUrlEncoded => {
            op_map.insert(
                "requestBody".to_string(),
                json!({
                    "content": {
                        "application/x-www-form-urlencoded": {
                            "schema": { "type": "object" }
                        }
                    }
                }),
            );
        }
        BodyType::None => {}
    }

    op_map.insert(
        "responses".to_string(),
        json!({
            "200": {
                "description": "Successful response"
            }
        }),
    );

    (path, method, Value::Object(op_map))
}

fn extract_path_and_query(url_str: &str) -> (String, Vec<KVParam>) {
    if url_str.is_empty() {
        return ("/".to_string(), Vec::new());
    }

    let url_without_scheme = if let Some(pos) = url_str.find("://") {
        &url_str[pos + 3..]
    } else {
        url_str
    };

    let (path_and_query, _) = url_without_scheme.split_once('#').unwrap_or((url_without_scheme, ""));
    let (raw_path, raw_query) = path_and_query.split_once('?').unwrap_or((path_and_query, ""));

    let path = if let Some(first_slash) = raw_path.find('/') {
        &raw_path[first_slash..]
    } else {
        "/"
    };

    let mut query_params = Vec::new();
    if !raw_query.is_empty() {
        for pair in raw_query.split('&') {
            if pair.is_empty() {
                continue;
            }
            if let Some((k, v)) = pair.split_once('=') {
                query_params.push(KVParam {
                    key: k.to_string(),
                    value: v.to_string(),
                    enabled: true,
                    description: None,
                });
            } else {
                query_params.push(KVParam {
                    key: pair.to_string(),
                    value: String::new(),
                    enabled: true,
                    description: None,
                });
            }
        }
    }

    (path.to_string(), query_params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collection::{
        Auth, Collection, CollectionItem, Folder, KVParam, Request, RequestBody,
    };
    use crate::cli::args::Method;

    #[test]
    fn test_export_openapi_basic() {
        let mut col = Collection::new("Petstore API".to_string());
        let req = Request::new(
            "Get Pet".to_string(),
            Method::Get,
            "https://petstore.swagger.io/v2/pet/123".to_string(),
        );
        col.items.push(CollectionItem::Request(req));

        let json_str = export_openapi(&col).expect("export openapi should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["openapi"], "3.0.3");
        assert_eq!(v["info"]["title"], "Petstore API");
        assert!(v["paths"].as_object().unwrap().contains_key("/v2/pet/123"));
        assert_eq!(v["paths"]["/v2/pet/123"]["get"]["summary"], "Get Pet");
    }

    #[test]
    fn test_export_openapi_nested_folder_and_auth_and_body() {
        let mut col = Collection::new("Full API".to_string());

        let mut folder = Folder::new("Users".to_string());
        let mut req1 = Request::new(
            "Create User".to_string(),
            Method::Post,
            "https://api.example.com/users?source=web".to_string(),
        );
        req1.auth = Auth::bearer("secret_token".to_string());
        req1.body = RequestBody::raw(
            r#"{"name": "Alice"}"#.to_string(),
            "application/json".to_string(),
        );
        req1.headers.push(KVParam {
            key: "X-Request-ID".to_string(),
            value: "12345".to_string(),
            enabled: true,
            description: Some("Request identifier".to_string()),
        });

        folder.items.push(CollectionItem::Request(req1));
        col.items.push(CollectionItem::Folder(folder));

        let json_str = export_openapi(&col).expect("export openapi should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(v["openapi"], "3.0.3");
        assert_eq!(v["info"]["title"], "Full API");
        assert!(v["paths"].as_object().unwrap().contains_key("/users"));

        let post_op = &v["paths"]["/users"]["post"];
        assert_eq!(post_op["summary"], "Create User");

        // Check query param from URL
        let params = post_op["parameters"].as_array().unwrap();
        let query_param = params.iter().find(|p| p["name"] == "source").unwrap();
        assert_eq!(query_param["in"], "query");

        // Check header param
        let header_param = params.iter().find(|p| p["name"] == "X-Request-ID").unwrap();
        assert_eq!(header_param["in"], "header");
        assert_eq!(header_param["description"], "Request identifier");

        // Check security scheme
        assert!(v["components"]["securitySchemes"].as_object().unwrap().contains_key("bearerAuth"));
        assert_eq!(
            post_op["security"][0]["bearerAuth"],
            serde_json::json!([])
        );

        // Check request body
        let body = &post_op["requestBody"]["content"]["application/json"];
        assert_eq!(body["example"], r#"{"name": "Alice"}"#);
    }

    #[test]
    fn test_export_openapi_auth_and_body_variants() {
        let mut col = Collection::new("Auth & Body API".to_string());

        let mut req1 = Request::new(
            "Basic Auth Request".to_string(),
            Method::Get,
            "https://api.example.com/basic".to_string(),
        );
        req1.auth = Auth::basic("user".to_string(), "pass".to_string());

        let mut req2 = Request::new(
            "ApiKey Auth Request".to_string(),
            Method::Post,
            "https://api.example.com/apikey".to_string(),
        );
        req2.auth = Auth::api_key("X-API-KEY".to_string(), "secret".to_string(), true);
        req2.body = RequestBody::form_data(vec![KVParam {
            key: "file".to_string(),
            value: "binary".to_string(),
            enabled: true,
            description: None,
        }]);

        let mut req3 = Request::new(
            "Urlencoded Request".to_string(),
            Method::Put,
            "https://api.example.com/form".to_string(),
        );
        req3.body = RequestBody::x_www_form_urlencoded(vec![KVParam {
            key: "grant_type".to_string(),
            value: "password".to_string(),
            enabled: true,
            description: None,
        }]);

        col.items.push(CollectionItem::Request(req1));
        col.items.push(CollectionItem::Request(req2));
        col.items.push(CollectionItem::Request(req3));

        let json_str = export_openapi(&col).expect("export openapi should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let schemes = v["components"]["securitySchemes"].as_object().unwrap();
        assert!(schemes.contains_key("basicAuth"));
        assert!(schemes.contains_key("apiKeyAuth"));

        let post_op = &v["paths"]["/apikey"]["post"];
        assert!(post_op["requestBody"]["content"]["multipart/form-data"].is_object());

        let put_op = &v["paths"]["/form"]["put"];
        assert!(put_op["requestBody"]["content"]["application/x-www-form-urlencoded"].is_object());
    }
}


