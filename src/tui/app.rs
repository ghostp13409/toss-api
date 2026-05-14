use crate::cli::args::Method;
use crate::core::collection::{Auth, Collection, CollectionItem, KVParam, Request, RequestBody};
use ratatui::widgets::{ListState, TableState};
use reqwest::Url;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Editing,
    Command,
    Rename,
    Search,
    ConfirmQuit,
    ConfirmDelete,
    CreateItem,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FocusedPanel {
    Collections,
    Apis,
    Environments,
    Properties,
    Details,
    Response,
    Stats,
    RequestBar,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LeftBottomTab {
    Apis,
    Environments,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RequestBarPart {
    Method,
    Url,
    Send,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PropertyTab {
    Params,
    Headers,
    Auth,
    Body,
    Scripts,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PropertyEditorField {
    Key,
    Value,
    Description,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TuiAction {
    SendRequest,
    EditBody,
    CopyBody,
    PasteBody,
    CopyResponseBody,
    CopyResponseAll,
}

pub struct App {
    pub input_mode: InputMode,
    pub focused_panel: FocusedPanel,
    pub left_bottom_tab: LeftBottomTab,
    pub active_request_part: RequestBarPart,
    pub show_method_search: bool,
    pub method_search_query: String,
    pub show_search: bool,
    pub search_query: String,
    pub url: String,
    pub method: Method,
    pub command_input: String,
    pub cursor_position: usize,
    pub should_quit: bool,
    pub collections: Vec<Collection>,
    pub current_request_id: Option<String>,
    pub active_collection_index: usize,
    pub active_folder_id: Option<String>,
    pub selected_collection_index: usize,
    pub selected_api_index: usize,
    pub rename_input: String,
    pub pending_item_type: Option<PendingItemType>,
    pub error_message: Option<String>,
    pub should_delete_item: bool,
    pub selected_property_tab: PropertyTab,
    pub property_editor_row: usize,
    pub property_editor_field: PropertyEditorField,
    pub response_body: String,
    pub response_status: Option<String>,
    pub response_stats: String,
    pub pending_actions: Vec<TuiAction>,
    pub response_scroll: u16,
    pub response_horizontal_scroll: u16,
    pub details_scroll: usize,
    pub collections_state: ListState,
    pub apis_state: ListState,
    pub details_table_state: TableState,
    pub environments_table_state: TableState,
    pub g_pressed: bool,
    pub mask_env_values: bool,
    pub selected_env_index: usize,
    pub show_autocomplete: bool,
    pub autocomplete_query: String,
    pub autocomplete_index: usize,
    pub last_cursor_pos: (u16, u16),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PendingItemType {
    Collection,
    Folder,
    Request,
}

pub struct VisibleItem {
    pub name: String,
    pub depth: usize,
    pub item_type: VisibleItemType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VisibleItemType {
    Collection { expanded: bool },
    Folder { expanded: bool },
    Request { method: Method, id: String },
}

impl VisibleItem {
    pub fn item_type_depth(&self) -> usize {
        match self.item_type {
            VisibleItemType::Collection { .. } => self.depth,
            _ => self.depth,
        }
    }
}

impl App {
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
}

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
            should_delete_item: false,
            selected_property_tab: PropertyTab::Params,
            property_editor_row: 0,
            property_editor_field: PropertyEditorField::Key,
            response_body: String::new(),
            response_status: None,
            response_stats: String::new(),
            pending_actions: Vec::new(),
            response_scroll: 0,
            response_horizontal_scroll: 0,
            details_scroll: 0,
            collections_state: ListState::default(),
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

    pub fn get_selected_item_name(&self) -> String {
        if self.focused_panel == FocusedPanel::Collections {
            let visible = self.get_visible_collections();
            visible
                .get(self.selected_collection_index)
                .map(|i| i.name.clone())
                .unwrap_or_default()
        } else {
            let visible = self.get_visible_items();
            visible
                .get(self.selected_api_index)
                .map(|i| i.name.clone())
                .unwrap_or_default()
        }
    }

    pub fn focus_request_bar(&mut self) {
        self.focused_panel = FocusedPanel::RequestBar;
        self.active_request_part = RequestBarPart::Url;
        self.input_mode = InputMode::Normal;
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self, max: usize) {
        if self.cursor_position < max {
            self.cursor_position += 1;
        }
    }

    pub fn insert_char(&mut self, target: &mut String, c: char) {
        let pos = self.cursor_position.min(target.len());
        target.insert(pos, c);
        self.cursor_position = pos + 1;
    }

    pub fn delete_char(&mut self, target: &mut String) {
        if self.cursor_position > 0 && self.cursor_position <= target.len() {
            target.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn delete_char_forward(&mut self, target: &mut String) {
        if self.cursor_position < target.len() {
            target.remove(self.cursor_position);
        }
    }

    pub fn insert_char_rename(&mut self, c: char) {
        self.rename_input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char_rename(&mut self) {
        if self.cursor_position > 0 {
            self.rename_input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn delete_char_forward_rename(&mut self) {
        if self.cursor_position < self.rename_input.len() {
            self.rename_input.remove(self.cursor_position);
        }
    }

    pub fn insert_char_url(&mut self, c: char) {
        self.url.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char_url(&mut self) {
        if self.cursor_position > 0 {
            self.url.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn delete_char_forward_url(&mut self) {
        if self.cursor_position < self.url.len() {
            self.url.remove(self.cursor_position);
        }
    }

    pub fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Collections => {
                if self.left_bottom_tab == LeftBottomTab::Apis {
                    FocusedPanel::Apis
                } else {
                    FocusedPanel::Environments
                }
            }
            FocusedPanel::Apis | FocusedPanel::Environments => FocusedPanel::RequestBar,
            FocusedPanel::RequestBar => FocusedPanel::Details,
            FocusedPanel::Properties => FocusedPanel::Details,
            FocusedPanel::Details => FocusedPanel::Response,
            FocusedPanel::Response => FocusedPanel::Stats,
            FocusedPanel::Stats => FocusedPanel::Collections,
        };
        self.input_mode = InputMode::Normal;
    }

    pub fn prev_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Collections => FocusedPanel::Stats,
            FocusedPanel::Apis | FocusedPanel::Environments => FocusedPanel::Collections,
            FocusedPanel::RequestBar => {
                if self.left_bottom_tab == LeftBottomTab::Apis {
                    FocusedPanel::Apis
                } else {
                    FocusedPanel::Environments
                }
            }
            FocusedPanel::Properties => FocusedPanel::RequestBar,
            FocusedPanel::Details => FocusedPanel::RequestBar,
            FocusedPanel::Response => FocusedPanel::Details,
            FocusedPanel::Stats => FocusedPanel::Response,
        };
        self.input_mode = InputMode::Normal;
    }

    pub fn next_property_tab(&mut self) {
        self.selected_property_tab = match self.selected_property_tab {
            PropertyTab::Params => PropertyTab::Headers,
            PropertyTab::Headers => PropertyTab::Auth,
            PropertyTab::Auth => PropertyTab::Body,
            PropertyTab::Body => PropertyTab::Scripts,
            PropertyTab::Scripts => PropertyTab::Params,
        };
        self.property_editor_row = 0;
        self.details_scroll = 0;
    }

    pub fn prev_property_tab(&mut self) {
        self.selected_property_tab = match self.selected_property_tab {
            PropertyTab::Params => PropertyTab::Scripts,
            PropertyTab::Headers => PropertyTab::Params,
            PropertyTab::Auth => PropertyTab::Headers,
            PropertyTab::Body => PropertyTab::Auth,
            PropertyTab::Scripts => PropertyTab::Body,
        };
        self.property_editor_row = 0;
        self.details_scroll = 0;
    }

    pub fn update_active_scope_from_tree(&mut self) {
        let visible = self.get_visible_collections();
        if let Some(_item) = visible.get(self.selected_collection_index) {
            let mut current_idx = 0;
            for (i, col) in self.collections.iter().enumerate() {
                if current_idx == self.selected_collection_index {
                    self.active_collection_index = i;
                    self.active_folder_id = None;
                    return;
                }
                current_idx += 1;
                if col.expanded {
                    let mut found_id = None;
                    if self.find_container_id_at_index(
                        &col.items,
                        &mut current_idx,
                        self.selected_collection_index,
                        &mut found_id,
                    ) {
                        self.active_collection_index = i;
                        self.active_folder_id = found_id;
                        return;
                    }
                }
            }
        }
    }

    fn find_container_id_at_index(
        &self,
        items: &[CollectionItem],
        current_idx: &mut usize,
        target_idx: usize,
        found_id: &mut Option<String>,
    ) -> bool {
        for item in items {
            if *current_idx == target_idx {
                match item {
                    CollectionItem::Folder(f) => *found_id = Some(f.id.clone()),
                    _ => {}
                }
                return true;
            }
            *current_idx += 1;
            if let CollectionItem::Folder(f) = item {
                if f.expanded {
                    let prev_found = found_id.clone();
                    *found_id = Some(f.id.clone());
                    if self.find_container_id_at_index(&f.items, current_idx, target_idx, found_id)
                    {
                        return true;
                    }
                    *found_id = prev_found;
                }
            }
        }
        false
    }

    pub fn clamp_selections(&mut self) {
        let collections = self.get_visible_collections();
        if !collections.is_empty() {
            self.selected_collection_index = self
                .selected_collection_index
                .min(collections.len().saturating_sub(1));
        } else {
            self.selected_collection_index = 0;
        }

        let items = self.get_visible_items();
        if !items.is_empty() {
            self.selected_api_index = self.selected_api_index.min(items.len().saturating_sub(1));
        } else {
            self.selected_api_index = 0;
        }
    }

    pub fn pop_up(&mut self) {
        if self.show_method_search {
            self.show_method_search = false;
            self.method_search_query.clear();
            return;
        }
        if self.error_message.is_some() {
            self.error_message = None;
            return;
        }
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Details => FocusedPanel::RequestBar,
            FocusedPanel::Properties => FocusedPanel::RequestBar,
            FocusedPanel::Response => FocusedPanel::Details,
            FocusedPanel::Stats => FocusedPanel::Response,
            _ => self.focused_panel,
        };
        self.input_mode = InputMode::Normal;
    }

    pub fn get_visible_collections(&self) -> Vec<VisibleItem> {
        let mut visible_items = Vec::new();
        let query = self.search_query.to_lowercase();

        for col in &self.collections {
            if !query.is_empty() && self.focused_panel == FocusedPanel::Collections {
                if !col.name.to_lowercase().contains(&query) {
                    continue;
                }
            }

            visible_items.push(VisibleItem {
                name: col.name.clone(),
                depth: 0,
                item_type: VisibleItemType::Collection {
                    expanded: col.expanded,
                },
            });
            if col.expanded {
                for item in &col.items {
                    Self::collect_visible_items_recursive(item, 1, &mut visible_items);
                }
            }
        }
        visible_items
    }

    pub fn get_visible_items(&self) -> Vec<VisibleItem> {
        let mut visible_items = Vec::new();
        let query = self.search_query.to_lowercase();

        if let Some(col) = self.collections.get(self.active_collection_index) {
            let items = if let Some(folder_id) = &self.active_folder_id {
                Self::find_folder_items(&col.items, folder_id).unwrap_or(&col.items)
            } else {
                &col.items
            };

            if !query.is_empty() && self.focused_panel == FocusedPanel::Apis {
                for item in items {
                    Self::collect_filtered_items_recursive(item, 0, &query, &mut visible_items);
                }
            } else {
                for item in items {
                    Self::collect_visible_items_recursive(item, 0, &mut visible_items);
                }
            }
        }
        visible_items
    }

    fn collect_filtered_items_recursive(
        item: &CollectionItem,
        depth: usize,
        query: &str,
        visible_items: &mut Vec<VisibleItem>,
    ) {
        match item {
            CollectionItem::Folder(f) => {
                let matches = f.name.to_lowercase().contains(query);
                if matches {
                    visible_items.push(VisibleItem {
                        name: f.name.clone(),
                        depth,
                        item_type: VisibleItemType::Folder {
                            expanded: f.expanded,
                        },
                    });
                }
                for sub_item in &f.items {
                    Self::collect_filtered_items_recursive(
                        sub_item,
                        if matches { depth + 1 } else { depth },
                        query,
                        visible_items,
                    );
                }
            }
            CollectionItem::Request(r) => {
                if r.name.to_lowercase().contains(query) {
                    visible_items.push(VisibleItem {
                        name: r.name.clone(),
                        depth,
                        item_type: VisibleItemType::Request {
                            method: r.method,
                            id: r.id.clone(),
                        },
                    });
                }
            }
        }
    }

    fn find_folder_items<'a>(
        items: &'a [CollectionItem],
        folder_id: &str,
    ) -> Option<&'a Vec<CollectionItem>> {
        for item in items {
            if let CollectionItem::Folder(f) = item {
                if f.id == folder_id {
                    return Some(&f.items);
                }
                if let Some(found) = Self::find_folder_items(&f.items, folder_id) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn collect_visible_items_recursive(
        item: &CollectionItem,
        depth: usize,
        visible_items: &mut Vec<VisibleItem>,
    ) {
        match item {
            CollectionItem::Folder(f) => {
                visible_items.push(VisibleItem {
                    name: f.name.clone(),
                    depth,
                    item_type: VisibleItemType::Folder {
                        expanded: f.expanded,
                    },
                });
                if f.expanded {
                    for sub_item in &f.items {
                        Self::collect_visible_items_recursive(sub_item, depth + 1, visible_items);
                    }
                }
            }
            CollectionItem::Request(r) => {
                visible_items.push(VisibleItem {
                    name: r.name.clone(),
                    depth,
                    item_type: VisibleItemType::Request {
                        method: r.method,
                        id: r.id.clone(),
                    },
                });
            }
        }
    }

    pub fn toggle_folder(&mut self) {
        if self.focused_panel == FocusedPanel::Collections {
            let visible = self.get_visible_collections();
            if let Some(item) = visible.get(self.selected_collection_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_collection_index;
                for col in &mut self.collections {
                    if current_idx == target_idx {
                        col.expanded = !col.expanded;
                        return;
                    }
                    current_idx += 1;
                    if col.expanded {
                        for it in &mut col.items {
                            if Self::find_and_toggle_folder_recursive(
                                it,
                                1,
                                target_depth,
                                &target_name,
                                &mut current_idx,
                                target_idx,
                            ) {
                                return;
                            }
                        }
                    }
                }
            }
        } else {
            let visible = self.get_visible_items();
            if let Some(item) = visible.get(self.selected_api_index) {
                if let VisibleItemType::Folder { .. } = item.item_type {
                    let target_name = item.name.clone();
                    let target_depth = item.depth;
                    let mut current_idx = 0;
                    let target_idx = self.selected_api_index;
                    if let Some(col) = self.collections.get_mut(self.active_collection_index) {
                        for it in &mut col.items {
                            if Self::find_and_toggle_folder_recursive(
                                it,
                                0,
                                target_depth,
                                &target_name,
                                &mut current_idx,
                                target_idx,
                            ) {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn find_and_toggle_folder_recursive(
        item: &mut CollectionItem,
        current_depth: usize,
        target_depth: usize,
        target_name: &str,
        current_idx: &mut usize,
        target_idx: usize,
    ) -> bool {
        match item {
            CollectionItem::Folder(f) => {
                if current_depth == target_depth
                    && f.name == target_name
                    && *current_idx == target_idx
                {
                    f.expanded = !f.expanded;
                    return true;
                }
                *current_idx += 1;
                if f.expanded {
                    for sub in &mut f.items {
                        if Self::find_and_toggle_folder_recursive(
                            sub,
                            current_depth + 1,
                            target_depth,
                            target_name,
                            current_idx,
                            target_idx,
                        ) {
                            return true;
                        }
                    }
                }
            }
            CollectionItem::Request(_) => *current_idx += 1,
        }
        false
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

    pub fn rename_item(&mut self) {
        let new_name = self.rename_input.clone();
        if self.focused_panel == FocusedPanel::Collections {
            let visible = self.get_visible_collections();
            if let Some(item) = visible.get(self.selected_collection_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_collection_index;
                for col in &mut self.collections {
                    if current_idx == target_idx {
                        col.name = new_name;
                        return;
                    }
                    current_idx += 1;
                    if col.expanded {
                        for it in &mut col.items {
                            if Self::find_and_rename_recursive(
                                it,
                                1,
                                target_depth,
                                &target_name,
                                &mut current_idx,
                                target_idx,
                                &new_name,
                            ) {
                                return;
                            }
                        }
                    }
                }
            }
        } else {
            let visible = self.get_visible_items();
            if let Some(item) = visible.get(self.selected_api_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_api_index;
                if let Some(col) = self.collections.get_mut(self.active_collection_index) {
                    for it in &mut col.items {
                        if Self::find_and_rename_recursive(
                            it,
                            0,
                            target_depth,
                            &target_name,
                            &mut current_idx,
                            target_idx,
                            &new_name,
                        ) {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn find_and_rename_recursive(
        item: &mut CollectionItem,
        current_depth: usize,
        target_depth: usize,
        target_name: &str,
        current_idx: &mut usize,
        target_idx: usize,
        new_name: &str,
    ) -> bool {
        let name = match item {
            CollectionItem::Folder(f) => &mut f.name,
            CollectionItem::Request(r) => &mut r.name,
        };
        if current_depth == target_depth && *name == target_name && *current_idx == target_idx {
            *name = new_name.to_string();
            return true;
        }
        *current_idx += 1;
        if let CollectionItem::Folder(f) = item {
            if f.expanded {
                for sub in &mut f.items {
                    if Self::find_and_rename_recursive(
                        sub,
                        current_depth + 1,
                        target_depth,
                        target_name,
                        current_idx,
                        target_idx,
                        new_name,
                    ) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn delete_item(&mut self) {
        if self.focused_panel == FocusedPanel::Collections {
            let visible = self.get_visible_collections();
            if let Some(item) = visible.get(self.selected_collection_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_collection_index;
                let mut to_remove = None;
                for (i, col) in self.collections.iter_mut().enumerate() {
                    if current_idx == target_idx {
                        to_remove = Some(i);
                        break;
                    }
                    current_idx += 1;
                    if col.expanded {
                        let mut item_to_remove = None;
                        for (j, it) in col.items.iter_mut().enumerate() {
                            if Self::find_and_delete_recursive(
                                it,
                                1,
                                target_depth,
                                &target_name,
                                &mut current_idx,
                                target_idx,
                            ) {
                                item_to_remove = Some(j);
                                break;
                            }
                        }
                        if let Some(j) = item_to_remove {
                            col.items.remove(j);
                            return;
                        }
                    }
                }
                if let Some(i) = to_remove {
                    self.collections.remove(i);
                }
            }
        } else {
            let visible = self.get_visible_items();
            if let Some(item) = visible.get(self.selected_api_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_api_index;
                if let Some(col) = self.collections.get_mut(self.active_collection_index) {
                    let mut item_to_remove = None;
                    for (i, it) in col.items.iter_mut().enumerate() {
                        if Self::find_and_delete_recursive(
                            it,
                            0,
                            target_depth,
                            &target_name,
                            &mut current_idx,
                            target_idx,
                        ) {
                            item_to_remove = Some(i);
                            break;
                        }
                    }
                    if let Some(i) = item_to_remove {
                        col.items.remove(i);
                    }
                }
            }
        }
    }

    fn find_and_delete_recursive(
        item: &mut CollectionItem,
        current_depth: usize,
        target_depth: usize,
        target_name: &str,
        current_idx: &mut usize,
        target_idx: usize,
    ) -> bool {
        let name = match item {
            CollectionItem::Folder(f) => &f.name,
            CollectionItem::Request(r) => &r.name,
        };
        if current_depth == target_depth && *name == target_name && *current_idx == target_idx {
            return true;
        }
        *current_idx += 1;
        if let CollectionItem::Folder(f) = item {
            if f.expanded {
                let mut sub_to_remove = None;
                for (i, sub) in f.items.iter_mut().enumerate() {
                    if Self::find_and_delete_recursive(
                        sub,
                        current_depth + 1,
                        target_depth,
                        target_name,
                        current_idx,
                        target_idx,
                    ) {
                        sub_to_remove = Some(i);
                        break;
                    }
                }
                if let Some(i) = sub_to_remove {
                    f.items.remove(i);
                    return false; // Already handled
                }
            }
        }
        false
    }

    pub fn add_collection(&mut self, name: String) {
        let name = if name.is_empty() {
            "New Collection".to_string()
        } else {
            name
        };
        self.collections.push(Collection::new(name));
    }

    pub fn add_folder(&mut self, name: String) {
        let name = if name.is_empty() {
            "New Folder".to_string()
        } else {
            name
        };
        let new_folder = crate::core::collection::Folder::new(name);
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            col.items.push(CollectionItem::Folder(new_folder));
        }
    }

    pub fn add_request(&mut self, name: String) {
        let name = if name.is_empty() {
            "New Request".to_string()
        } else {
            name
        };
        let new_req = Request::new(name, Method::Get, "https://httpbin.org/get".to_string());
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            col.items.push(CollectionItem::Request(new_req));
        }
    }

    pub fn load_sample_data(&mut self) {
        let mut col = Collection::new("Sample Collection".to_string());
        let mut req = Request::new(
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

    pub fn reset_scroll(&mut self) {
        self.response_scroll = 0;
        self.response_horizontal_scroll = 0;
        self.details_scroll = 0;
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
            if self.focused_panel == FocusedPanel::RequestBar && self.active_request_part == RequestBarPart::Url {
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
            } else if self.focused_panel == FocusedPanel::Details || self.focused_panel == FocusedPanel::Environments {
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
