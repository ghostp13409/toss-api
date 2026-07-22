use crate::tui::app::{App, InputMode, PropertyTab, ScriptsSubTab};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_body_editor_input(app: &mut App, key: KeyEvent) {
    let is_scripts = app.focused_panel == crate::tui::app::FocusedPanel::Details
        && app.selected_property_tab == PropertyTab::Scripts;

    let is_normal_mode = if is_scripts {
        match app.scripts_subtab {
            ScriptsSubTab::PreRequest => app.pre_request_editor_state.mode == edtui::EditorMode::Normal,
            ScriptsSubTab::PostResponse => app.post_response_editor_state.mode == edtui::EditorMode::Normal,
        }
    } else {
        app.body_editor_state.mode == edtui::EditorMode::Normal
    };

    if key.code == KeyCode::Esc && is_normal_mode {
        // Save and exit
        if is_scripts {
            let new_content: String = match app.scripts_subtab {
                ScriptsSubTab::PreRequest => app.pre_request_editor_state.lines.clone().into(),
                ScriptsSubTab::PostResponse => app.post_response_editor_state.lines.clone().into(),
            };
            if let Some(col) = app.collections.get_mut(app.active_collection_index) {
                if let Some(req_id) = &app.current_request_id {
                    if let Some(req) = col.find_request_mut(req_id) {
                        match app.scripts_subtab {
                            ScriptsSubTab::PreRequest => req.pre_request_script = Some(new_content),
                            ScriptsSubTab::PostResponse => req.post_response_script = Some(new_content),
                        }
                    }
                }
            }
        } else {
            let new_content: String = app.body_editor_state.lines.clone().into();
            if let Some(col) = app.collections.get_mut(app.active_collection_index) {
                if let Some(req_id) = &app.current_request_id {
                    if let Some(req) = col.find_request_mut(req_id) {
                        req.body.raw.content = new_content;
                    }
                }
            }
        }
        app.input_mode = InputMode::Normal;
        return;
    }

    // Pass event to edtui
    if is_scripts {
        match app.scripts_subtab {
            ScriptsSubTab::PreRequest => {
                app.body_editor_handler.on_key_event(key, &mut app.pre_request_editor_state);
            }
            ScriptsSubTab::PostResponse => {
                app.body_editor_handler.on_key_event(key, &mut app.post_response_editor_state);
            }
        }
    } else {
        app.body_editor_handler.on_key_event(key, &mut app.body_editor_state);
    }
}
