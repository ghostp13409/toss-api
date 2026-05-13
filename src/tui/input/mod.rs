use crate::cli::args::Method;
use crate::tui::app::{
    App, FocusedPanel, InputMode, PendingItemType, PropertyEditorField, PropertyTab,
    RequestBarPart, TuiAction,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_input(app: &mut App, key: KeyEvent) {
    if app.show_method_search {
        handle_method_search_input(app, key);
        return;
    }

    // Global keys
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && (key.code == KeyCode::Enter || key.code == KeyCode::Char('s'))
    {
        app.pending_actions.push(TuiAction::SendRequest);
        app.focused_panel = FocusedPanel::Response;
        return;
    }

    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Editing => handle_editing_mode(app, key),
        InputMode::Command => handle_command_mode(app, key),
        InputMode::Rename => handle_rename_mode(app, key),
        InputMode::Search => handle_search_mode(app, key),
        InputMode::ConfirmQuit => handle_confirm_quit(app, key),
        InputMode::ConfirmDelete => handle_confirm_delete(app, key),
        InputMode::CreateItem => handle_create_item_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('C') => app.focused_panel = FocusedPanel::Collections,
        KeyCode::Char('A') => app.focused_panel = FocusedPanel::Apis,
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

        KeyCode::Char('q') => app.input_mode = InputMode::ConfirmQuit,
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
                        _ => 0,
                    };
                    if app.property_editor_row < max_rows.saturating_sub(1) {
                        app.property_editor_row += 1;
                    }
                }
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
            FocusedPanel::Details => {
                if app.property_editor_row > 0 {
                    app.property_editor_row -= 1;
                }
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
                                && let Some(req) = col.find_request_mut(&id_clone)
                            {
                                app.url = req.url.clone();
                            }
                            app.focus_request_bar();
                            app.cursor_position = app.url.len();
                        }
                        _ => {}
                    }
                }
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
            FocusedPanel::RequestBar => {
                app.active_request_part = match app.active_request_part {
                    RequestBarPart::Method => RequestBarPart::Send,
                    RequestBarPart::Url => RequestBarPart::Method,
                    RequestBarPart::Send => RequestBarPart::Url,
                };
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

fn handle_editing_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.save_current_request();
            app.input_mode = InputMode::Normal;
            if app.focused_panel == FocusedPanel::Details
                && app.selected_property_tab == PropertyTab::Auth
            {
                app.property_editor_field = PropertyEditorField::Key;
            }
        }
        KeyCode::Enter => {
            if app.focused_panel == FocusedPanel::RequestBar
                && app.active_request_part == RequestBarPart::Url
            {
                app.save_current_request();
                app.active_request_part = RequestBarPart::Send;
                app.input_mode = InputMode::Normal;
            } else if app.focused_panel == FocusedPanel::Details {
                app.save_current_request();
                match app.selected_property_tab {
                    PropertyTab::Auth => {
                        app.input_mode = InputMode::Normal;
                        app.property_editor_field = PropertyEditorField::Key;
                    }
                    PropertyTab::Params | PropertyTab::Headers | PropertyTab::Body => {
                        match app.property_editor_field {
                            PropertyEditorField::Key => {
                                app.property_editor_field = PropertyEditorField::Value;
                                let current_val = app.get_kv_editor_value();
                                app.cursor_position = current_val.len();
                            }
                            PropertyEditorField::Value => {
                                app.property_editor_field = PropertyEditorField::Description;
                                let current_val = app.get_kv_editor_value();
                                app.cursor_position = current_val.len();
                            }
                            PropertyEditorField::Description => {
                                app.input_mode = InputMode::Normal;
                                app.property_editor_field = PropertyEditorField::Key;
                            }
                        }
                    }
                    _ => app.pop_up(),
                }
            } else {
                app.save_current_request();
                app.pop_up();
            }
        }
        KeyCode::Tab => {
            if app.focused_panel == FocusedPanel::Details {
                if app.selected_property_tab == PropertyTab::Auth {
                    // Don't cycle fields in Auth tab, just keep focus on Value
                    app.property_editor_field = PropertyEditorField::Value;
                } else {
                    app.property_editor_field = match app.property_editor_field {
                        PropertyEditorField::Key => PropertyEditorField::Value,
                        PropertyEditorField::Value => PropertyEditorField::Description,
                        PropertyEditorField::Description => PropertyEditorField::Key,
                    };
                }
                let current_val = app.get_kv_editor_value();
                app.cursor_position = current_val.len();
            } else {
                app.save_current_request();
                app.next_panel();
                if app.active_request_part == RequestBarPart::Url {
                    app.cursor_position = app.url.len();
                }
            }
        }
        KeyCode::Char(c) => {
            if app.focused_panel == FocusedPanel::RequestBar
                && app.active_request_part == RequestBarPart::Url
            {
                app.insert_char_url(c);
                app.sync_params_from_url();
            } else if app.focused_panel == FocusedPanel::Details {
                let mut current_val = app.get_kv_editor_value();
                app.insert_char(&mut current_val, c);
                app.update_kv_param(current_val);
            }
        }
        KeyCode::Backspace => {
            if app.focused_panel == FocusedPanel::RequestBar
                && app.active_request_part == RequestBarPart::Url
            {
                app.delete_char_url();
                app.sync_params_from_url();
            } else if app.focused_panel == FocusedPanel::Details {
                let mut current_val = app.get_kv_editor_value();
                app.delete_char(&mut current_val);
                app.update_kv_param(current_val);
            }
        }
        KeyCode::Delete => {
            if app.focused_panel == FocusedPanel::RequestBar
                && app.active_request_part == RequestBarPart::Url
            {
                app.delete_char_forward_url();
                app.sync_params_from_url();
            } else if app.focused_panel == FocusedPanel::Details {
                let mut current_val = app.get_kv_editor_value();
                app.delete_char_forward(&mut current_val);
                app.update_kv_param(current_val);
            }
        }
        KeyCode::Left => app.move_cursor_left(),
        KeyCode::Right => {
            let max = if app.focused_panel == FocusedPanel::Details {
                app.get_kv_editor_value().len()
            } else {
                app.url.len()
            };
            app.move_cursor_right(max);
        }
        KeyCode::Home => app.cursor_position = 0,
        KeyCode::End => {
            app.cursor_position = if app.focused_panel == FocusedPanel::Details {
                app.get_kv_editor_value().len()
            } else {
                app.url.len()
            };
        }
        _ => {}
    }
}

