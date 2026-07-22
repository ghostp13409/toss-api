pub mod context;
pub mod engine;
pub mod crypto;
pub mod console;

pub use engine::{execute_pre_request_script, execute_post_response_script, ScriptExecutionResult, TestResult, ConsoleLog};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collection::KVParam;

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
