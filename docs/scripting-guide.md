# Scripting Engine & Automation Guide

The **Toss API Scripting Engine** allows you to automate workflows, dynamically modify requests before dispatch, and write tests to validate HTTP responses. Built on an embedded JavaScript runtime (boa engine), Toss API provides full compatibility with Postman-style scripting conventions using `client` and `pm` global objects, built-in crypto helpers, console logging, and BDD-style assertion libraries.

---

## Table of Contents

1. [Overview & Execution Lifecycle](#1-overview--execution-lifecycle)
2. [Pre-Request vs Post-Response (Tests) Scripts](#2-pre-request-vs-post-response-tests-scripts)
3. [JavaScript API Reference](#3-javascript-api-reference)
   - [Globals: `client` and `pm`](#globals-client-and-pm)
   - [Environment & Variable Scopes](#environment--variable-scopes)
   - [Request Object & Header Mutation](#request-object--header-mutation)
   - [Response Object & Parsing](#response-object--parsing)
   - [Testing & Assertions (`client.test`, `client.expect`)](#testing--assertions-clienttest-clientexpect)
   - [Crypto & Encoding (`Crypto.*`)](#crypto--encoding-crypto)
   - [Utility Functions (`uuid()`, `timestamp()`)](#utility-functions-uuid-timestamp)
   - [Console Logging (`console.log`, `warn`, `error`)](#console-logging-consolelog-warn-error)
4. [TUI Workflows & Keyboard Shortcuts](#4-tui-workflows--keyboard-shortcuts)
5. [CLI Usage & Execution](#5-cli-usage--execution)
6. [Copy & Paste Script Snippets](#6-copy--paste-script-snippets)

---

## 1. Overview & Execution Lifecycle

When sending a request in Toss API, the execution follows a deterministic 3-stage pipeline:

```
┌───────────────────────────┐
│  1. Pre-Request Script    │  ➜ Mutates headers, environment, URL
└─────────────┬─────────────┘
              │
              ▼
┌───────────────────────────┐
│  2. HTTP Request Dispatch │  ➜ Sends request to remote server
└─────────────┬─────────────┘
              │
              ▼
┌───────────────────────────┐
│  3. Post-Response Script  │  ➜ Executes assertions, parses response JSON,
└───────────────────────────┘    updates variables for subsequent requests
```

1. **Pre-Request Script:** Executes in JS runtime before network dispatch. Can alter request headers, URL parameters, and set/update environment variables.
2. **HTTP Dispatch:** Requests are dispatched with any headers or URL modifications injected by the pre-request script.
3. **Post-Response Script:** Executes after response headers and body are received. Runs test assertions, extracts tokens, updates variables, and logs warnings/errors.

---

## 2. Pre-Request vs Post-Response (Tests) Scripts

| Feature | Pre-Request Script | Post-Response (Tests) Script |
| :--- | :--- | :--- |
| **Execution Moment** | Before HTTP request is sent | After HTTP response is received |
| **Primary Use Cases** | Token generation, timestamping, HMAC signing, dynamic header injection | Status code checks, JSON schema validation, environment updates, response assertions |
| **Available Objects** | `client`, `pm`, `request`, `environment`, `globals`, `Crypto`, `uuid`, `timestamp`, `console` | `client`, `pm`, `request`, `response`, `environment`, `globals`, `Crypto`, `uuid`, `timestamp`, `console` |
| **Response Access** | ❌ Not available | ✅ Fully accessible via `response` or `pm.response` |

---

## 3. JavaScript API Reference

### Globals: `client` and `pm`

Both `client` and `pm` are exposed as identical global objects for max compatibility with Postman scripts and IntelliJ HTTP Client scripts.

```javascript
client.environment.set("auth_token", "xyz");
pm.environment.set("auth_token", "xyz"); // Equivalent
```

---

### Environment & Variable Scopes

Manipulate environment variables dynamically across requests.

#### `client.environment` / `pm.environment`
- **`get(key: string): string | null`**: Returns the value of an environment variable.
- **`set(key: string, value: any): void`**: Sets or updates an environment variable (coerced to string).
- **`unset(key: string): void`**: Removes an environment variable.

#### `client.globals` / `pm.globals`
- **`get(key: string): string | null`**
- **`set(key: string, value: any): void`**
- **`unset(key: string): void`**

#### `client.variables` / `pm.variables`
- **`get(key: string): string | null`**: Resolves variable looking in environment first, then globals.
- **`set(key: string, value: any): void`**: Sets in active scope.

```javascript
// Example
let token = client.environment.get("access_token");
client.environment.set("last_login", timestamp());
client.globals.set("api_version", "v2");
```

---

### Request Object & Header Mutation

Pre-request scripts can read and mutate outgoing headers.

```javascript
// Access request URL and method
console.log(client.request.url);     // "https://httpbin.org/get"
console.log(client.request.method);  // "GET"

// Add or override request headers directly
client.request.headers["X-Trace-ID"] = uuid();
client.request.headers["Authorization"] = "Bearer " + client.environment.get("token");
```

---

### Response Object & Parsing

Post-response scripts can inspect status, headers, and parse response bodies.

```javascript
// Response status and status text
console.log(response.status);      // e.g. 200
console.log(response.statusText);  // e.g. "OK"
console.log(pm.response.code);     // Alias for status

// Response headers
console.log(response.headers["content-type"]);

// Parse JSON body
let body = response.json(); // Returns parsed JS object
console.log(body.user.id);

// Raw text body
let raw = response.text();
```

---

### Testing & Assertions (`client.test`, `client.expect`)

Toss API includes a BDD assertion library for writing post-response test suites.

#### `client.test(name: string, callback: function)` / `pm.test(name, fn)`
Executes a named test block. If any assertion inside fails or throws an exception, the test is marked as failed.

#### `client.expect(value)` / `pm.expect(value)`
Chainable assertion engine supporting:
- `.to.be(expected)` / `.to.equal(expected)`
- `.to.not.be(expected)` / `.to.not.equal(expected)`
- `.to.have.property(propName)`
- `.to.include(substringOrItem)`
- `.to.exist`

```javascript
client.test("Status code is 200 OK", function() {
    client.expect(response.status).to.be(200);
});

client.test("Response body contains user object", function() {
    let data = response.json();
    client.expect(data).to.have.property("user");
    client.expect(data.user.email).to.include("@");
});
```

---

### Crypto & Encoding (`Crypto.*`)

Perform hashing, HMAC signatures, and base64 encoding without external npm dependencies.

| Method | Parameters | Returns | Description |
| :--- | :--- | :--- | :--- |
| `Crypto.md5(data)` | `data: string` | `string` | Hex-encoded MD5 hash |
| `Crypto.sha1(data)` | `data: string` | `string` | Hex-encoded SHA1 hash |
| `Crypto.sha256(data)` | `data: string` | `string` | Hex-encoded SHA256 hash |
| `Crypto.sha512(data)` | `data: string` | `string` | Hex-encoded SHA512 hash |
| `Crypto.hmacSha256(key, message)` | `key: string`, `message: string` | `string` | Hex-encoded HMAC SHA256 signature |
| `Crypto.base64Encode(data)` | `data: string` | `string` | Base64 string |
| `Crypto.base64Decode(encoded)` | `encoded: string` | `string` | Decoded string |

```javascript
// Calculate HMAC SHA256 signature for API authentication
let secret = "my_api_secret";
let payload = "POST&/v1/orders&" + timestamp();
let signature = Crypto.hmacSha256(secret, payload);

client.request.headers["X-Signature"] = signature;
```

---

### Utility Functions (`uuid()`, `timestamp()`)

- **`uuid()`**: Returns a fresh UUID v4 string (e.g. `"f47ac10b-58cc-4372-a567-0e02b2c3d479"`).
- **`timestamp()`**: Returns current Unix timestamp in seconds (integer).

```javascript
let reqId = uuid();
let ts = timestamp();
```

---

### Console Logging (`console.log`, `warn`, `error`)

Scripts can emit log messages captured in the Toss API Console overlay.

```javascript
console.log("Processing user request:", userId);
console.warn("Token expires soon, refreshing...");
console.error("Failed to parse response body!");
```

---

## 4. TUI Workflows & Keyboard Shortcuts

Toss API provides an integrated script editor and live console viewer within the terminal interface.

```
┌────────────────────────────────────────────────────────┐
│ Request Details: [Params] [Headers] [Body] [Scripts]   │
│                                                        │
│ Subtabs: [ (1) Pre-Request ]   [ (2) Post-Response ] │
│                                                        │
│  1  console.log("Preparing request");                  │
│  2  let token = uuid();                                │
│  3  client.environment.set("auth_token", token);       │
└────────────────────────────────────────────────────────┘
```

### Keyboard Shortcuts

| Shortcut | Scope / Context | Action |
| :--- | :--- | :--- |
| **`S`** | Normal Mode | Navigate directly to the **Scripts** tab in Request Details |
| **`t`** | Scripts Tab | Toggle subtab between **Pre-Request** script and **Post-Response** script |
| **`Tab`** / **`Shift+Tab`** | Normal Mode | Cycle forward / backward through Request Details tabs |
| **`T`** | Normal Mode | View **Test Results** tab in Response Panel after request execution |
| **`Shift+S`** or **`:console`** | Normal Mode | Toggle the **Console Logs Overlay** to inspect script `console.log/warn/error` output |
| **`i`** / **`Esc`** | Script Editor | Enter inline edit mode / Exit edit mode back to normal navigation |

---

## 5. CLI Usage & Execution

When running collection tests via CLI (`toss run collection.json`), Toss API automatically executes pre-request and post-response scripts for each request item.

```bash
# Run collection with script execution enabled
toss run src/samples/httpbin.json --env env.json
```

### Sample CLI Test Summary Output

```
Running collection: Httpbin Comprehensive Sample
[PASS] GET Methods > Detailed GET
       ✓ Status code is 200 (1ms)
       ✓ Response contains expected query params (0ms)
[PASS] POST Methods > POST JSON Body
       ✓ Status code is 200 OK (0ms)
       ✓ JSON body user name match (1ms)

Test Summary: 4 passed, 0 failed.
```

---

## 6. Copy & Paste Script Snippets

### Snippet 1: Pre-Request HMAC SHA256 Signature Header

```javascript
// Pre-Request Script: Sign request with HMAC SHA256
let apiSecret = client.environment.get("API_SECRET") || "default_secret";
let currentTs = timestamp();

client.environment.set("req_timestamp", String(currentTs));
client.request.headers["X-Timestamp"] = String(currentTs);

let signature = Crypto.hmacSha256(apiSecret, client.request.method + ":" + currentTs);
client.request.headers["X-Signature"] = signature;
console.log("Injected HMAC signature:", signature);
```

### Snippet 2: Post-Response Token Extraction & Environment Mutation

```javascript
// Post-Response Script: Save authentication token for subsequent requests
client.test("Login status is 200", function() {
    client.expect(response.status).to.be(200);
});

let body = response.json();
if (body && body.token) {
    client.environment.set("auth_token", body.token);
    console.log("Updated environment variable 'auth_token'");
} else {
    console.warn("No token returned in response body");
}
```

### Snippet 3: Validating JSON Response Structure and Fields

```javascript
// Post-Response Script: Comprehensive API testing
client.test("Response is successful JSON", function() {
    client.expect(response.status).to.be(200);
    client.expect(response.headers["content-type"]).to.include("application/json");
});

client.test("Payload fields validation", function() {
    let json = response.json();
    client.expect(json).to.have.property("status");
    client.expect(json.status).to.be("active");
    client.expect(json).to.have.property("id");
});
```

---
