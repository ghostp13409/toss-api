use crate::core::collection::{Auth, AuthType, BodyType, RequestBody};
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
                    request = request.header(reqwest::header::CONTENT_TYPE, &body_type.raw.content_type);
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
}
