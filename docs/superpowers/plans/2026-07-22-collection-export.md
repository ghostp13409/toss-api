# Collection Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add collection export support for Postman v2.1 and OpenAPI 3.0.3 JSON formats in both CLI (`toss-api export`) and TUI (`:export`).

**Architecture:** Create a `src/core/export/` module containing `postman.rs` and `openapi.rs` serializers. Wire CLI subcommand `Export` in `src/cli/args.rs` / `src/main.rs` and `:export` command in `src/tui/input/popups.rs`.

**Tech Stack:** Rust, serde / serde_json, clap, anyhow, ratatui (for TUI).

## Global Constraints

- Postman Schema: Collection v2.1.0 (`https://schema.getpostman.com/json/collection/v2.1.0/collection.json`).
- OpenAPI Schema: 3.0.3.
- CLI subcommand: `toss-api export <name> [-f postman|openapi] [-o path]`.
- TUI command: `:export [postman|openapi] [path]`.

---

### Task 1: Core Export Module & Postman Exporter

**Files:**
- Create: `src/core/export/mod.rs`
- Create: `src/core/export/postman.rs`
- Modify: `src/core/mod.rs`

**Interfaces:**
- Consumes: `Collection`, `CollectionItem`, `Folder`, `Request`, `KVParam`, `Auth`, `RequestBody` from `crate::core::collection`.
- Produces: `export_collection(col: &Collection, format: ExportFormat) -> anyhow::Result<String>`, `export_postman(col: &Collection) -> anyhow::Result<String>`.

- [ ] **Step 1: Write failing unit test for Postman exporter**

Create `src/core/export/postman.rs` with a failing test:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collection::{Collection, Request};
    use crate::cli::args::Method;

    #[test]
    fn test_export_postman_basic() {
        let mut col = Collection::new("Test Collection".to_string());
        let mut req = Request::default();
        req.name = "Get Test".to_string();
        req.method = Method::Get;
        req.url = "https://httpbin.org/get".to_string();
        col.items.push(crate::core::collection::CollectionItem::Request(req));

        let json_str = export_postman(&col).expect("export should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["info"]["name"], "Test Collection");
        assert_eq!(v["info"]["schema"], "https://schema.getpostman.com/json/collection/v2.1.0/collection.json");
        assert_eq!(v["item"][0]["name"], "Get Test");
        assert_eq!(v["item"][0]["request"]["method"], "GET");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_export_postman_basic`
Expected: FAIL due to missing module / function.

- [ ] **Step 3: Implement `src/core/export/mod.rs` and `src/core/export/postman.rs`**

In `src/core/export/mod.rs`:
```rust
pub mod openapi;
pub mod postman;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Postman,
    OpenApi,
}

pub fn export_collection(
    collection: &crate::core::collection::Collection,
    format: ExportFormat,
) -> anyhow::Result<String> {
    match format {
        ExportFormat::Postman => postman::export_postman(collection),
        ExportFormat::OpenApi => openapi::export_openapi(collection),
    }
}
```

In `src/core/export/postman.rs`:
```rust
use crate::core::collection::{
    AuthType, Collection, CollectionItem, Folder, KVParam, Request,
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
            let formdata: Vec<Value> = req.body.form_data.items.iter().map(|item| {
                json!({
                    "key": item.key,
                    "value": item.value,
                    "type": "text",
                    "disabled": !item.enabled
                })
            }).collect();
            json!({
                "mode": "formdata",
                "formdata": formdata
            })
        },
        crate::core::collection::BodyType::XWwwFormUrlEncoded => {
            let urlencoded: Vec<Value> = req.body.x_www_form_urlencoded.items.iter().map(|item| {
                json!({
                    "key": item.key,
                    "value": item.value,
                    "disabled": !item.enabled
                })
            }).collect();
            json!({
                "mode": "urlencoded",
                "urlencoded": urlencoded
            })
        },
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

    json!({
        "name": req.name,
        "request": req_map
    })
}
```

In `src/core/mod.rs`, add `pub mod export;`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_export_postman_basic`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/export src/core/mod.rs
git commit -m "feat: add postman collection export implementation"
```

---

### Task 2: OpenAPI 3.0.3 Exporter

**Files:**
- Create: `src/core/export/openapi.rs`

**Interfaces:**
- Consumes: `Collection`, `CollectionItem`, `Folder`, `Request`, `KVParam`, `Auth` from `crate::core::collection`.
- Produces: `export_openapi(col: &Collection) -> anyhow::Result<String>`.

- [ ] **Step 1: Write failing unit test for OpenAPI exporter**

In `src/core/export/openapi.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collection::{Collection, Request};
    use crate::cli::args::Method;

    #[test]
    fn test_export_openapi_basic() {
        let mut col = Collection::new("Petstore API".to_string());
        let mut req = Request::default();
        req.name = "Get Pet".to_string();
        req.method = Method::Get;
        req.url = "https://petstore.swagger.io/v2/pet/123".to_string();
        col.items.push(crate::core::collection::CollectionItem::Request(req));

        let json_str = export_openapi(&col).expect("export openapi should succeed");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["openapi"], "3.0.3");
        assert_eq!(v["info"]["title"], "Petstore API");
        assert!(v["paths"].as_object().unwrap().contains_key("/v2/pet/123"));
        assert_eq!(v["paths"]["/v2/pet/123"]["get"]["summary"], "Get Pet");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_export_openapi_basic`
Expected: FAIL due to missing `export_openapi` implementation.

- [ ] **Step 3: Implement `export_openapi` in `src/core/export/openapi.rs`**

```rust
use crate::core::collection::{
    AuthType, Collection, CollectionItem, KVParam, Request, RequestBody, BodyType,
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
            if let Some((k, v)) = pair.split_once('=') {
                query_params.push(KVParam {
                    key: k.to_string(),
                    value: v.to_string(),
                    enabled: true,
                    description: None,
                });
            }
        }
    }

    (path.to_string(), query_params)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_export_openapi_basic`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/export/openapi.rs
git commit -m "feat: add openapi 3.0 export implementation"
```

