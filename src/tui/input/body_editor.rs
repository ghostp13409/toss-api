use crate::tui::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_body_editor_input(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc && app.body_editor_state.mode == edtui::EditorMode::Normal {
        // Save and exit
        let new_content: String = app.body_editor_state.lines.clone().into();
        if let Some(col) = app.collections.get_mut(app.active_collection_index) {
            if let Some(req_id) = &app.current_request_id {
                if let Some(req) = col.find_request_mut(req_id) {
                    req.body.raw.content = new_content;
                }
            }
        }
        app.input_mode = InputMode::Normal;
        return;
    }

    // Pass event to edtui
    app.body_editor_handler
        .on_key_event(key, &mut app.body_editor_state);
}