fn handle_method_search_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.show_method_search = false;
            app.method_search_query.clear();
        }
        KeyCode::Enter => {
            let all_methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE"];
            let filtered: Vec<&str> = all_methods
                .into_iter()
                .filter(|m| m.contains(&app.method_search_query.to_uppercase()))
                .collect();

            if let Some(first) = filtered.first() {
                app.method = match *first {
                    "GET" => Method::Get,
                    "POST" => Method::Post,
                    "PUT" => Method::Put,
                    "PATCH" => Method::Patch,
                    "DELETE" => Method::Delete,
                    _ => Method::Get,
                };
            }
            app.show_method_search = false;
            app.method_search_query.clear();
        }
        KeyCode::Char(c) => {
            app.method_search_query.push(c);
        }
        KeyCode::Backspace => {
            app.method_search_query.pop();
        }
        _ => {}
    }
}

fn handle_command_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        KeyCode::Enter => {
            let cmd = app.command_input.clone();
            if let Some(path) = cmd.strip_prefix("import ") {
                app.import_collection(path);
            } else {
                match cmd.as_str() {
                    "q" | "quit" => app.should_quit = true,
                    _ => {}
                }
            }
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
            app.cursor_position += 1;
        }
        KeyCode::Backspace => {
            app.command_input.pop();
            app.cursor_position = app.cursor_position.saturating_sub(1);
        }
        _ => {}
    }
}

fn handle_rename_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        KeyCode::Enter => {
            app.rename_item();
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.insert_char_rename(c);
        }
        KeyCode::Backspace => {
            app.delete_char_rename();
        }
        KeyCode::Delete => {
            app.delete_char_forward_rename();
        }
        KeyCode::Left => app.move_cursor_left(),
        KeyCode::Right => {
            let max = app.rename_input.len();
            app.move_cursor_right(max);
        }
        _ => {}
    }
}

fn handle_search_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.search_query.clear();
            app.show_search = false;
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Enter => {
            app.show_search = false;
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.cursor_position += 1;
            app.clamp_selections();
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.cursor_position = app.cursor_position.saturating_sub(1);
            app.clamp_selections();
        }
        _ => {}
    }
}

fn handle_confirm_quit(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => app.should_quit = true,
        _ => app.input_mode = InputMode::Normal,
    }
}

fn handle_confirm_delete(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.delete_item();
            app.input_mode = InputMode::Normal;
        }
        _ => app.input_mode = InputMode::Normal,
    }
}

fn handle_create_item_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        KeyCode::Enter => {
            let name = app.rename_input.clone();
            match app.pending_item_type {
                Some(PendingItemType::Collection) => app.add_collection(name),
                Some(PendingItemType::Folder) => app.add_folder(name),
                Some(PendingItemType::Request) => app.add_request(name),
                None => {}
            }
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.insert_char_rename(c);
        }
        KeyCode::Backspace => {
            app.delete_char_rename();
        }
        KeyCode::Delete => {
            app.delete_char_forward_rename();
        }
        KeyCode::Left => app.move_cursor_left(),
        KeyCode::Right => {
            let max = app.rename_input.len();
            app.move_cursor_right(max);
        }
        _ => {}
    }
}
