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
            } else if let Some(args) = cmd.strip_prefix("export ") {
                let parts: Vec<&str> = args.split_whitespace().collect();
                if parts.len() == 1 {
                    app.export_active_collection(parts[0], None);
                } else if parts.len() >= 2 {
                    app.export_active_collection(parts[0], Some(parts[1]));
                }
            } else if cmd == "export" {
                app.export_active_collection("postman", None);
            } else if cmd == "console" {
                app.show_console = !app.show_console;
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
            app.help_scroll = 0;
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.help_scroll = app.help_scroll.saturating_add(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.help_scroll = app.help_scroll.saturating_sub(1);
        }
        _ => {}
    }
}

pub fn handle_console_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.show_console = false;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.console_scroll = app.console_scroll.saturating_add(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.console_scroll = app.console_scroll.saturating_sub(1);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::core::collection::Collection;

    #[test]
    fn test_handle_command_mode_export() {
        let mut app = App::new();
        app.collections.push(Collection::new("Command Mode Col".to_string()));
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("cmd_export_out.json");
        let path_str = file_path.to_str().unwrap();

        app.command_input = format!("export openapi {}", path_str);
        handle_command_mode(
            &mut app,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert!(file_path.exists());
        assert_eq!(app.input_mode, InputMode::Normal);
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_show_console_toggle() {
        let mut app = App::new();
        app.show_console = false;

        // Toggle on
        app.command_input = "console".to_string();
        handle_command_mode(
            &mut app,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        assert!(app.show_console);

        // Toggle off
        app.command_input = "console".to_string();
        handle_command_mode(
            &mut app,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        assert!(!app.show_console);
    }
}
