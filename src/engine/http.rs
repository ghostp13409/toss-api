use crate::core::collection::{Auth, AuthType, BodyType, RequestBody, KVParam};
use crate::core::scripting::{execute_pre_request_script, execute_post_response_script, ScriptExecutionResult};
use reqwest::{Client, Method, Response};
use std::collections::HashMap;

pub struct PipelineResult {
    pub status: reqwest::StatusCode,
    pub version: reqwest::Version,
    pub remote_addr: Option<std::net::SocketAddr>,
    pub headers: reqwest::header::HeaderMap,
    pub body: Vec<u8>,
    pub pre_script_result: Option<ScriptExecutionResult>,
    pub post_script_result: Option<ScriptExecutionResult>,
}

#[derive(Clone)]
pub struct RequestEngine {
    client: Client,
}

impl RequestEngine {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    pub async fn send(
        &self,
        method: Method,
        url: &str,
        headers: HashMap<String, String>,
        params: Vec<(String, String)>,
        body_type: RequestBody,
        auth: Auth,
    ) -> Result<Response, reqwest::Error> {
        let mut parsed_url = reqwest::Url::parse(url)
            .unwrap_or_else(|_| reqwest::Url::parse("http://localhost").unwrap());

        {
            let mut query = parsed_url.query_pairs_mut();
            for (k, v) in params {
                query.append_pair(&k, &v);
            }
        }

        let mut request = self.client.request(method, parsed_url);

        // Apply headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Apply Auth
        match auth.selected {
            AuthType::None => {}
            AuthType::Bearer => {
                request = request.bearer_auth(auth.bearer.token);
            }
            AuthType::Basic => {
                request = request.basic_auth(auth.basic.username, Some(auth.basic.password));
            }
            AuthType::ApiKey => {
                if auth.api_key.in_header {
                    request = request.header(auth.api_key.key, auth.api_key.value);
                } else {
                    // TODO: Handle ApiKey in query if needed
                }
            }
        }

        // Apply Body
        match body_type.selected {
            BodyType::None => {}
            BodyType::Raw => {
                if !body_type.raw.content_type.is_empty() {
                    request =
                        request.header(reqwest::header::CONTENT_TYPE, &body_type.raw.content_type);
                }
                request = request.body(body_type.raw.content);
            }
            BodyType::FormData => {
                let mut form = reqwest::multipart::Form::new();
                for item in body_type.form_data.items {
                    if item.enabled {
                        form = form.text(item.key, item.value);
                    }
                }
                request = request.multipart(form);
            }
            BodyType::XWwwFormUrlEncoded => {
                let mut params = Vec::new();
                for item in body_type.x_www_form_urlencoded.items {
                    if item.enabled {
                        params.push((item.key, item.value));
                    }
                }
                request = request.form(&params);
            }
        }

        request.send().await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn send_with_pipeline(
        &self,
        method: Method,
        url: &str,
        headers: HashMap<String, String>,
        params: Vec<(String, String)>,
        body_type: RequestBody,
        auth: Auth,
        pre_request_script: Option<&str>,
        post_response_script: Option<&str>,
        env_vars: &mut Vec<KVParam>,
    ) -> Result<PipelineResult, reqwest::Error> {
        let mut final_url = url.to_string();
        
        let mut kv_headers = headers.into_iter()
            .map(|(k, v)| KVParam { key: k, value: v, enabled: true, description: None })
            .collect::<Vec<_>>();

        let pre_res = if let Some(script) = pre_request_script {
            if !script.trim().is_empty() {
                execute_pre_request_script(script, env_vars, &mut kv_headers, &mut final_url).ok()
            } else {
                None
            }
        } else {
            None
        };

        let mut final_headers = HashMap::new();
        for h in &kv_headers {
            if h.enabled && !h.key.is_empty() {
                final_headers.insert(h.key.clone(), h.value.clone());
            }
        }

        let response = self.send(
            method,
            &final_url,
            final_headers,
            params,
            body_type,
            auth,
        ).await?;

        let status = response.status();
        let version = response.version();
        let remote_addr = response.remote_addr();
        let res_headers = response.headers().clone();
        
        let status_code = status.as_u16();
        let status_text = status.canonical_reason().unwrap_or("");
        
        let mut response_kv_headers = Vec::new();
        for (k, v) in &res_headers {
            response_kv_headers.push(KVParam {
                key: k.as_str().to_string(),
                value: v.to_str().unwrap_or("").to_string(),
                enabled: true,
                description: None,
            });
        }

        let body_bytes = response.bytes().await?;
        let body_text = String::from_utf8_lossy(&body_bytes).into_owned();

        let post_res = if let Some(script) = post_response_script {
            if !script.trim().is_empty() {
                execute_post_response_script(script, env_vars, status_code, status_text, &response_kv_headers, &body_text).ok()
            } else {
                None
            }
        } else {
            None
        };

        Ok(PipelineResult {
            status,
            version,
            remote_addr,
            headers: res_headers,
            body: body_bytes.to_vec(),
            pre_script_result: pre_res,
            post_script_result: post_res,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collection::{Auth, RequestBody, KVParam};

    #[tokio::test]
    async fn test_pipeline_execution() {
        let engine = RequestEngine::new();
        let mut env_vars = vec![
            KVParam { key: "foo".to_string(), value: "bar".to_string(), enabled: true, description: None }
        ];
        
        let pre_script = "client.environment.set('foo', 'baz'); client.request.headers.add('X-Custom', '123');";
        let post_script = "pm.test('Status is 200', function() { pm.expect(response.status).to.equal(200); });";
        
        // This method doesn't exist yet, so the test will fail to compile.
        // We'll implement it next.
        let res = engine.send_with_pipeline(
            reqwest::Method::GET,
            "https://httpbin.org/get",
            HashMap::new(),
            vec![],
            RequestBody::default(),
            Auth::default(),
            Some(pre_script),
            Some(post_script),
            &mut env_vars,
        ).await.expect("Failed to send request");

        assert_eq!(res.status, 200);
        assert_eq!(env_vars.iter().find(|v| v.key == "foo").unwrap().value, "baz");
        
        let post_res = res.post_script_result.unwrap();
        assert_eq!(post_res.test_results.len(), 1);
        assert!(post_res.test_results[0].passed);
    }
}
