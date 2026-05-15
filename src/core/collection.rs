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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Auth {
    None,
    Bearer {
        token: String,
    },
    Basic {
        username: String,
        password: String,
    },
    ApiKey {
        key: String,
        value: String,
        in_header: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RequestBody {
    None,
    Raw {
        content: String,
        content_type: String,
    },
    FormData {
        items: Vec<KVParam>,
    },
    XWwwFormUrlEncoded {
        items: Vec<KVParam>,
    },
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
            auth: Auth::None,
            body: RequestBody::None,
            pre_request_script: None,
            post_response_script: None,
        }
    }
}