---

### Task 3: CLI Subcommand Integration

**Files:**
- Modify: `src/cli/args.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `ExportFormat`, `export_collection` from `crate::core::export`.
- Produces: CLI `toss-api export <name> -f <format> -o <output>`.

- [ ] **Step 1: Add `Export` subcommand to `src/cli/args.rs`**

```rust
    /// Export a saved collection to Postman or OpenAPI format JSON
    Export {
        /// Name of the collection to export
        name: String,

        /// Format to export: postman (default) or openapi
        #[arg(short, long, default_value = "postman")]
        format: String,

        /// Output file path (defaults to <collection>.<format>.json)
        #[arg(short, long)]
        output: Option<String>,
    },
```

- [ ] **Step 2: Add CLI export handler in `src/main.rs`**

In `src/main.rs`, match `Commands::Export`:
```rust
Commands::Export { name, format, output } => {
    let fmt = match format.to_lowercase().as_str() {
        "openapi" | "swagger" => toss_api::core::export::ExportFormat::OpenApi,
        _ => toss_api::core::export::ExportFormat::Postman,
    };

    let mut store = toss_api::core::persistence::CollectionStore::load()?;
    let col = match store.collections.iter().find(|c| c.name.eq_ignore_ascii_case(&name)) {
        Some(c) => c,
        None => {
            eprintln!("Collection '{}' not found.", name);
            eprintln!("Available collections:");
            for c in &store.collections {
                eprintln!("  - {}", c.name);
            }
            std::process::exit(1);
        }
    };

    let json_output = toss_api::core::export::export_collection(col, fmt)?;
    let out_path = output.unwrap_or_else(|| {
        let ext = match fmt {
            toss_api::core::export::ExportFormat::Postman => "postman_collection.json",
            toss_api::core::export::ExportFormat::OpenApi => "openapi.json",
        };
        format!("{}.{}", col.name.to_lowercase().replace(' ', "_"), ext)
    });

    std::fs::write(&out_path, json_output)?;
    println!("Exported collection '{}' to {}", col.name, out_path);
}
```

- [ ] **Step 3: Test CLI build and export command**

Run: `cargo build`
Run: `cargo run -- export --help`
Expected: Displays export command help output.

- [ ] **Step 4: Commit**

```bash
git add src/cli/args.rs src/main.rs
git commit -m "feat: add CLI export command"
```

---

### Task 4: TUI Command Mode Integration

**Files:**
- Modify: `src/tui/app/handlers/mod.rs`
- Modify: `src/tui/input/popups.rs`
- Modify: `src/tui/ui/widgets/popups.rs`

**Interfaces:**
- Consumes: `App::export_collection(&mut self, format_or_path: &str, maybe_path: Option<&str>)`

- [ ] **Step 1: Implement `export_collection` method in `src/tui/app/handlers/mod.rs`**

```rust
    pub fn export_active_collection(&mut self, format_str: &str, path: Option<&str>) {
        if self.collections.is_empty() {
            self.error_message = Some("No active collection to export".to_string());
            return;
        }

        let col = &self.collections[self.selected_collection_index];
        let (fmt, file_path) = match format_str.to_lowercase().as_str() {
            "openapi" | "swagger" => (
                crate::core::export::ExportFormat::OpenApi,
                path.unwrap_or("exported_collection.openapi.json"),
            ),
            "postman" => (
                crate::core::export::ExportFormat::Postman,
                path.unwrap_or("exported_collection.postman.json"),
            ),
            _ => {
                // If format is not recognized, treat format_str as the output path for postman format
                (
                    crate::core::export::ExportFormat::Postman,
                    format_str,
                )
            }
        };

        match crate::core::export::export_collection(col, fmt) {
            Ok(content) => {
                if let Err(e) = std::fs::write(file_path, content) {
                    self.error_message = Some(format!("Failed to write export file: {}", e));
                } else {
                    self.status_message = Some(format!("Exported '{}' to {}", col.name, file_path));
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Export failed: {}", e));
            }
        }
    }
```

- [ ] **Step 2: Add `:export` command parsing in `src/tui/input/popups.rs`**

In `handle_command_mode`:
```rust
} else if let Some(args) = cmd.strip_prefix("export ") {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() == 1 {
        app.export_active_collection(parts[0], None);
    } else if parts.len() >= 2 {
        app.export_active_collection(parts[0], Some(parts[1]));
    }
```

- [ ] **Step 3: Update command help popup in `src/tui/ui/widgets/popups.rs`**

Add `:export <format> <path>` description to command help popup:
```rust
Span::styled("  :export [format] <path> ", Style::default().fg(Color::Cyan)),
Span::raw(": Export current collection to Postman or OpenAPI JSON"),
```

- [ ] **Step 4: Verify build and run all tests**

Run: `cargo test`
Expected: All tests pass cleanly.

- [ ] **Step 5: Commit**

```bash
git add src/tui/app/handlers/mod.rs src/tui/input/popups.rs src/tui/ui/widgets/popups.rs
git commit -m "feat: add TUI :export command handling"
```
