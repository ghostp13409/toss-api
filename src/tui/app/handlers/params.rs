use super::super::enums::*;
use super::super::state::App;
use crate::core::collection::{KVParam, Request};

impl App {
    pub fn add_env_var(&mut self, key: String) {
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            col.env_vars.push(KVParam {
                key,
                value: String::new(),
                enabled: true,
                description: None,
            });
            self.selected_env_index = col.env_vars.len().saturating_sub(1);
            self.property_editor_field = PropertyEditorField::Value;
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

    // pub fn update_env_var(&mut self, key: String, value: String) {
    //     if let Some(col) = self.collections.get_mut(self.active_collection_index) {
    //         if let Some(var) = col.env_vars.get_mut(self.selected_env_index) {
    //             var.key = key;
    //             var.value = value;
    //         }
    //     }
    // }

    pub fn sync_url_from_params(&mut self) {
        let (params, current_url) = {
            if let Some(req) = self.get_current_request() {
                (req.params.clone(), req.url.clone())
            } else {
                return;
            }
        };

        if current_url.is_empty() {
            return;
        }

        // Split base URL and existing query
        let base_url = if let Some(q_pos) = current_url.find('?') {
            &current_url[..q_pos]
        } else {
            &current_url
        };

        let mut query_parts = Vec::new();
        for param in &params {
            if param.enabled && (!param.key.is_empty() || !param.value.is_empty()) {
                query_parts.push(format!("{}={}", param.key, param.value));
            }
        }

        let new_url = if query_parts.is_empty() {
            base_url.to_string()
        } else {
            format!("{}?{}", base_url, query_parts.join("&"))
        };

        self.url = new_url.clone();
        if let Some(mut_req) = self.get_current_request_mut() {
            mut_req.url = new_url;
        }
    }

    pub fn sync_params_from_url(&mut self) {
        let url_clone = self.url.clone();

        // Always sync the URL string to the request model
        if let Some(req) = self.get_current_request_mut() {
            req.url = url_clone.clone();
        }

        // Extract query string manually to be robust against template variables in host/scheme
        if let Some(q_pos) = url_clone.find('?') {
            let query_str = &url_clone[q_pos + 1..];
            let new_params: Vec<KVParam> = if query_str.is_empty() {
                Vec::new()
            } else {
                query_str
                    .split('&')
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        let (k, v) = s.split_once('=').unwrap_or((s, ""));
                        KVParam {
                            key: k.to_string(),
                            value: v.to_string(),
                            enabled: true,
                            description: None,
                        }
                    })
                    .collect()
            };

            if let Some(req) = self.get_current_request_mut() {
                // We want to preserve descriptions and enabled status if possible
                let mut merged_params = Vec::new();
                for mut new_p in new_params {
                    if let Some(existing) = req.params.iter().find(|p| p.key == new_p.key) {
                        new_p.description = existing.description.clone();
                        new_p.enabled = existing.enabled;
                    }
                    merged_params.push(new_p);
                }
                req.params = merged_params;
            }
        } else {
            // No query string, clear params if they came from the URL
            if let Some(req) = self.get_current_request_mut() {
                req.params.clear();
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

    pub fn add_kv_param(&mut self, key: String) {
        let tab = self.selected_property_tab;
        if let Some(req) = self.get_current_request_mut() {
            let target = match tab {
                PropertyTab::Params => &mut req.params,
                PropertyTab::Headers => &mut req.headers,
                PropertyTab::Body => match req.body.selected {
                    crate::core::collection::BodyType::FormData => &mut req.body.form_data.items,
                    crate::core::collection::BodyType::XWwwFormUrlEncoded => {
                        &mut req.body.x_www_form_urlencoded.items
                    }
                    _ => return,
                },
                _ => return,
            };
            target.push(KVParam {
                key,
                value: String::new(),
                enabled: true,
                description: None,
            });
            self.property_editor_row = target.len() - 1;
            self.property_editor_field = PropertyEditorField::Value;
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
                PropertyTab::Body => match req.body.selected {
                    crate::core::collection::BodyType::FormData => &mut req.body.form_data.items,
                    crate::core::collection::BodyType::XWwwFormUrlEncoded => {
                        &mut req.body.x_www_form_urlencoded.items
                    }
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
            if req.auth.selected == crate::core::collection::AuthType::ApiKey {
                if row == 2 {
                    req.auth.api_key.in_header = !req.auth.api_key.in_header;
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
                PropertyTab::Auth => match req.auth.selected {
                    crate::core::collection::AuthType::Bearer => req.auth.bearer.token = value,
                    crate::core::collection::AuthType::Basic => {
                        if row == 0 {
                            req.auth.basic.username = value;
                        } else {
                            req.auth.basic.password = value;
                        }
                    }
                    crate::core::collection::AuthType::ApiKey => {
                        if row == 0 {
                            req.auth.api_key.key = value;
                        } else if row == 1 {
                            req.auth.api_key.value = value;
                        } else {
                            req.auth.api_key.in_header = value.to_lowercase() == "true";
                        }
                    }
                    _ => {}
                },
                PropertyTab::Body => match req.body.selected {
                    crate::core::collection::BodyType::FormData => {
                        if let Some(p) = req.body.form_data.items.get_mut(row) {
                            match field {
                                PropertyEditorField::Key => p.key = value,
                                PropertyEditorField::Value => p.value = value,
                                PropertyEditorField::Description => p.description = Some(value),
                            }
                        }
                    }
                    crate::core::collection::BodyType::XWwwFormUrlEncoded => {
                        if let Some(p) = req.body.x_www_form_urlencoded.items.get_mut(row) {
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
                PropertyTab::Body => match req.body.selected {
                    crate::core::collection::BodyType::FormData => &mut req.body.form_data.items,
                    crate::core::collection::BodyType::XWwwFormUrlEncoded => {
                        &mut req.body.x_www_form_urlencoded.items
                    }
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
            req.body.selected = match req.body.selected {
                crate::core::collection::BodyType::None => crate::core::collection::BodyType::Raw,
                crate::core::collection::BodyType::Raw => crate::core::collection::BodyType::FormData,
                crate::core::collection::BodyType::FormData => {
                    crate::core::collection::BodyType::XWwwFormUrlEncoded
                }
                crate::core::collection::BodyType::XWwwFormUrlEncoded => {
                    crate::core::collection::BodyType::None
                }
            };
            if req.body.selected == crate::core::collection::BodyType::Raw
                && req.body.raw.content_type.is_empty()
            {
                req.body.raw.content_type = "application/json".to_string();
            }
        }
    }

    pub fn cycle_raw_content_type(&mut self) {
        if let Some(req) = self.get_current_request_mut() {
            if req.body.selected == crate::core::collection::BodyType::Raw {
                req.body.raw.content_type = match req.body.raw.content_type.as_str() {
                    "application/json" => "application/xml".to_string(),
                    "application/xml" => "text/html".to_string(),
                    "text/html" => "text/plain".to_string(),
                    _ => "application/json".to_string(),
                };
            }
        }
    }

    pub fn cycle_auth_type(&mut self) {
        if let Some(req) = self.get_current_request_mut() {
            req.auth.selected = match req.auth.selected {
                crate::core::collection::AuthType::None => crate::core::collection::AuthType::Bearer,
                crate::core::collection::AuthType::Bearer => {
                    crate::core::collection::AuthType::Basic
                }
                crate::core::collection::AuthType::Basic => {
                    crate::core::collection::AuthType::ApiKey
                }
                crate::core::collection::AuthType::ApiKey => {
                    crate::core::collection::AuthType::None
                }
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
                PropertyTab::Auth => match req.auth.selected {
                    crate::core::collection::AuthType::Bearer => return req.auth.bearer.token.clone(),
                    crate::core::collection::AuthType::Basic => {
                        if self.property_editor_row == 0 {
                            return req.auth.basic.username.clone();
                        } else {
                            return req.auth.basic.password.clone();
                        }
                    }
                    crate::core::collection::AuthType::ApiKey => {
                        if self.property_editor_row == 0 {
                            return req.auth.api_key.key.clone();
                        } else if self.property_editor_row == 1 {
                            return req.auth.api_key.value.clone();
                        } else {
                            return req.auth.api_key.in_header.to_string();
                        }
                    }
                    _ => {}
                },
                PropertyTab::Body => match req.body.selected {
                    crate::core::collection::BodyType::FormData => {
                        if let Some(p) = req.body.form_data.items.get(self.property_editor_row) {
                            return match self.property_editor_field {
                                PropertyEditorField::Key => p.key.clone(),
                                PropertyEditorField::Value => p.value.clone(),
                                PropertyEditorField::Description => {
                                    p.description.clone().unwrap_or_default()
                                }
                            };
                        }
                    }
                    crate::core::collection::BodyType::XWwwFormUrlEncoded => {
                        if let Some(p) = req
                            .body
                            .x_www_form_urlencoded
                            .items
                            .get(self.property_editor_row)
                        {
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
                    PropertyEditorField::Description => var.description.clone().unwrap_or_default(),
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
