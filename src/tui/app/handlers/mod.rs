use super::enums::*;
use super::state::App;
use crate::cli::args::Method;
use crate::core::collection::{Collection, CollectionItem, KVParam};
use ratatui::widgets::{ListState, TableState};
use std::collections::HashMap;

pub mod collections;
pub mod navigation;
pub mod params;

impl App {
    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            focused_panel: FocusedPanel::Collections,
            left_bottom_tab: LeftBottomTab::Apis,
            active_request_part: RequestBarPart::Url,
            show_method_search: false,
            method_search_query: String::new(),
            show_search: false,
            search_query: String::new(),
            url: "https://httpbin.org/get".to_string(),
            method: Method::Get,
            command_input: String::new(),
            cursor_position: 0,
            should_quit: false,
            collections: Vec::new(),
            current_request_id: None,
            active_collection_index: 0,
            active_folder_id: None,
            selected_collection_index: 0,
            selected_api_index: 0,
            rename_input: String::new(),
            pending_item_type: None,
            error_message: None,
            notification: None,
            // should_delete_item: false,
            selected_property_tab: PropertyTab::Params,
            property_editor_row: 0,
            property_editor_field: PropertyEditorField::Key,
            response_body: String::new(),
            response_content_type: None,
            response_status: None,
            response_stats_data: None,
            selected_stats_tab: StatsTab::Overview,            pending_actions: Vec::new(),            response_scroll: 0,
            response_horizontal_scroll: 0,
            stats_scroll: 0,
            details_scroll: 0,            collections_state: ListState::default(),
            apis_state: ListState::default(),
            details_table_state: TableState::default(),
            environments_table_state: TableState::default(),
            g_pressed: false,
            mask_env_values: true,
            selected_env_index: 0,
            show_autocomplete: false,
            autocomplete_query: String::new(),
            autocomplete_index: 0,
            last_cursor_pos: (0, 0),
        }
    }

    pub fn get_active_collection_env_vars(&self) -> Vec<KVParam> {
        self.collections
            .get(self.active_collection_index)
            .map(|c| c.env_vars.clone())
            .unwrap_or_default()
    }

    pub fn get_active_env(&self) -> crate::core::env::Environment {
        let mut variables = HashMap::new();
        if let Some(col) = self.collections.get(self.active_collection_index) {
            for v in &col.env_vars {
                if v.enabled {
                    variables.insert(v.key.clone(), v.value.clone());
                }
            }
        }
        crate::core::env::Environment {
            name: "Active".to_string(),
            variables,
        }
    }

    pub fn toggle_env_mask(&mut self) {
        self.mask_env_values = !self.mask_env_values;
    }

    pub fn notify(&mut self, message: impl Into<String>) {
        self.notification = Some((message.into(), std::time::Instant::now()));
    }

    pub fn create_smart_env(&mut self) {
        let (base_url, _urls) = {
            let col = match self.collections.get(self.active_collection_index) {
                Some(c) => c,
                None => return,
            };

            let mut urls = Vec::new();
            self.collect_urls(&col.items, &mut urls);

            if urls.is_empty() {
                return;
            }

            // Find common prefix
            let mut common_prefix = urls[0].clone();
            for url in &urls[1..] {
                let mut new_prefix = String::new();
                for (c1, c2) in common_prefix.chars().zip(url.chars()) {
                    if c1 == c2 {
                        new_prefix.push(c1);
                    } else {
                        break;
                    }
                }
                common_prefix = new_prefix;
            }

            // Trim to last slash to avoid cutting in middle of hostname/path
            if let Some(last_slash) = common_prefix.rfind('/') {
                if last_slash > 7 {
                    // keep at least http://
                    common_prefix.truncate(last_slash);
                }
            }

            if common_prefix.len() < 8 {
                // Too short to be a useful baseURL
                return;
            }

            (common_prefix, urls)
        };

        // Add to env vars
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            if !col.env_vars.iter().any(|v| v.key == "baseUrl") {
                col.env_vars.push(KVParam {
                    key: "baseUrl".to_string(),
                    value: base_url.clone(),
                    enabled: true,
                    description: Some("Auto-generated smart variable".to_string()),
                });
            }

            // Replace in requests
            let placeholder = "{{baseUrl}}";
            let changed = col.replace_urls_with_placeholder(&base_url, placeholder);

            // Update current app.url if it was one of the changed ones
            if let Some(curr_id) = &self.current_request_id {
                if let Some((_, new_url)) = changed.iter().find(|(id, _)| id == curr_id) {
                    self.url = new_url.clone();
                }
            }
        }
    }

    fn collect_urls(&self, items: &[CollectionItem], urls: &mut Vec<String>) {
        for item in items {
            match item {
                CollectionItem::Request(r) => urls.push(r.url.clone()),
                CollectionItem::Folder(f) => self.collect_urls(&f.items, urls),
            }
        }
    }

    pub fn save_current_request(&mut self) {
        if let Some(req_id) = &self.current_request_id {
            let id = req_id.clone();
            let url = self.url.clone();
            let method = self.method;
            if let Some(col) = self.collections.get_mut(self.active_collection_index) {
                if let Some(req) = col.find_request_mut(&id) {
                    req.url = url;
                    req.method = method;
                }
            }
        }
    }

    pub fn load_sample_data(&mut self) {
        let mut col = Collection::new("Sample Collection".to_string());
        let mut req = crate::core::collection::Request::new(
            "Get Sample".to_string(),
            Method::Get,
            "https://httpbin.org/get".to_string(),
        );
        req.params.push(KVParam {
            key: "foo".to_string(),
            value: "bar".to_string(),
            enabled: true,
            description: None,
        });
        col.items.push(CollectionItem::Request(req));
        self.collections.push(col);
    }

    pub fn import_collection(&mut self, path: &str) {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(col) = crate::core::import::postman::import_postman(&content) {
                self.collections.push(col);
            }
        }
    }

    pub fn parse_project_tui(&mut self, path_str: &str) {
        let path = if path_str.is_empty() {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        } else {
            std::path::PathBuf::from(path_str)
        };

        match crate::core::parser::parse_project(&path) {
            Ok(new_col) => {
                // Check if collection with same name already exists
                if let Some(existing_idx) =
                    self.collections.iter().position(|c| c.name == new_col.name)
                {
                    // Update existing collection's items
                    self.collections[existing_idx].items = new_col.items;
                    // Keep env_vars as they might have been manually added/edited
                } else {
                    self.collections.push(new_col);
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to parse project: {}", e));
            }
        }
    }

    pub fn get_autocomplete_options(&self) -> Vec<String> {
        let env_vars = self.get_active_collection_env_vars();
        let query = self.autocomplete_query.to_lowercase();
        env_vars
            .iter()
            .map(|v| v.key.clone())
            .filter(|k| k.to_lowercase().contains(&query))
            .collect()
    }

    pub fn insert_autocomplete_selection(&mut self) {
        let options = self.get_autocomplete_options();
        if let Some(selection) = options.get(self.autocomplete_index) {
            if self.focused_panel == FocusedPanel::RequestBar
                && self.active_request_part == RequestBarPart::Url
            {
                if let Some(start_pos) = self.url[..self.cursor_position].rfind("{{") {
                    let end_text = self.url[self.cursor_position..].to_string();
                    self.url.truncate(start_pos);
                    self.url.push_str("{{");
                    self.url.push_str(&selection);
                    self.url.push_str("}}");
                    let new_cursor = self.url.len();
                    self.url.push_str(&end_text);
                    self.cursor_position = new_cursor;
                }
            } else if self.focused_panel == FocusedPanel::Details
                || self.focused_panel == FocusedPanel::Environments
            {
                let mut current_val = self.get_kv_editor_value_internal();
                if let Some(start_pos) = current_val[..self.cursor_position].rfind("{{") {
                    let end_text = current_val[self.cursor_position..].to_string();
                    current_val.truncate(start_pos);
                    current_val.push_str("{{");
                    current_val.push_str(&selection);
                    current_val.push_str("}}");
                    let new_cursor = current_val.len();
                    current_val.push_str(&end_text);
                    self.cursor_position = new_cursor;
                    self.update_kv_param_internal(current_val);
                }
            }
        }
        self.show_autocomplete = false;
        self.autocomplete_query.clear();
        self.autocomplete_index = 0;
    }

    fn get_kv_editor_value_internal(&self) -> String {
        if self.focused_panel == FocusedPanel::Environments {
            self.get_env_editor_value()
        } else {
            self.get_kv_editor_value()
        }
    }

    fn update_kv_param_internal(&mut self, val: String) {
        if self.focused_panel == FocusedPanel::Environments {
            self.update_env_editor_value(val);
        } else {
            self.update_kv_param(val);
        }
    }
}
