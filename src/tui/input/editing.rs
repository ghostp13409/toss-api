use crate::tui::app::{
    App, FocusedPanel, InputMode, PropertyEditorField, PropertyTab, RequestBarPart,
};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_editing_mode(app: &mut App, key: KeyEvent) {
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
            } else if app.focused_panel == FocusedPanel::Environments {
                match app.property_editor_field {
                    PropertyEditorField::Key => {
                        app.property_editor_field = PropertyEditorField::Value;
                        let current_val = app.get_env_editor_value();
                        app.cursor_position = current_val.len();
                    }
                    PropertyEditorField::Value => {
                        app.input_mode = InputMode::Normal;
                        app.property_editor_field = PropertyEditorField::Key;
                    }
                    _ => {
                        app.input_mode = InputMode::Normal;
                        app.property_editor_field = PropertyEditorField::Key;
                    }
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
            } else if app.focused_panel == FocusedPanel::Environments {
                app.property_editor_field = match app.property_editor_field {
                    PropertyEditorField::Key => PropertyEditorField::Value,
                    PropertyEditorField::Value => PropertyEditorField::Description,
                    PropertyEditorField::Description => PropertyEditorField::Key,
                };
                let current_val = app.get_env_editor_value();
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
                if app.url[..app.cursor_position].ends_with("{{") {
                    app.show_autocomplete = true;
                    app.autocomplete_query.clear();
                    app.autocomplete_index = 0;
                }
            } else if app.focused_panel == FocusedPanel::Details {
                let mut current_val = app.get_kv_editor_value();
                app.insert_char(&mut current_val, c);
                app.update_kv_param(current_val.clone());
                if current_val[..app.cursor_position].ends_with("{{") {
                    app.show_autocomplete = true;
                    app.autocomplete_query.clear();
                    app.autocomplete_index = 0;
                }
            } else if app.focused_panel == FocusedPanel::Environments {
                let mut current_val = app.get_env_editor_value();
                app.insert_char(&mut current_val, c);
                app.update_env_editor_value(current_val.clone());
                if current_val[..app.cursor_position].ends_with("{{") {
                    app.show_autocomplete = true;
                    app.autocomplete_query.clear();
                    app.autocomplete_index = 0;
                }
            }
        }
        KeyCode::Backspace => {
            if app.focused_panel == FocusedPanel::RequestBar
                && app.active_request_part == RequestBarPart::Url
            {
                app.delete_char_url();
                app.sync_params_from_url();
                if app.url[..app.cursor_position].ends_with("{{") {
                    app.show_autocomplete = true;
                    app.autocomplete_query.clear();
                    app.autocomplete_index = 0;
                }
            } else if app.focused_panel == FocusedPanel::Details {
                let mut current_val = app.get_kv_editor_value();
                app.delete_char(&mut current_val);
                app.update_kv_param(current_val.clone());
                if current_val[..app.cursor_position].ends_with("{{") {
                    app.show_autocomplete = true;
                    app.autocomplete_query.clear();
                    app.autocomplete_index = 0;
                }
            } else if app.focused_panel == FocusedPanel::Environments {
                let mut current_val = app.get_env_editor_value();
                app.delete_char(&mut current_val);
                app.update_env_editor_value(current_val.clone());
                if current_val[..app.cursor_position].ends_with("{{") {
                    app.show_autocomplete = true;
                    app.autocomplete_query.clear();
                    app.autocomplete_index = 0;
                }
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
            } else if app.focused_panel == FocusedPanel::Environments {
                let mut current_val = app.get_env_editor_value();
                app.delete_char_forward(&mut current_val);
                app.update_env_editor_value(current_val);
            }
        }
        KeyCode::Left => app.move_cursor_left(),
        KeyCode::Right => {
            let max = if app.focused_panel == FocusedPanel::Details {
                app.get_kv_editor_value().len()
            } else if app.focused_panel == FocusedPanel::Environments {
                app.get_env_editor_value().len()
            } else {
                app.url.len()
            };
            app.move_cursor_right(max);
        }
        KeyCode::Home => app.cursor_position = 0,
        KeyCode::End => {
            app.cursor_position = if app.focused_panel == FocusedPanel::Details {
                app.get_kv_editor_value().len()
            } else if app.focused_panel == FocusedPanel::Environments {
                app.get_env_editor_value().len()
            } else {
                app.url.len()
            };
        }
        _ => {}
    }
}
