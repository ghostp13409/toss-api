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
            client.request.url = client.request.url + "?query=1";
            pm.globals.set("global_token", "g123");
            console.log("Pre-request done");
        "#;

        let mut env_vars = Vec::new();
        let mut headers = Vec::new();
        let mut url = "https://httpbin.org/get".to_string();

        let res = execute_pre_request_script(script, &mut env_vars, &mut headers, &mut url)
            .expect("script execution should succeed");

        assert_eq!(env_vars.iter().find(|v| v.key == "token").map(|v| v.value.as_str()), Some("secret123"));
        assert_eq!(env_vars.iter().find(|v| v.key == "global_token").map(|v| v.value.as_str()), Some("g123"));
        assert_eq!(headers.iter().find(|h| h.key == "Authorization").map(|h| h.value.as_str()), Some("Bearer secret123"));
        assert_eq!(url, "https://httpbin.org/get?query=1");
        assert_eq!(res.console_logs.len(), 1);
        assert_eq!(res.console_logs[0].message, "Pre-request done");
    }

    #[test]
    fn test_post_response_script_and_assertions() {
        let script = r#"
            pm.test("Status code is 200", function () {
                pm.expect(response.status).to.equal(200);
            });
            pm.test("Response contains data", function () {
                var jsonData = response.json();
                pm.expect(jsonData.success).to.equal(true);
            });
            console.error("Post-response error log");
        "#;

        let mut env_vars = Vec::new();
        let response_headers = vec![];
        let response_body = r#"{"success": true}"#;

        let res = execute_post_response_script(
            script,
            &mut env_vars,
            200,
            "OK",
            &response_headers,
            response_body,
        ).expect("script execution should succeed");

        assert_eq!(res.test_results.len(), 2);
        assert_eq!(res.test_results[0].name, "Status code is 200");
        assert_eq!(res.test_results[0].passed, true);
        assert_eq!(res.test_results[1].name, "Response contains data");
        assert_eq!(res.test_results[1].passed, true);

        assert_eq!(res.console_logs.len(), 1);
        assert_eq!(res.console_logs[0].level, "error");
        assert_eq!(res.console_logs[0].message, "Post-response error log");
    }

    #[test]
    fn test_pm_response_json() {
        let script = r#"
            pm.test("pm.response.status is 200", function () {
                pm.expect(pm.response.status).to.equal(200);
            });
            pm.test("pm.response.json() works", function () {
                var jsonData = pm.response.json();
                pm.expect(jsonData.foo).to.equal("bar");
            });
            pm.test("client.response.json() works", function () {
                var jsonData = client.response.json();
                pm.expect(jsonData.foo).to.equal("bar");
            });
        "#;

        let mut env_vars = Vec::new();
        let response_headers = vec![];
        let response_body = r#"{"foo": "bar"}"#;

        let res = execute_post_response_script(
            script,
            &mut env_vars,
            200,
            "OK",
            &response_headers,
            response_body,
        ).expect("script execution should succeed");

        assert_eq!(res.test_results.len(), 3);
        for t in &res.test_results {
            assert_eq!(t.passed, true, "Test failed: {}", t.name);
        }
    }

    #[test]
    fn test_crypto_uuid_timestamp() {
        let script = r#"
            var hash = Crypto.sha256("test");
            var hmac = Crypto.hmacSha256("test", "secret");
            var b64e = Crypto.base64Encode("hello");
            var b64d = Crypto.base64Decode(b64e);
            var id = uuid();
            var ts = timestamp();
            
            pm.test("Hash is string", function() { pm.expect(typeof hash).to.equal("string"); });
            pm.test("Hmac is string", function() { pm.expect(typeof hmac).to.equal("string"); });
            pm.test("B64 matches", function() { pm.expect(b64d).to.equal("hello"); });
            pm.test("UUID works", function() { pm.expect(id.length).to.equal(36); });
            pm.test("Timestamp works", function() { pm.expect(typeof ts).to.equal("number"); });
        "#;

        let mut env_vars = Vec::new();
        let mut headers = Vec::new();
        let mut url = "https://httpbin.org/get".to_string();

        let res = execute_pre_request_script(script, &mut env_vars, &mut headers, &mut url)
            .expect("script execution should succeed");

        assert_eq!(res.test_results.len(), 5);
        for t in &res.test_results {
            assert_eq!(t.passed, true, "Test failed: {}", t.name);
        }
    }

    #[test]
    fn test_utf8_non_ascii_support() {
        let script = r#"
            client.environment.set("city", "München");
            client.environment.set("cafe", "café");
            var hash = Crypto.sha256("café");
            console.log("Welcome to café");
            pm.test("Crypto sha256 non-ascii", function() {
                pm.expect(hash).to.equal("850f7dc43910ff890f8879c0ed26fe697c93a067ad93a7d50f466a7028a9bf4e");
            });
        "#;

        let mut env_vars = Vec::new();
        let mut headers = Vec::new();
        let mut url = "https://httpbin.org/get".to_string();

        let res = execute_pre_request_script(script, &mut env_vars, &mut headers, &mut url)
            .expect("script execution should succeed");

        assert_eq!(env_vars.iter().find(|v| v.key == "city").map(|v| v.value.as_str()), Some("München"));
        assert_eq!(env_vars.iter().find(|v| v.key == "cafe").map(|v| v.value.as_str()), Some("café"));
        assert_eq!(res.console_logs.len(), 1);
        assert_eq!(res.console_logs[0].message, "Welcome to café");
        assert_eq!(res.test_results.len(), 1);
        assert_eq!(res.test_results[0].passed, true, "Test failed: {:?}", res.test_results[0].error);
    }
}
