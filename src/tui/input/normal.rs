use crate::tui::app::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('C') => app.focused_panel = FocusedPanel::Collections,
        KeyCode::Char('R') => {
            app.focus_request_bar();
            app.cursor_position = app.url.len();
        }
        KeyCode::Char('P') => {
            if app.current_request_id.is_some() {
                app.selected_property_tab = PropertyTab::Params;
                app.focused_panel = FocusedPanel::Details;
            }
        }
        KeyCode::Char('H') => {
            if app.current_request_id.is_some() {
                app.selected_property_tab = PropertyTab::Headers;
                app.focused_panel = FocusedPanel::Details;
            }
        }
        KeyCode::Char('U') => {
            if app.current_request_id.is_some() {
                app.selected_property_tab = PropertyTab::Auth;
                app.focused_panel = FocusedPanel::Details;
            }
        }
        KeyCode::Char('B') => {
            if app.current_request_id.is_some() {
                app.selected_property_tab = PropertyTab::Body;
                app.focused_panel = FocusedPanel::Details;
            }
        }
        KeyCode::Char('S') => {
            if app.current_request_id.is_some() {
                app.selected_property_tab = PropertyTab::Scripts;
                app.focused_panel = FocusedPanel::Details;
            }
        }
        KeyCode::Char('E') => app.focused_panel = FocusedPanel::Response,
        KeyCode::Char('T') => app.focused_panel = FocusedPanel::Stats,
        KeyCode::Char('V') => {
            app.left_bottom_tab = crate::tui::app::LeftBottomTab::Environments;
            app.focused_panel = FocusedPanel::Environments;
        }
        KeyCode::Char('A') => {
            app.left_bottom_tab = crate::tui::app::LeftBottomTab::Apis;
            app.focused_panel = FocusedPanel::Apis;
        }
        KeyCode::Char('m') => {
            if app.focused_panel == FocusedPanel::Environments {
                app.toggle_env_mask();
            }
        }

        // gg and G shortcuts
        KeyCode::Char('g') => {
            if app.g_pressed {
                match app.focused_panel {
                    FocusedPanel::Collections => app.selected_collection_index = 0,
                    FocusedPanel::Apis => app.selected_api_index = 0,
                    FocusedPanel::Environments => app.selected_env_index = 0,
                    FocusedPanel::Details => app.property_editor_row = 0,
                    FocusedPanel::Response | FocusedPanel::Stats => app.response_scroll = 0,
                    _ => {}
                }
                app.g_pressed = false;
            } else {
                app.g_pressed = true;
            }
            return;
        }
        KeyCode::Char('G') => {
            match app.focused_panel {
                FocusedPanel::Collections => {
                    app.selected_collection_index =
                        app.get_visible_collections().len().saturating_sub(1)
                }
                FocusedPanel::Apis => {
                    app.selected_api_index = app.get_visible_items().len().saturating_sub(1)
                }
                FocusedPanel::Environments => {
                    app.selected_env_index =
                        app.get_active_collection_env_vars().len().saturating_sub(1)
                }
                FocusedPanel::Details => {
                    if let Some(req) = app.get_current_request() {
                        let max_rows = match app.selected_property_tab {
                            PropertyTab::Params => req.params.len(),
                            PropertyTab::Headers => req.headers.len(),
                            PropertyTab::Auth => match &req.auth {
                                crate::core::collection::Auth::None => 0,
                                crate::core::collection::Auth::Bearer { .. } => 1,
                                crate::core::collection::Auth::Basic { .. } => 2,
                                crate::core::collection::Auth::ApiKey { .. } => 3,
                            },
                            PropertyTab::Body => match &req.body {
                                crate::core::collection::RequestBody::FormData { items } => {
                                    items.len()
                                }
                                crate::core::collection::RequestBody::XWwwFormUrlEncoded {
                                    items,
                                } => items.len(),
                                _ => 0,
                            },
                            _ => 0,
                        };
                        app.property_editor_row = max_rows.saturating_sub(1);
                    }
                }
                FocusedPanel::Response | FocusedPanel::Stats => {
                    // Approximate bottom by setting high scroll
                    app.response_scroll = 1000;
                }
                _ => {}
            }
        }

        // Page Up/Down (Ctrl-u / Ctrl-d)
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            match app.focused_panel {
                FocusedPanel::Collections => {
                    app.selected_collection_index =
                        app.selected_collection_index.saturating_sub(10);
                }
                FocusedPanel::Apis => {
                    app.selected_api_index = app.selected_api_index.saturating_sub(10);
                }
                FocusedPanel::Details => {
                    app.property_editor_row = app.property_editor_row.saturating_sub(10);
                    app.details_scroll = app.details_scroll.saturating_sub(10);
                }
                FocusedPanel::Response | FocusedPanel::Stats => {
                    app.response_scroll = app.response_scroll.saturating_sub(10);
                }
                _ => {}
            }
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            match app.focused_panel {
                FocusedPanel::Collections => {
                    let max = app.get_visible_collections().len().saturating_sub(1);
                    app.selected_collection_index = (app.selected_collection_index + 10).min(max);
                }
                FocusedPanel::Apis => {
                    let max = app.get_visible_items().len().saturating_sub(1);
                    app.selected_api_index = (app.selected_api_index + 10).min(max);
                }
                FocusedPanel::Details => {
                    app.property_editor_row += 10; // Capped in render/logic later
                    app.details_scroll = app.details_scroll.saturating_add(10);
                }
                FocusedPanel::Response | FocusedPanel::Stats => {
                    app.response_scroll = app.response_scroll.saturating_add(10);
                }
                _ => {}
            }
        }

        KeyCode::Char('q') => app.input_mode = InputMode::ConfirmQuit,
        KeyCode::Char('?') => app.input_mode = InputMode::Help,
        KeyCode::Tab => app.next_panel(),
        KeyCode::BackTab => app.prev_panel(),

        // Navigation Down
        KeyCode::Char('j') | KeyCode::Down => match app.focused_panel {
            FocusedPanel::Collections => {
                let max_idx = app.get_visible_collections().len().saturating_sub(1);
                if app.selected_collection_index < max_idx {
                    app.selected_collection_index += 1;
                    app.update_active_scope_from_tree();
                }
            }
            FocusedPanel::Apis => {
                let visible_items = app.get_visible_items();
                if app.selected_api_index < visible_items.len().saturating_sub(1) {
                    app.selected_api_index += 1;
                }
            }
            FocusedPanel::Environments => {
                let env_vars = app.get_active_collection_env_vars();
                if app.selected_env_index < env_vars.len().saturating_sub(1) {
                    app.selected_env_index += 1;
                }
            }
            FocusedPanel::Details => {
                if let Some(req) = app.get_current_request() {
                    let is_kv_tab = matches!(
                        app.selected_property_tab,
                        PropertyTab::Params | PropertyTab::Headers | PropertyTab::Auth
                    ) || (app.selected_property_tab == PropertyTab::Body
                        && matches!(
                            req.body,
                            crate::core::collection::RequestBody::FormData { .. }
                                | crate::core::collection::RequestBody::XWwwFormUrlEncoded { .. }
                        ));

                    if is_kv_tab {
                        let max_rows = match app.selected_property_tab {
                            PropertyTab::Params => req.params.len(),
                            PropertyTab::Headers => req.headers.len(),
                            PropertyTab::Auth => match &req.auth {
                                crate::core::collection::Auth::None => 0,
                                crate::core::collection::Auth::Bearer { .. } => 1,
                                crate::core::collection::Auth::Basic { .. } => 2,
                                crate::core::collection::Auth::ApiKey { .. } => 3,
                            },
                            PropertyTab::Body => match &req.body {
                                crate::core::collection::RequestBody::FormData { items } => {
                                    items.len()
                                }
                                crate::core::collection::RequestBody::XWwwFormUrlEncoded {
                                    items,
                                } => items.len(),
                                _ => 0,
                            },
                            _ => 0,
                        };
                        if app.property_editor_row < max_rows.saturating_sub(1) {
                            app.property_editor_row += 1;
                        }
                    } else {
                        app.details_scroll = app.details_scroll.saturating_add(1);
                    }
                }
            }
            FocusedPanel::Response | FocusedPanel::Stats => {
                app.response_scroll = app.response_scroll.saturating_add(1);
            }
            _ => {}
        },

        // Navigation Up
        KeyCode::Char('k') | KeyCode::Up => match app.focused_panel {
            FocusedPanel::Collections => {
                if app.selected_collection_index > 0 {
                    app.selected_collection_index -= 1;
                    app.update_active_scope_from_tree();
                }
            }
            FocusedPanel::Apis => {
                if app.selected_api_index > 0 {
                    app.selected_api_index -= 1;
                }
            }
            FocusedPanel::Environments => {
                if app.selected_env_index > 0 {
                    app.selected_env_index -= 1;
                }
            }
            FocusedPanel::Details => {
                if let Some(req) = app.get_current_request() {
                    let is_kv_tab = matches!(
                        app.selected_property_tab,
                        PropertyTab::Params | PropertyTab::Headers | PropertyTab::Auth
                    ) || (app.selected_property_tab == PropertyTab::Body
                        && matches!(
                            req.body,
                            crate::core::collection::RequestBody::FormData { .. }
                                | crate::core::collection::RequestBody::XWwwFormUrlEncoded { .. }
                        ));

                    if is_kv_tab {
                        if app.property_editor_row > 0 {
                            app.property_editor_row -= 1;
                        }
                    } else {
                        app.details_scroll = app.details_scroll.saturating_sub(1);
                    }
                }
            }
            FocusedPanel::Response | FocusedPanel::Stats => {
                app.response_scroll = app.response_scroll.saturating_sub(1);
            }
            _ => {}
        },

        // Drill down / Enter
        KeyCode::Enter => match app.focused_panel {
            FocusedPanel::Collections => {
                let visible_collections = app.get_visible_collections();
                if let Some(item) = visible_collections.get(app.selected_collection_index) {
                    match &item.item_type {
                        crate::tui::app::VisibleItemType::Collection { .. }
                        | crate::tui::app::VisibleItemType::Folder { .. } => {
                            app.toggle_folder();
                        }
                        crate::tui::app::VisibleItemType::Request { method, id, .. } => {
                            app.save_current_request();
                            app.current_request_id = Some(id.clone());
                            app.method = *method;
                            let id_clone = id.clone();
                            for col in &mut app.collections {
                                if let Some(req) = col.find_request_mut(&id_clone) {
                                    app.url = req.url.clone();
                                    break;
                                }
                            }
                            app.focus_request_bar();
                            app.cursor_position = app.url.len();
                            app.reset_scroll();
                        }
                    }
                }
            }
            FocusedPanel::Apis => {
                let visible_items = app.get_visible_items();
                if let Some(item) = visible_items.get(app.selected_api_index) {
                    match &item.item_type {
                        crate::tui::app::VisibleItemType::Folder { .. } => {
                            app.toggle_folder();
                        }
                        crate::tui::app::VisibleItemType::Request { method, id, .. } => {
                            app.save_current_request();
                            app.current_request_id = Some(id.clone());
                            app.method = *method;
                            let id_clone = id.clone();
                            if let Some(col) = app.collections.get_mut(app.active_collection_index)
                            {
                                if let Some(req) = col.find_request_mut(&id_clone) {
                                    app.url = req.url.clone();
                                }
                            }
                            app.focus_request_bar();
                            app.cursor_position = app.url.len();
                            app.reset_scroll();
                        }
                        _ => {}
                    }
                }
            }
            FocusedPanel::Environments => {
                app.input_mode = InputMode::Editing;
                app.property_editor_field = PropertyEditorField::Value;
                let current_val = app.get_env_editor_value();
                app.cursor_position = current_val.len();
            }
            FocusedPanel::Properties => {
                app.focused_panel = FocusedPanel::Details;
                // For Auth, always start on the field name (Key)
                if app.selected_property_tab == PropertyTab::Auth {
                    app.property_editor_field = PropertyEditorField::Key;
                }
            }
            FocusedPanel::Details => {
                if app.selected_property_tab == PropertyTab::Auth {
                    if let Some(req) = app.get_current_request() {
                        if let crate::core::collection::Auth::ApiKey { .. } = &req.auth {
                            if app.property_editor_row == 2 {
                                app.toggle_auth_bool();
                                return;
                            }
                        }
                    }
                    // Always switch to Value field when starting to edit Auth
                    app.property_editor_field = PropertyEditorField::Value;
                }
                app.input_mode = InputMode::Editing;
                let current_val = app.get_kv_editor_value();
                app.cursor_position = current_val.len();
            }
            FocusedPanel::RequestBar => match app.active_request_part {
                RequestBarPart::Method => {
                    app.show_method_search = true;
                    app.method_search_query.clear();
                    app.cursor_position = 0;
                }
                RequestBarPart::Url => {
                    app.input_mode = InputMode::Editing;
                    app.cursor_position = app.url.len();
                }
                RequestBarPart::Send => {
                    app.pending_actions.push(TuiAction::SendRequest);
                    app.focused_panel = FocusedPanel::Response;
                }
            },
            _ => {}
        },

        // Move Right / Next Tab
        KeyCode::Char('l') | KeyCode::Right => match app.focused_panel {
            FocusedPanel::Properties => {
                app.next_property_tab();
            }
            FocusedPanel::Details => match app.selected_property_tab {
                PropertyTab::Params | PropertyTab::Headers => {
                    app.property_editor_field = match app.property_editor_field {
                        PropertyEditorField::Key => PropertyEditorField::Value,
                        PropertyEditorField::Value => PropertyEditorField::Description,
                        PropertyEditorField::Description => PropertyEditorField::Description,
                    };
                }
                _ => {}
            },
            FocusedPanel::Environments => {
                app.property_editor_field = match app.property_editor_field {
                    PropertyEditorField::Key => PropertyEditorField::Value,
                    PropertyEditorField::Value => PropertyEditorField::Value,
                    _ => PropertyEditorField::Value,
                };
            }
            FocusedPanel::RequestBar => {
                app.active_request_part = match app.active_request_part {
                    RequestBarPart::Method => RequestBarPart::Url,
                    RequestBarPart::Url => RequestBarPart::Send,
                    RequestBarPart::Send => RequestBarPart::Method,
                };
            }
            FocusedPanel::Collections => {
                app.focused_panel = FocusedPanel::Apis;
            }
            FocusedPanel::Apis => {
                app.focused_panel = FocusedPanel::RequestBar;
            }
            FocusedPanel::Response => {
                app.response_horizontal_scroll = app.response_horizontal_scroll.saturating_add(1);
            }
            _ => {
                app.next_panel();
            }
        },

        // Move Left / Prev Tab
        KeyCode::Char('h') | KeyCode::Left => match app.focused_panel {
            FocusedPanel::Properties => {
                app.prev_property_tab();
            }
            FocusedPanel::Details => match app.selected_property_tab {
                PropertyTab::Params | PropertyTab::Headers => {
                    app.property_editor_field = match app.property_editor_field {
                        PropertyEditorField::Key => PropertyEditorField::Key,
                        PropertyEditorField::Value => PropertyEditorField::Key,
                        PropertyEditorField::Description => PropertyEditorField::Value,
                    };
                }
                _ => {}
            },
            FocusedPanel::Environments => {
                app.property_editor_field = match app.property_editor_field {
                    PropertyEditorField::Key => PropertyEditorField::Key,
                    PropertyEditorField::Value => PropertyEditorField::Key,
                    _ => PropertyEditorField::Key,
                };
            }
            FocusedPanel::RequestBar => {
                app.active_request_part = match app.active_request_part {
                    RequestBarPart::Method => RequestBarPart::Send,
                    RequestBarPart::Url => RequestBarPart::Method,
                    RequestBarPart::Send => RequestBarPart::Url,
                };
            }
            FocusedPanel::Response => {
                app.response_horizontal_scroll = app.response_horizontal_scroll.saturating_sub(1);
            }
            _ => {
                app.pop_up();
            }
        },

        // Pop up explicitly
        KeyCode::Esc => {
            app.pop_up();
        }

        KeyCode::Char(' ') => {
            if app.focused_panel == FocusedPanel::Apis
                || app.focused_panel == FocusedPanel::Collections
            {
                app.toggle_folder();
            } else if app.focused_panel == FocusedPanel::Details {
                app.toggle_kv_param();
            }
        }

        KeyCode::Char('/') => {
            if app.focused_panel == FocusedPanel::Apis
                || app.focused_panel == FocusedPanel::Collections
            {
                app.input_mode = InputMode::Search;
                app.show_search = true;
                app.search_query.clear();
                app.cursor_position = 0;
            }
        }

        KeyCode::Char('e') => {
            app.focus_request_bar();
            app.cursor_position = app.url.len();
        }

        KeyCode::Char('a') => {
            if app.focused_panel == FocusedPanel::Apis
                || app.focused_panel == FocusedPanel::Collections
            {
                app.input_mode = InputMode::CreateItem;
                app.pending_item_type = Some(PendingItemType::Request);
                app.rename_input.clear();
                app.cursor_position = 0;
            } else if app.focused_panel == FocusedPanel::Details {
                app.add_kv_param();
                app.input_mode = InputMode::Editing;
                app.property_editor_field = PropertyEditorField::Key;
                app.cursor_position = 0;
            } else if app.focused_panel == FocusedPanel::Environments {
                app.add_env_var();
                app.input_mode = InputMode::Editing;
                app.property_editor_field = PropertyEditorField::Key;
                let current_val = app.get_env_editor_value();
                app.cursor_position = current_val.len();
            }
        }

        KeyCode::Char('f') => {
            if app.focused_panel == FocusedPanel::Apis
                || app.focused_panel == FocusedPanel::Collections
            {
                app.input_mode = InputMode::CreateItem;
                app.pending_item_type = Some(PendingItemType::Folder);
                app.rename_input.clear();
                app.cursor_position = 0;
            }
        }

        KeyCode::Char('t') => {
            if app.focused_panel == FocusedPanel::Details {
                match app.selected_property_tab {
                    PropertyTab::Auth => app.cycle_auth_type(),
                    PropertyTab::Body => app.cycle_body_type(),
                    _ => {}
                }
            }
        }

        KeyCode::Char('n') => {
            if app.focused_panel == FocusedPanel::Collections {
                app.input_mode = InputMode::CreateItem;
                app.pending_item_type = Some(PendingItemType::Collection);
                app.rename_input.clear();
                app.cursor_position = 0;
            }
        }

        KeyCode::Char('d') => {
            if app.focused_panel == FocusedPanel::Details {
                app.delete_kv_param();
            } else if app.focused_panel == FocusedPanel::Environments {
                app.delete_env_var();
            } else {
                app.input_mode = InputMode::ConfirmDelete;
            }
        }

        KeyCode::Char('r') => {
            if app.focused_panel == FocusedPanel::Apis
                || app.focused_panel == FocusedPanel::Collections
            {
                app.input_mode = InputMode::Rename;
                app.rename_input.clear();
                if app.focused_panel == FocusedPanel::Collections {
                    let visible_collections = app.get_visible_collections();
                    if let Some(item) = visible_collections.get(app.selected_collection_index) {
                        app.rename_input = item.name.clone();
                    }
                } else if app.focused_panel == FocusedPanel::Apis {
                    let visible_items = app.get_visible_items();
                    if let Some(item) = visible_items.get(app.selected_api_index) {
                        app.rename_input = item.name.clone();
                    }
                }
                app.cursor_position = app.rename_input.len();
            } else if app.focused_panel == FocusedPanel::Environments {
                app.input_mode = InputMode::Editing;
                app.property_editor_field = PropertyEditorField::Key;
                let current_val = app.get_env_editor_value();
                app.cursor_position = current_val.len();
            }
        }

        KeyCode::Char('v') => {
            if app.focused_panel == FocusedPanel::Details
                && app.selected_property_tab == PropertyTab::Body
            {
                app.pending_actions.push(TuiAction::EditBody);
            }
        }

        KeyCode::Char('y') => {
            if app.focused_panel == FocusedPanel::Details
                && app.selected_property_tab == PropertyTab::Body
            {
                app.pending_actions.push(TuiAction::CopyBody);
            } else if app.focused_panel == FocusedPanel::Response {
                app.pending_actions.push(TuiAction::CopyResponseBody);
            }
        }

        KeyCode::Char('p') => {
            if app.focused_panel == FocusedPanel::Details
                && app.selected_property_tab == PropertyTab::Body
            {
                app.pending_actions.push(TuiAction::PasteBody);
            }
        }

        KeyCode::Char('Y') => {
            if app.focused_panel == FocusedPanel::Response {
                app.pending_actions.push(TuiAction::CopyResponseAll);
            }
        }

        KeyCode::Char(':') => {
            app.input_mode = InputMode::Command;
            app.command_input.clear();
            app.cursor_position = 0;
        }

        KeyCode::Char('i') => {
            if app.focused_panel == FocusedPanel::Details {
                match app.selected_property_tab {
                    PropertyTab::Params | PropertyTab::Headers | PropertyTab::Auth => {
                        if app.selected_property_tab == PropertyTab::Auth {
                            if let Some(req) = app.get_current_request() {
                                if let crate::core::collection::Auth::ApiKey { .. } = &req.auth {
                                    if app.property_editor_row == 2 {
                                        // It's a boolean, don't enter editing mode, just toggle
                                        app.toggle_auth_bool();
                                        return;
                                    }
                                }
                            }
                            app.property_editor_field = PropertyEditorField::Value;
                        }
                        app.input_mode = InputMode::Editing;
                        let current_val = app.get_kv_editor_value();
                        app.cursor_position = current_val.len();
                    }
                    _ => {}
                }
            }
        }

        _ => {}
    }
}
