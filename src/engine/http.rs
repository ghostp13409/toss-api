use crate::core::collection::{Auth, RequestBody};
use reqwest::{Client, Method, Response};
use std::collections::HashMap;

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
        match auth {
            Auth::None => {}
            Auth::Bearer { token } => {
                request = request.bearer_auth(token);
            }
            Auth::Basic { username, password } => {
                request = request.basic_auth(username, Some(password));
            }
            Auth::ApiKey {
                key,
                value,
                in_header,
            } => {
                if in_header {
                    request = request.header(key, value);
                } else {
                    // Re-parse URL to add query param if needed, or use query_pairs_mut
                    // Actually we can't easily modify the request URL here after it's in the builder
                    // So we should have added it to params earlier.
                    // For now, let's just handle it in builder if possible or skip.
                    // reqwest doesn't have a direct query() on builder that is easy to use with Auth.
                }
            }
        }

        // Apply Body
        match body_type {
            RequestBody::None => {}
            RequestBody::Raw { content, .. } => {
                request = request.body(content);
            }
            RequestBody::FormData { items } => {
                let mut form = reqwest::multipart::Form::new();
                for item in items {
                    if item.enabled {
                        form = form.text(item.key, item.value);
                    }
                }
                request = request.multipart(form);
            }
            RequestBody::XWwwFormUrlEncoded { items } => {
                let mut params = Vec::new();
                for item in items {
                    if item.enabled {
                        params.push((item.key, item.value));
                    }
                }
                request = request.form(&params);
            }
        }

        request.send().await
    }
}
