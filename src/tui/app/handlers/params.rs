use crate::core::collection::{Auth, KVParam, Request, RequestBody};
use reqwest::Url;
use super::super::enums::*;
use super::super::state::App;

impl App {
    pub fn add_env_var(&mut self) {
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            col.env_vars.push(KVParam {
                key: String::new(),
                value: String::new(),
                enabled: true,
                description: None,
            });
            self.selected_env_index = col.env_vars.len().saturating_sub(1);
            self.property_editor_field = PropertyEditorField::Key;
        }
    }

    pub fn delete_env_var(&mut self) {
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            if !col.env_vars.is_empty() && self.selected_env_index < col.env_vars.len() {
                col.env_vars.remove(self.selected_env_index);
                self.selected_env_index = self.selected_env_index.saturating_sub(1);
            }
        }
    }

    pub fn update_env_var(&mut self, key: String, value: String) {
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            if let Some(var) = col.env_vars.get_mut(self.selected_env_index) {
                var.key = key;
                var.value = value;
            }
        }
    }

    pub fn sync_url_from_params(&mut self) {
        let (params, base_url) = {
            if let Some(req) = self.get_current_request() {
                (req.params.clone(), req.url.clone())
            } else {
                return;
            }
        };

        if let Ok(mut parsed_url) = Url::parse(&base_url) {
            parsed_url.query_pairs_mut().clear();
            for param in &params {
                if param.enabled && (!param.key.is_empty() || !param.value.is_empty()) {
                    parsed_url
                        .query_pairs_mut()
                        .append_pair(&param.key, &param.value);
                }
            }
            let new_url = parsed_url.to_string();
            self.url = new_url.clone();
            if let Some(mut_req) = self.get_current_request_mut() {
                mut_req.url = new_url;
            }
        }
    }

    pub fn sync_params_from_url(&mut self) {
        if let Ok(parsed_url) = Url::parse(&self.url) {
            let new_params: Vec<KVParam> = parsed_url
                .query_pairs()
                .map(|(k, v)| KVParam {
                    key: k.into_owned(),
                    value: v.into_owned(),
                    enabled: true,
                    description: None,
                })
                .collect();

            let url_str = parsed_url.to_string();
            if let Some(req) = self.get_current_request_mut() {
                req.params = new_params;
                req.url = url_str;
            }
        }
    }

    pub fn get_current_request(&self) -> Option<&Request> {
        let req_id = self.current_request_id.as_ref()?;
        let col = self.collections.get(self.active_collection_index)?;
        col.find_request(req_id)
    }

    pub fn get_current_request_mut(&mut self) -> Option<&mut Request> {
        let req_id = self.current_request_id.clone()?;
        let col = self.collections.get_mut(self.active_collection_index)?;
        col.find_request_mut(&req_id)
    }

    pub fn add_kv_param(&mut self) {
        let tab = self.selected_property_tab;
        if let Some(req) = self.get_current_request_mut() {
            let target = match tab {
                PropertyTab::Params => &mut req.params,
                PropertyTab::Headers => &mut req.headers,
                PropertyTab::Body => match &mut req.body {
                    RequestBody::FormData { items } => items,
                    RequestBody::XWwwFormUrlEncoded { items } => items,
                    _ => return,
                },
                _ => return,
            };
            target.push(KVParam {
                key: String::new(),
                value: String::new(),
                enabled: true,
                description: None,
            });
            self.property_editor_row = target.len() - 1;
            self.property_editor_field = PropertyEditorField::Key;
        }
        if self.selected_property_tab == PropertyTab::Params {
            self.sync_url_from_params();
        }
    }

    pub fn delete_kv_param(&mut self) {
        let tab = self.selected_property_tab;
        let row = self.property_editor_row;
        if let Some(req) = self.get_current_request_mut() {
            let target = match tab {
                PropertyTab::Params => &mut req.params,
                PropertyTab::Headers => &mut req.headers,
                PropertyTab::Body => match &mut req.body {
                    RequestBody::FormData { items } => items,
                    RequestBody::XWwwFormUrlEncoded { items } => items,
                    _ => return,
                },
                _ => return,
            };
            if !target.is_empty() && row < target.len() {
                target.remove(row);
                self.property_editor_row = row.saturating_sub(1);
            }
        }
        if self.selected_property_tab == PropertyTab::Params {
            self.sync_url_from_params();
        }
    }

    pub fn toggle_auth_bool(&mut self) {
        let row = self.property_editor_row;
        if let Some(req) = self.get_current_request_mut() {
            if let Auth::ApiKey { in_header, .. } = &mut req.auth {
                if row == 2 {
                    *in_header = !*in_header;
                }
            }
        }
    }

    pub fn update_kv_param(&mut self, value: String) {
        let tab = self.selected_property_tab;
        let row = self.property_editor_row;
        let field = self.property_editor_field;
        if let Some(req) = self.get_current_request_mut() {
            match tab {
                PropertyTab::Params => {
                    if let Some(p) = req.params.get_mut(row) {
                        match field {
                            PropertyEditorField::Key => p.key = value,
                            PropertyEditorField::Value => p.value = value,
                            PropertyEditorField::Description => p.description = Some(value),
                        }
                    }
                }
                PropertyTab::Headers => {
                    if let Some(p) = req.headers.get_mut(row) {
                        match field {
                            PropertyEditorField::Key => p.key = value,
                            PropertyEditorField::Value => p.value = value,
                            PropertyEditorField::Description => p.description = Some(value),
                        }
                    }
                }
                PropertyTab::Auth => match &mut req.auth {
                    Auth::Bearer { token } => *token = value,
                    Auth::Basic { username, password } => {
                        if row == 0 {
                            *username = value;
                        } else {
                            *password = value;
                        }
                    }
                    Auth::ApiKey {
                        key,
                        value: v,
                        in_header,
                    } => {
                        if row == 0 {
                            *key = value;
                        } else if row == 1 {
                            *v = value;
                        } else {
                            *in_header = value.to_lowercase() == "true";
                        }
                    }
                    _ => {}
                },
                PropertyTab::Body => match &mut req.body {
                    RequestBody::FormData { items } => {
                        if let Some(p) = items.get_mut(row) {
                            match field {
                                PropertyEditorField::Key => p.key = value,
                                PropertyEditorField::Value => p.value = value,
                                PropertyEditorField::Description => p.description = Some(value),
                            }
                        }
                    }
                    RequestBody::XWwwFormUrlEncoded { items } => {
                        if let Some(p) = items.get_mut(row) {
                            match field {
                                PropertyEditorField::Key => p.key = value,
                                PropertyEditorField::Value => p.value = value,
                                PropertyEditorField::Description => p.description = Some(value),
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        if self.selected_property_tab == PropertyTab::Params {
            self.sync_url_from_params();
        }
    }

    pub fn toggle_kv_param(&mut self) {
        let tab = self.selected_property_tab;
        let row = self.property_editor_row;
        if let Some(req) = self.get_current_request_mut() {
            let target = match tab {
                PropertyTab::Params => &mut req.params,
                PropertyTab::Headers => &mut req.headers,
                PropertyTab::Body => match &mut req.body {
                    RequestBody::FormData { items } => items,
                    RequestBody::XWwwFormUrlEncoded { items } => items,
                    _ => return,
                },
                _ => return,
            };
            if let Some(param) = target.get_mut(row) {
                param.enabled = !param.enabled;
            }
        }
        if self.selected_property_tab == PropertyTab::Params {
            self.sync_url_from_params();
        }
    }

    pub fn cycle_body_type(&mut self) {
        if let Some(req) = self.get_current_request_mut() {
            req.body = match req.body {
                RequestBody::None => RequestBody::Raw {
                    content: String::new(),
                    content_type: "application/json".to_string(),
                },
                RequestBody::Raw { .. } => RequestBody::FormData { items: Vec::new() },
                RequestBody::FormData { .. } => {
                    RequestBody::XWwwFormUrlEncoded { items: Vec::new() }
                }
                RequestBody::XWwwFormUrlEncoded { .. } => RequestBody::None,
            };
        }
    }

    pub fn cycle_auth_type(&mut self) {
        if let Some(req) = self.get_current_request_mut() {
            req.auth = match req.auth {
                Auth::None => Auth::Bearer {
                    token: String::new(),
                },
                Auth::Bearer { .. } => Auth::Basic {
                    username: String::new(),
                    password: String::new(),
                },
                Auth::Basic { .. } => Auth::ApiKey {
                    key: String::new(),
                    value: String::new(),
                    in_header: true,
                },
                Auth::ApiKey { .. } => Auth::None,
            };
        }
    }

    pub fn get_kv_editor_value(&self) -> String {
        if let Some(req) = self.get_current_request() {
            match self.selected_property_tab {
                PropertyTab::Params => {
                    if let Some(p) = req.params.get(self.property_editor_row) {
                        return match self.property_editor_field {
                            PropertyEditorField::Key => p.key.clone(),
                            PropertyEditorField::Value => p.value.clone(),
                            PropertyEditorField::Description => {
                                p.description.clone().unwrap_or_default()
                            }
                        };
                    }
                }
                PropertyTab::Headers => {
                    if let Some(p) = req.headers.get(self.property_editor_row) {
                        return match self.property_editor_field {
                            PropertyEditorField::Key => p.key.clone(),
                            PropertyEditorField::Value => p.value.clone(),
                            PropertyEditorField::Description => {
                                p.description.clone().unwrap_or_default()
                            }
                        };
                    }
                }
                PropertyTab::Auth => match &req.auth {
                    Auth::Bearer { token } => return token.clone(),
                    Auth::Basic { username, password } => {
                        if self.property_editor_row == 0 {
                            return username.clone();
                        } else {
                            return password.clone();
                        }
                    }
                    Auth::ApiKey { key, value, .. } => {
                        if self.property_editor_row == 0 {
                            return key.clone();
                        } else if self.property_editor_row == 1 {
                            return value.clone();
                        } else {
                            // "In Header" is bool, returning as string
                            match &req.auth {
                                Auth::ApiKey { in_header, .. } => return in_header.to_string(),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                PropertyTab::Body => match &req.body {
                    RequestBody::FormData { items } | RequestBody::XWwwFormUrlEncoded { items } => {
                        if let Some(p) = items.get(self.property_editor_row) {
                            return match self.property_editor_field {
                                PropertyEditorField::Key => p.key.clone(),
                                PropertyEditorField::Value => p.value.clone(),
                                PropertyEditorField::Description => {
                                    p.description.clone().unwrap_or_default()
                                }
                            };
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        String::new()
    }

    pub fn get_env_editor_value(&self) -> String {
        if let Some(col) = self.collections.get(self.active_collection_index) {
            if let Some(var) = col.env_vars.get(self.selected_env_index) {
                return match self.property_editor_field {
                    PropertyEditorField::Key => var.key.clone(),
                    PropertyEditorField::Value => var.value.clone(),
                    PropertyEditorField::Description => {
                        var.description.clone().unwrap_or_default()
                    }
                };
            }
        }
        String::new()
    }

    pub fn update_env_editor_value(&mut self, new_val: String) {
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            if let Some(var) = col.env_vars.get_mut(self.selected_env_index) {
                match self.property_editor_field {
                    PropertyEditorField::Key => var.key = new_val,
                    PropertyEditorField::Value => var.value = new_val,
                    PropertyEditorField::Description => var.description = Some(new_val),
                }
            }
        }
    }
}
