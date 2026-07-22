# Scripting Engine Design Spec

## Overview
This feature introduces a full-featured JavaScript Scripting Engine to `toss-api` powered by `boa_engine`. It enables users to run **Pre-Request Scripts** (modifying URLs, headers, query params, request bodies, dynamic signatures, and environment variables before sending HTTP requests) and **Post-Response / Test Scripts** (running assertions, parsing response JSON, setting environment variables from response data, and logging messages) in both TUI and CLI modes.

## Architecture & Core Modules

A new module `src/core/scripting/` will be created with:
- `src/core/scripting/mod.rs`: Public entry points (`execute_pre_request_script`, `execute_post_response_script`), execution results struct (`ScriptExecutionResult`), and test result struct (`TestResult`).
- `src/core/scripting/engine.rs`: Wraps `boa_engine::Context`, managing global environment variables, request mutation, response state, and execution timeouts (5000ms limit).
- `src/core/scripting/context.rs`: Implements JavaScript global bindings for `client` and alias `pm` (`client.environment`, `client.globals`, `client.variables`, `client.request`, `response`, `client.test`, `client.expect`).
- `src/core/scripting/crypto.rs`: Built-in utilities for `Crypto.md5`, `Crypto.sha256`, `Crypto.hmacSha256`, `Crypto.base64Encode`, `Crypto.base64Decode`, `uuid()`, and `timestamp()`.
- `src/core/scripting/console.rs`: Captures `console.log`, `console.warn`, and `console.error` messages.

### JavaScript API Surface

#### 1. Variable Scope Manipulation
```javascript
// Environment variables
client.environment.get("key");
client.environment.set("key", "value");
client.environment.unset("key");

// Global variables
client.globals.get("key");
client.globals.set("key", "value");
client.globals.unset("key");

// Collection / Request variables
client.variables.get("key");
client.variables.set("key", "value");
```
Alias `pm` is available (`pm.environment.set(...)`, `pm.globals.get(...)`, etc.).

#### 2. Request Object Mutation (Pre-Request)
```javascript
client.request.url;                          // read / write
client.request.method;                       // read / write
client.request.headers.get("Header-Name");
client.request.headers.set("Header-Name", "value");
client.request.headers.add("Header-Name", "value");
client.request.headers.remove("Header-Name");
client.request.body;                         // read / write raw content
```

#### 3. Response Object Access & Assertions (Post-Response / Tests)
```javascript
response.status;               // e.g. 200
response.statusText;           // e.g. "OK"
response.headers.get("Content-Type");
response.text();               // returns raw response body string
response.json();               // parses and returns JSON object

client.test("Status code is 200", function() {
    client.expect(response.status).to.equal(200);
});

client.test("Response body has id", function() {
    let data = response.json();
    client.expect(data.id).to.exist();
});
```

#### 4. Cryptography & Utilities
```javascript
Crypto.md5("string");
Crypto.sha256("string");
Crypto.hmacSha256("message", "secret");
Crypto.base64Encode("string");
Crypto.base64Decode("encoded_string");

let id = uuid();             // UUID v4 string
let ts = timestamp();        // Unix timestamp ms
```

#### 5. Logging & Debugging
```javascript
console.log("Informational message", data);
console.warn("Warning message");
console.error("Error message");
```

## TUI Workflows & UI Components

1. **Property Details Scripts Tab**:
   - Tab **`Scripts`** (shortcut `S` in details panel).
   - Sub-toggle between `Pre-Request` and `Tests` (Post-Response) script text editors.
2. **Response Inspector Test Results Tab**:
   - Tab **`Tests`** in Response inspector.
   - Displays summary `PASS: X / Y` and itemized list of pass/fail assertions.
3. **Console Log Viewer Modal**:
   - Command mode `:console` or keybinding `Shift + S`.
   - Opens scrollable console log overlay showing `console.log/warn/error` entries.
4. **Script Presets**:
   - Code snippet insert snippets for quick boilerplate (e.g. status code 200, set env var, HMAC signature).

## Documentation Deliverable
- Create `docs/scripting-guide.md`: A user-facing, comprehensive guide covering all scripting features, JS API reference, TUI & CLI workflows, and copy-pasteable script examples.

## Sample Collections
- Update sample collection files in `src/samples/` to include pre-request and post-response test scripts covering environment manipulation, HMAC signature headers, assertions, and logging.
