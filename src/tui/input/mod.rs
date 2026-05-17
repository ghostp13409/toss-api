use crate::tui::app::{App, FocusedPanel, InputMode, TuiAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub mod autocomplete;
pub mod editing;
pub mod normal;
pub mod popups;

use autocomplete::*;
use editing::*;
use normal::*;
use popups::*;

pub fn handle_input(app: &mut App, key: KeyEvent) {
    if let Some(_) = &app.error_message {
        app.error_message = None;
        return;
    }
    if app.show_autocomplete {
        handle_autocomplete_input(app, key);
        return;
    }
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
        InputMode::Normal => {
            handle_normal_mode(app, key);
            if key.code != KeyCode::Char('g') {
                app.g_pressed = false;
            }
        }
        InputMode::Editing => handle_editing_mode(app, key),
        InputMode::Command => handle_command_mode(app, key),
        InputMode::Rename => handle_rename_mode(app, key),
        InputMode::Search => handle_search_mode(app, key),
        InputMode::ConfirmQuit => handle_confirm_quit(app, key),
        InputMode::ConfirmDelete => handle_confirm_delete(app, key),
        InputMode::CreateItem => handle_create_item_mode(app, key),
        InputMode::Help => handle_help_mode(app, key),
    }
}
