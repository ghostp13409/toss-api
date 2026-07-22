# Scripting Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a full JavaScript Scripting Engine in `toss-api` for Pre-Request scripts and Post-Response / Test scripts with crypto utilities, assertions, TUI views, console debugger, sample collections, and user guide.

**Architecture:** Use `boa_engine` in `src/core/scripting/` to expose `client` and `pm` globals. Hook pre-request execution before HTTP dispatch and post-response execution after HTTP response in `src/engine/http.rs` and TUI/CLI handlers. Add TUI `Scripts` tab, `Tests` tab, `:console` popup, sample scripts, and `docs/scripting-guide.md`.

**Tech Stack:** Rust, `boa_engine` (v0.20), `serde` / `serde_json`, `ratatui`, `reqwest`, `uuid`, `sha2`/`md-5`/`hmac`.

## Global Constraints

- JS Engine: `boa_engine` = "0.20".
- JS API: `client` and alias `pm` (`client.environment.get/set/unset`, `client.globals.get/set/unset`, `client.variables.get/set`, `client.request`, `response`, `client.test`, `client.expect`, `Crypto.sha256/hmacSha256/md5/base64Encode/base64Decode`, `uuid()`, `timestamp()`, `console.log/warn/error`).
- Execution Timeout: 5000ms limit.
- Documentation: User-facing `docs/scripting-guide.md`.

---

### Task 1: Core Script Engine Module (`src/core/scripting/`)

**Files:**
- Modify: `Cargo.toml`
- Create: `src/core/scripting/mod.rs`
- Create: `src/core/scripting/engine.rs`
- Create: `src/core/scripting/context.rs`
- Create: `src/core/scripting/crypto.rs`
- Create: `src/core/scripting/console.rs`
- Modify: `src/core/mod.rs`

**Interfaces:**
- Consumes: `Collection`, `Request`, `KVParam`, `Environment` from `crate::core`.
- Produces: `execute_pre_request_script(...)`, `execute_post_response_script(...)`, `ScriptExecutionResult`, `TestResult`, `ConsoleLog`.

- [ ] **Step 1: Add `boa_engine = "0.20"` dependency to `Cargo.toml`**

Update `Cargo.toml`:
```toml
boa_engine = "0.20"
```

- [ ] **Step 2: Write failing unit test for Scripting Engine**

Create `src/core/scripting/mod.rs` with a failing test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_request_script_variable_and_header_mutation() {
        let script = r#"
            client.environment.set("token", "secret123");
            client.request.headers.add("Authorization", "Bearer " + client.environment.get("token"));
        "#;

        let mut env_vars = Vec::new();
        let mut headers = Vec::new();
        let mut url = "https://httpbin.org/get".to_string();

        let res = execute_pre_request_script(script, &mut env_vars, &mut headers, &mut url)
            .expect("script execution should succeed");

        assert_eq!(env_vars.iter().find(|v| v.key == "token").map(|v| v.value.as_str()), Some("secret123"));
        assert_eq!(headers.iter().find(|h| h.key == "Authorization").map(|h| h.value.as_str()), Some("Bearer secret123"));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test test_pre_request_script_variable_and_header_mutation`
Expected: FAIL due to missing `execute_pre_request_script`.

- [ ] **Step 4: Implement `src/core/scripting/` modules**

Implement JS runtime bindings in `src/core/scripting/engine.rs`, `context.rs`, `crypto.rs`, `console.rs`, and `mod.rs`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/core/scripting src/core/mod.rs
git commit -m "feat: implement core scripting engine using boa_engine"
```

---

### Task 2: HTTP Execution Pipeline Integration

**Files:**
- Modify: `src/engine/http.rs`
- Modify: `src/cli/mod.rs`
- Modify: `src/tui/app/handlers/mod.rs`

**Interfaces:**
- Consumes: `execute_pre_request_script`, `execute_post_response_script` from `crate::core::scripting`.
- Produces: Execution of scripts before/after HTTP requests in CLI and TUI, storing `ScriptExecutionResult` on response state.

- [ ] **Step 1: Write failing unit test for script pipeline execution**

Add a test verifying script execution around HTTP request calls.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test`

- [ ] **Step 3: Implement pipeline execution in `src/engine/http.rs` and handlers**

- Hook pre-request script execution before sending request.
- Hook post-response script execution after receiving response.
- Attach `test_results` and `console_logs` to response state.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test`

- [ ] **Step 5: Commit**

```bash
git add src/engine/http.rs src/cli/mod.rs src/tui/app/handlers/mod.rs
git commit -m "feat: integrate scripting engine into request pipeline"
```

---

### Task 3: TUI Views & Commands (`Scripts` tab, `Tests` tab, `:console` overlay)

**Files:**
- Modify: `src/tui/app/enums.rs`
- Modify: `src/tui/app/state.rs`
- Modify: `src/tui/ui/widgets/details.rs`
- Modify: `src/tui/ui/widgets/response.rs`
- Modify: `src/tui/ui/widgets/popups.rs`
- Modify: `src/tui/input/popups.rs`

**Interfaces:**
- Consumes: `test_results` and `console_logs` from response state.
- Produces: `Scripts` tab in Details panel, `Tests` tab in Response panel, `:console` command mode popup, shortcut `Shift+S`.

- [ ] **Step 1: Write failing unit test for console overlay and script sub-tabs**

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement TUI rendering & inputs for Scripts, Tests, and Console popup**

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test`

- [ ] **Step 5: Commit**

```bash
git add src/tui/
git commit -m "feat: add TUI Scripts tab, Tests response tab, and console debugger popup"
```

---

### Task 4: Sample Collections & User Documentation Guide

**Files:**
- Modify: `src/samples/httpbin.json`
- Modify: `src/samples/petstore.json`
- Create: `docs/scripting-guide.md`
- Modify: `README.md`

**Interfaces:**
- Produces: User-facing `docs/scripting-guide.md` documentation covering all JS API methods, TUI & CLI workflows, code snippets, and sample scripts in `src/samples/`.

- [ ] **Step 1: Add pre-request and post-response test scripts to `src/samples/httpbin.json` and `petstore.json`**

Include dynamic tokens, HMAC signatures, JSON response assertions, and `console.log` statements.

- [ ] **Step 2: Create `docs/scripting-guide.md`**

Write a clear, beginner-friendly, comprehensive user guide covering:
- Overview of Scripting in Toss API
- Pre-Request vs Post-Response (Tests) Scripts
- Full JavaScript API Reference (`client.environment`, `client.globals`, `client.request`, `response`, `client.test`, `client.expect`, `Crypto.*`, `uuid()`, `timestamp()`, `console.log`)
- TUI Workflows & Keyboard Shortcuts (Details Scripts tab `S`, Test Results tab, Console overlay `:console` / `Shift+S`)
- CLI Usage
- Complete Copy & Paste Code Examples

- [ ] **Step 3: Update `README.md`**

Mark Scripting as completed in `README.md` roadmap.

- [ ] **Step 4: Run tests and build**

Run: `cargo test` and `cargo build`

- [ ] **Step 5: Commit**

```bash
git add src/samples docs/scripting-guide.md README.md
git commit -m "docs: add comprehensive scripting guide and sample collection scripts"
```
