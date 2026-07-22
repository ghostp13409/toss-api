use boa_engine::{Context, Source};
use crate::core::collection::KVParam;
use serde::{Deserialize, Serialize};

pub struct ScriptExecutionResult {
    pub console_logs: Vec<ConsoleLog>,
    pub test_results: Vec<TestResult>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ConsoleLog {
    pub level: String,
    pub message: String,
}

pub fn execute_pre_request_script(
    script: &str,
    env_vars: &mut Vec<KVParam>,
    headers: &mut Vec<KVParam>,
    url: &mut String,
) -> Result<ScriptExecutionResult, String> {
    let mut context = Context::default();

    if let Err(e) = crate::core::scripting::crypto::register_crypto(&mut context) {
        return Err(e.to_string());
    }

    // Create JSON strings for current state
    let env_json = serde_json::to_string(env_vars).unwrap_or_else(|_| "[]".to_string());
    let headers_json = serde_json::to_string(headers).unwrap_or_else(|_| "[]".to_string());
    let url_json = serde_json::to_string(url).unwrap_or_else(|_| "\"\"".to_string());

    let shim = format!(r#"
        const __test_results = [];
        {}
        {}
    "#, 
        crate::core::scripting::console::get_console_shim(),
        crate::core::scripting::context::get_client_shim(&env_json, &headers_json, &url_json)
    );

    context.eval(Source::from_bytes(&shim)).map_err(|e| e.to_string())?;

    // Evaluate user script
    if let Err(e) = context.eval(Source::from_bytes(script)) {
        return Err(e.to_string());
    }

    // Extract updated state
    let extract_script = r#"
        JSON.stringify({
            env: Object.values(__env_vars),
            headers: Object.values(__headers),
            url: client.request.url,
            logs: __logs,
            tests: __test_results
        })
    "#;

    let result = context.eval(Source::from_bytes(extract_script)).map_err(|e| e.to_string())?;
    
    let result_json = result.to_string(&mut context).map_err(|e| e.to_string())?.to_std_string().map_err(|e| e.to_string())?;
    
    #[derive(Deserialize)]
    struct ExtractedState {
        env: Vec<KVParam>,
        headers: Vec<KVParam>,
        url: String,
        logs: Vec<ConsoleLog>,
        tests: Vec<TestResult>,
    }

    let state: ExtractedState = serde_json::from_str(&result_json).map_err(|e| e.to_string())?;

    *env_vars = state.env;
    *headers = state.headers;
    *url = state.url;

    Ok(ScriptExecutionResult {
        console_logs: state.logs,
        test_results: state.tests,
    })
}

pub fn execute_post_response_script(
    script: &str,
    env_vars: &mut Vec<KVParam>,
    response_status: u16,
    response_status_text: &str,
    response_headers: &[KVParam],
    response_body: &str,
) -> Result<ScriptExecutionResult, String> {
    let mut context = Context::default();

    if let Err(e) = crate::core::scripting::crypto::register_crypto(&mut context) {
        return Err(e.to_string());
    }

    let env_json = serde_json::to_string(env_vars).unwrap_or_else(|_| "[]".to_string());
    let res_headers_json = serde_json::to_string(response_headers).unwrap_or_else(|_| "[]".to_string());
    let res_body_json = serde_json::to_string(response_body).unwrap_or_else(|_| "\"\"".to_string());
    let res_status_text_json = serde_json::to_string(response_status_text).unwrap_or_else(|_| "\"\"".to_string());

    let shim = format!(r#"
        const __test_results = [];
        {}
        {}
        
        const response = {{
            status: {},
            statusText: {},
            headers: {},
            text: function() {{ return {}; }},
            json: function() {{ return JSON.parse({}); }}
        }};
        client.response = response;
    "#, 
        crate::core::scripting::console::get_console_shim(),
        crate::core::scripting::context::get_client_shim(&env_json, "[]", "\"\""),
        response_status,
        res_status_text_json,
        res_headers_json,
        res_body_json,
        res_body_json
    );

    context.eval(Source::from_bytes(&shim)).map_err(|e| e.to_string())?;

    if let Err(e) = context.eval(Source::from_bytes(script)) {
        return Err(e.to_string());
    }

    let extract_script = r#"
        JSON.stringify({
            env: Object.values(__env_vars),
            logs: __logs,
            tests: __test_results
        })
    "#;

    let result = context.eval(Source::from_bytes(extract_script)).map_err(|e| e.to_string())?;
    
    let result_json = result.to_string(&mut context).map_err(|e| e.to_string())?.to_std_string().map_err(|e| e.to_string())?;
    
    #[derive(Deserialize)]
    struct ExtractedState {
        env: Vec<KVParam>,
        logs: Vec<ConsoleLog>,
        tests: Vec<TestResult>,
    }

    let state: ExtractedState = serde_json::from_str(&result_json).map_err(|e| e.to_string())?;

    *env_vars = state.env;

    Ok(ScriptExecutionResult {
        console_logs: state.logs,
        test_results: state.tests,
    })
}
