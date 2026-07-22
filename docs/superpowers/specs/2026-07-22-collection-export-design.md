# Collection Export Feature Design Spec

## Overview
This feature adds the ability to export saved API collections from `toss-api` into Postman Collection v2.1 format and OpenAPI 3.0.3 format JSON files, via both the CLI interface (`toss-api export ...`) and the TUI command mode (`:export ...`).

## Architecture & Modules

A new export module will be added under `src/core/export/`:

- `src/core/export/mod.rs`: Defines `ExportFormat` enum and public `export_collection` entry point.
- `src/core/export/postman.rs`: Converts a `Collection` into a Postman Collection v2.1 JSON string.
- `src/core/export/openapi.rs`: Converts a `Collection` into an OpenAPI 3.0.3 JSON string.

### Data Mapping Details

#### 1. Postman Collection v2.1 (`postman.rs`)
- `info`: Maps collection name, UUID id, and schema URL (`https://schema.getpostman.com/json/collection/v2.1.0/collection.json`).
- `item`: Recursively converts `CollectionItem::Folder` to `{ "name": "...", "item": [...] }` and `CollectionItem::Request` to `{ "name": "...", "request": { ... } }`.
- `request`:
  - `method`: HTTP method string (`GET`, `POST`, etc.).
  - `header`: List of `{ "key": ..., "value": ..., "disabled": bool, "description": ... }`.
  - `url`: `{ "raw": ..., "host": [...], "path": [...], "query": [...] }`.
  - `auth`: Postman auth object for `Bearer`, `Basic`, and `ApiKey` types.
  - `body`: Postman body object supporting `raw` (with `mode: "raw"`, `options: { "raw": { "language": "json" } }`), `formdata`, and `urlencoded`.

#### 2. OpenAPI 3.0.3 (`openapi.rs`)
- `openapi`: Set to `"3.0.3"`.
- `info`: Set to `{ "title": collection.name, "version": "1.0.0" }`.
- `paths`: Grouped by URL path string (normalizing path parameters like `:id` or `{{id}}` into `{id}`).
- Each operation (`get`, `post`, etc.):
  - `summary`: Request name.
  - `parameters`: Array of path/query/header parameters (`in: "query"`, `in: "header"`, `in: "path"`).
  - `requestBody`: Generated for `POST`/`PUT`/`PATCH` when request body exists (mapped to `application/json`, `multipart/form-data`, or `application/x-www-form-urlencoded`).
  - `responses`: Standard `200` response object (`{ "description": "Successful response" }`).
- `components.securitySchemes`: Defines `bearerAuth`, `basicAuth`, and `apiKeyAuth` schemes when present.

## Interfaces

### 1. CLI Subcommand (`src/cli/args.rs` & `src/main.rs`)
```rust
Export {
    /// Name of the collection to export
    name: String,

    /// Format to export: postman (default) or openapi
    #[arg(short, long, default_value = "postman")]
    format: String,

    /// Output file path
    #[arg(short, long)]
    output: Option<String>,
}
```
* CLI usage examples:
  - `toss-api export "Petstore"` -> Exports Petstore to `Petstore.postman_collection.json`.
  - `toss-api export "Petstore" -f openapi -o petstore.json` -> Exports Petstore to `petstore.json` in OpenAPI 3.0 format.

### 2. TUI Command Mode (`src/tui/input/popups.rs` & `src/tui/app/handlers/mod.rs`)
* Handle `:export <format> <path>` or `:export <path>` in command mode.
* Example: `:export openapi my_api.json` or `:export postman_collection.json`.
* Displays status notification on success or error popup on failure.

## Testing & Verification Strategy
- **Unit Tests**:
  - `export_postman_collection_test`: Tests exporting a collection with folders, headers, auth, and body, then validating JSON format.
  - `export_openapi_collection_test`: Tests exporting a collection to OpenAPI 3.0 JSON, verifying `openapi`, `paths`, and `components`.
  - CLI execution tests verifying exported file contents.
