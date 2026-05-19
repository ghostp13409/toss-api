use crate::cli::args::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub items: Vec<CollectionItem>,
    #[serde(default)]
    pub expanded: bool,
    #[serde(default)]
    pub env_vars: Vec<KVParam>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum CollectionItem {
    Folder(Folder),
    Request(Request),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub items: Vec<CollectionItem>,
    #[serde(default)]
    pub expanded: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KVParam {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    None,
    Bearer,
    Basic,
    ApiKey,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BearerAuth {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ApiKeyAuth {
    pub key: String,
    pub value: String,
    pub in_header: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Auth {
    pub selected: AuthType,
    pub bearer: BearerAuth,
    pub basic: BasicAuth,
    pub api_key: ApiKeyAuth,
}

impl Default for AuthType {
    fn default() -> Self {
        Self::None
    }
}

impl Auth {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bearer(token: String) -> Self {
        Self {
            selected: AuthType::Bearer,
            bearer: BearerAuth { token },
            ..Default::default()
        }
    }

    pub fn basic(username: String, password: String) -> Self {
        Self {
            selected: AuthType::Basic,
            basic: BasicAuth { username, password },
            ..Default::default()
        }
    }

    pub fn api_key(key: String, value: String, in_header: bool) -> Self {
        Self {
            selected: AuthType::ApiKey,
            api_key: ApiKeyAuth {
                key,
                value,
                in_header,
            },
            ..Default::default()
        }
    }

    pub fn auto_select(&mut self) {
        if self.selected != AuthType::None {
            return;
        }

        if !self.bearer.token.is_empty() {
            self.selected = AuthType::Bearer;
        } else if !self.basic.username.is_empty() || !self.basic.password.is_empty() {
            self.selected = AuthType::Basic;
        } else if !self.api_key.key.is_empty() || !self.api_key.value.is_empty() {
            self.selected = AuthType::ApiKey;
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    None,
    Raw,
    FormData,
    XWwwFormUrlEncoded,
}

impl Default for BodyType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RawBody {
    pub content: String,
    pub content_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FormDataBody {
    pub items: Vec<KVParam>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RequestBody {
    pub selected: BodyType,
    pub raw: RawBody,
    pub form_data: FormDataBody,
    pub x_www_form_urlencoded: FormDataBody,
}

impl RequestBody {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn raw(content: String, content_type: String) -> Self {
        Self {
            selected: BodyType::Raw,
            raw: RawBody {
                content,
                content_type,
            },
            ..Default::default()
        }
    }

    pub fn form_data(items: Vec<KVParam>) -> Self {
        Self {
            selected: BodyType::FormData,
            form_data: FormDataBody { items },
            ..Default::default()
        }
    }

    pub fn x_www_form_urlencoded(items: Vec<KVParam>) -> Self {
        Self {
            selected: BodyType::XWwwFormUrlEncoded,
            x_www_form_urlencoded: FormDataBody { items },
            ..Default::default()
        }
    }

    pub fn auto_select(&mut self) {
        if self.selected != BodyType::None {
            return;
        }

        if !self.raw.content.is_empty() {
            self.selected = BodyType::Raw;
        } else if !self.form_data.items.is_empty() {
            self.selected = BodyType::FormData;
        } else if !self.x_www_form_urlencoded.items.is_empty() {
            self.selected = BodyType::XWwwFormUrlEncoded;
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub id: String,
    pub name: String,
    pub method: Method,
    pub url: String,
    pub params: Vec<KVParam>,
    pub headers: Vec<KVParam>,
    pub auth: Auth,
    pub body: RequestBody,
    pub pre_request_script: Option<String>,
    pub post_response_script: Option<String>,
}

impl Collection {
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            items: Vec::new(),
            expanded: false,
            env_vars: Vec::new(),
        }
    }

    pub fn find_request_mut(&mut self, id: &str) -> Option<&mut Request> {
        for item in &mut self.items {
            if let Some(req) = item.find_request_mut(id) {
                return Some(req);
            }
        }
        None
    }

    pub fn find_request(&self, id: &str) -> Option<&Request> {
        for item in &self.items {
            if let Some(req) = item.find_request(id) {
                return Some(req);
            }
        }
        None
    }

    pub fn find_request_by_name(&self, name: &str) -> Option<&Request> {
        for item in &self.items {
            if let Some(req) = item.find_request_by_name(name) {
                return Some(req);
            }
        }
        None
    }

    pub fn replace_urls_with_placeholder(
        &mut self,
        base_url: &str,
        placeholder: &str,
    ) -> Vec<(String, String)> {
        let mut changed_ids = Vec::new();
        Self::recursive_replace(&mut self.items, base_url, placeholder, &mut changed_ids);
        changed_ids
    }

    fn recursive_replace(
        items: &mut [CollectionItem],
        base_url: &str,
        placeholder: &str,
        changed: &mut Vec<(String, String)>,
    ) {
        for item in items {
            match item {
                CollectionItem::Request(r) => {
                    if r.url.starts_with(base_url) {
                        r.url = r.url.replace(base_url, placeholder);
                        changed.push((r.id.clone(), r.url.clone()));
                    }
                }
                CollectionItem::Folder(f) => {
                    Self::recursive_replace(&mut f.items, base_url, placeholder, changed)
                }
            }
        }
    }
}

impl Folder {
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            items: Vec::new(),
            expanded: false,
        }
    }
}

impl CollectionItem {
    pub fn find_request_mut(&mut self, id: &str) -> Option<&mut Request> {
        match self {
            CollectionItem::Request(req) => {
                if req.id == id {
                    Some(req)
                } else {
                    None
                }
            }
            CollectionItem::Folder(f) => {
                for item in &mut f.items {
                    if let Some(req) = item.find_request_mut(id) {
                        return Some(req);
                    }
                }
                None
            }
        }
    }

    pub fn find_request(&self, id: &str) -> Option<&Request> {
        match self {
            CollectionItem::Request(req) => {
                if req.id == id {
                    Some(req)
                } else {
                    None
                }
            }
            CollectionItem::Folder(f) => {
                for item in &f.items {
                    if let Some(req) = item.find_request(id) {
                        return Some(req);
                    }
                }
                None
            }
        }
    }

    pub fn find_request_by_name(&self, name: &str) -> Option<&Request> {
        match self {
            CollectionItem::Request(req) => {
                if req.name == name {
                    Some(req)
                } else {
                    None
                }
            }
            CollectionItem::Folder(f) => {
                for item in &f.items {
                    if let Some(req) = item.find_request_by_name(name) {
                        return Some(req);
                    }
                }
                None
            }
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            CollectionItem::Folder(f) => &f.name,
            CollectionItem::Request(r) => &r.name,
        }
    }

    #[allow(dead_code)]
    pub fn set_name(&mut self, name: String) {
        match self {
            CollectionItem::Folder(f) => f.name = name,
            CollectionItem::Request(r) => r.name = name,
        }
    }
}

impl Request {
    pub fn new(name: String, method: Method, url: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            method,
            url,
            params: Vec::new(),
            headers: Vec::new(),
            auth: Auth::default(),
            body: RequestBody::default(),
            pre_request_script: None,
            post_response_script: None,
        }
    }
}
