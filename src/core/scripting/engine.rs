use boa_engine::{Context, Source};
use crate::core::collection::KVParam;
use serde::{Deserialize, Serialize};

pub struct ScriptExecutionResult {
    pub console_logs: Vec<ConsoleLog>,
    pub test_results: Vec<TestResult>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConsoleLog {
    pub level: String,
    pub message: String,
}

pub fn execute_pre_request_script(
    script: &str,
    env_vars: &mut Vec<KVParam>,
    headers: &mut Vec<KVParam>,
    _url: &mut String,
) -> Result<ScriptExecutionResult, String> {
    let mut context = Context::default();

    // Create JSON strings for current state
    let env_json = serde_json::to_string(env_vars).unwrap_or_else(|_| "[]".to_string());
    let headers_json = serde_json::to_string(headers).unwrap_or_else(|_| "[]".to_string());

    let shim = format!(r#"
        const __test_results = [];
        {}
        {}
        {}
    "#, 
        crate::core::scripting::console::get_console_shim(),
        crate::core::scripting::crypto::get_crypto_shim(),
        crate::core::scripting::context::get_client_shim(&env_json, &headers_json)
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
            logs: __logs,
            tests: __test_results
        })
    "#;

    let result = context.eval(Source::from_bytes(extract_script)).map_err(|e| e.to_string())?;
    
    let result_json = result.to_string(&mut context).map_err(|e| e.to_string())?.to_std_string_escaped();
    
    #[derive(Deserialize)]
    struct ExtractedState {
        env: Vec<KVParam>,
        headers: Vec<KVParam>,
        logs: Vec<ConsoleLog>,
        tests: Vec<TestResult>,
    }

    let state: ExtractedState = serde_json::from_str(&result_json).map_err(|e| e.to_string())?;

    *env_vars = state.env;
    *headers = state.headers;

    Ok(ScriptExecutionResult {
        console_logs: state.logs,
        test_results: state.tests,
    })
}

pub fn execute_post_response_script(
    _script: &str,
    _env_vars: &mut Vec<KVParam>,
) -> Result<ScriptExecutionResult, String> {
    Ok(ScriptExecutionResult {
        console_logs: vec![],
        test_results: vec![],
    })
}
