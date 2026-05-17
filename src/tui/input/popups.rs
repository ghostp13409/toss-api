use crate::tui::app::{App, InputMode, PendingItemType};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_command_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        KeyCode::Enter => {
            let cmd = app.command_input.clone();
            if let Some(path) = cmd.strip_prefix("import ") {
                app.import_collection(path);
            } else if let Some(path) = cmd.strip_prefix("parse ") {
                app.parse_project_tui(path);
            } else if cmd == "parse" {
                app.parse_project_tui("");
            } else if cmd == "env create" {
                app.create_smart_env();
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

pub fn handle_rename_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        KeyCode::Enter => {
            if app.rename_input.trim().is_empty() {
                app.error_message = Some("Name cannot be empty".to_string());
                return;
            }
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

pub fn handle_search_mode(app: &mut App, key: KeyEvent) {
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

pub fn handle_confirm_quit(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => app.should_quit = true,
        _ => app.input_mode = InputMode::Normal,
    }
}

pub fn handle_confirm_delete(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.delete_item();
            app.input_mode = InputMode::Normal;
        }
        _ => app.input_mode = InputMode::Normal,
    }
}

pub fn handle_create_item_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.input_mode = InputMode::Normal,
        KeyCode::Enter => {
            let name = app.rename_input.clone();
            if name.trim().is_empty()
                && !matches!(
                    app.pending_item_type,
                    Some(PendingItemType::Collection)
                        | Some(PendingItemType::Folder)
                        | Some(PendingItemType::Request)
                )
            {
                app.error_message = Some("Name cannot be empty".to_string());
                return;
            }
            match app.pending_item_type {
                Some(PendingItemType::Collection) => app.add_collection(name),
                Some(PendingItemType::Folder) => app.add_folder(name),
                Some(PendingItemType::Request) => app.add_request(name),
                Some(PendingItemType::KVParam) => app.add_kv_param(name),
                Some(PendingItemType::EnvVar) => app.add_env_var(name),
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

pub fn handle_help_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
            app.input_mode = InputMode::Normal
        }
        _ => {}
    }
}
