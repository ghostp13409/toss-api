use crate::cli::args::Method;
use crate::tui::app::{App, FocusedPanel, RequestBarPart};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_autocomplete_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.show_autocomplete = false;
            app.autocomplete_query.clear();
        }
        KeyCode::Enter => {
            app.insert_autocomplete_selection();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let options = app.get_autocomplete_options();
            if !options.is_empty() {
                app.autocomplete_index = (app.autocomplete_index + 1) % options.len();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let options = app.get_autocomplete_options();
            if !options.is_empty() {
                if app.autocomplete_index == 0 {
                    app.autocomplete_index = options.len() - 1;
                } else {
                    app.autocomplete_index -= 1;
                }
            }
        }
        KeyCode::Char(c) => {
            app.autocomplete_query.push(c);
            app.autocomplete_index = 0;
            
            if app.focused_panel == FocusedPanel::RequestBar && app.active_request_part == RequestBarPart::Url {
                app.insert_char_url(c);
                app.sync_params_from_url();
            } else if app.focused_panel == FocusedPanel::Details {
                let mut val = app.get_kv_editor_value();
                app.insert_char(&mut val, c);
                app.update_kv_param(val);
            } else if app.focused_panel == FocusedPanel::Environments {
                let mut val = app.get_env_editor_value();
                app.insert_char(&mut val, c);
                app.update_env_editor_value(val);
            }
        }
        KeyCode::Backspace => {
            let query_is_empty = app.autocomplete_query.is_empty();
            if !query_is_empty {
                app.autocomplete_query.pop();
                app.autocomplete_index = 0;
            }

            if app.focused_panel == FocusedPanel::RequestBar && app.active_request_part == RequestBarPart::Url {
                app.delete_char_url();
                app.sync_params_from_url();
                if app.url[..app.cursor_position].ends_with("{{") {
                    app.show_autocomplete = true;
                } else if query_is_empty {
                    app.show_autocomplete = false;
                }
            } else if app.focused_panel == FocusedPanel::Details {
                let mut val = app.get_kv_editor_value();
                app.delete_char(&mut val);
                app.update_kv_param(val.clone());
                if val[..app.cursor_position].ends_with("{{") {
                    app.show_autocomplete = true;
                } else if query_is_empty {
                    app.show_autocomplete = false;
                }
            } else if app.focused_panel == FocusedPanel::Environments {
                let mut val = app.get_env_editor_value();
                app.delete_char(&mut val);
                app.update_env_editor_value(val.clone());
                if val[..app.cursor_position].ends_with("{{") {
                    app.show_autocomplete = true;
                } else if query_is_empty {
                    app.show_autocomplete = false;
                }
            }
        }
        _ => {}
    }
}

pub fn handle_method_search_input(app: &mut App, key: KeyEvent) {
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
