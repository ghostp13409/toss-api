use crate::tui::app::{App, FocusedPanel, LeftBottomTab, PropertyTab, VisibleItemType};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

pub fn handle_mouse_event(app: &mut App, event: MouseEvent) {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left) => {
            let x = event.column;
            let y = event.row;

            // Check each panel's rect
            if let Some(rect) = app.layout_rects.collections {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    app.focused_panel = FocusedPanel::Collections;
                    let visible_collections = app.get_visible_collections();
                    if y > rect.y && y < rect.y + rect.height - 1 {
                        let clicked_row = (y - rect.y - 1) as usize;
                        if let Some(item) = visible_collections.get(clicked_row) {
                            app.selected_collection_index = clicked_row;
                            match &item.item_type {
                                VisibleItemType::Collection { .. } | VisibleItemType::Folder { .. } => {
                                    app.toggle_folder();
                                }
                                VisibleItemType::Request { id, method, .. } => {
                                    let id_clone = id.clone();
                                    let method_clone = *method;
                                    app.select_request(id_clone, method_clone);
                                }
                            }
                        }
                    }
                    return;
                }
            }

            if let Some(rect) = app.layout_rects.apis {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    // Check if clicking on tabs (APIs | Variables)
                    if y == rect.y {
                        if x > rect.x + 1 && x < rect.x + 8 { // " A APIs "
                            app.left_bottom_tab = LeftBottomTab::Apis;
                            app.focused_panel = FocusedPanel::Apis;
                        } else if x > rect.x + 10 && x < rect.x + 22 { // " V Variables "
                            app.left_bottom_tab = LeftBottomTab::Environments;
                            app.focused_panel = FocusedPanel::Environments;
                        }
                    } else {
                        match app.left_bottom_tab {
                            LeftBottomTab::Apis => {
                                app.focused_panel = FocusedPanel::Apis;
                                let visible_items = app.get_visible_items();
                                if y > rect.y && y < rect.y + rect.height - 1 {
                                    let clicked_row = (y - rect.y - 1) as usize;
                                    if let Some(item) = visible_items.get(clicked_row) {
                                        app.selected_api_index = clicked_row;
                                        match &item.item_type {
                                            VisibleItemType::Folder { .. } => {
                                                app.toggle_folder();
                                            }
                                            VisibleItemType::Request { id, method, .. } => {
                                                let id_clone = id.clone();
                                                let method_clone = *method;
                                                app.select_request(id_clone, method_clone);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            LeftBottomTab::Environments => {
                                app.focused_panel = FocusedPanel::Environments;
                                let env_vars = app.get_active_collection_env_vars();
                                if y > rect.y + 1 && y < rect.y + rect.height - 1 {
                                    let clicked_row = (y - rect.y - 2) as usize;
                                    if clicked_row < env_vars.len() {
                                        app.selected_env_index = clicked_row;
                                    }
                                }
                            }
                        }
                    }
                    return;
                }
            }

            if let Some(rect) = app.layout_rects.request_bar {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    app.focused_panel = FocusedPanel::RequestBar;
                    
                    let rel_x = x.saturating_sub(rect.x + 1);
                    if rel_x < 10 {
                        app.active_request_part = crate::tui::app::RequestBarPart::Method;
                        app.show_method_search = true;
                    } else if x >= rect.x + rect.width - 11 {
                        app.active_request_part = crate::tui::app::RequestBarPart::Send;
                        app.pending_actions.push(crate::tui::app::TuiAction::SendRequest);
                    } else {
                        app.active_request_part = crate::tui::app::RequestBarPart::Url;
                        app.input_mode = crate::tui::app::InputMode::Editing;
                        app.cursor_position = app.url.len();
                    }
                    return;
                }
            }

            if let Some(rect) = app.layout_rects.properties_tabs {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    app.focused_panel = FocusedPanel::Properties;
                    let inner_width = rect.width.saturating_sub(2);
                    let tab_width = inner_width / 5;
                    if tab_width > 0 {
                        let rel_x = x.saturating_sub(rect.x + 1);
                        let clicked_tab = rel_x / tab_width;
                        app.selected_property_tab = match clicked_tab {
                            0 => PropertyTab::Params,
                            1 => PropertyTab::Headers,
                            2 => PropertyTab::Auth,
                            3 => PropertyTab::Body,
                            _ => PropertyTab::Scripts,
                        };
                    }
                    return;
                }
            }

            if let Some(rect) = app.layout_rects.details {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    app.focused_panel = FocusedPanel::Details;
                    // Click in KV editor logic
                    if matches!(app.selected_property_tab, PropertyTab::Params | PropertyTab::Headers | PropertyTab::Body) {
                        if y > rect.y + 1 && y < rect.y + rect.height - 1 {
                            let clicked_row = (y - rect.y - 2) as usize;
                            let items_len = match app.selected_property_tab {
                                PropertyTab::Params => app.get_current_request().map(|r| r.params.len()).unwrap_or(0),
                                PropertyTab::Headers => app.get_current_request().map(|r| r.headers.len()).unwrap_or(0),
                                PropertyTab::Body => {
                                    match app.get_current_request().map(|r| r.body.clone()) {
                                        Some(crate::core::collection::RequestBody::FormData { items }) => items.len(),
                                        Some(crate::core::collection::RequestBody::XWwwFormUrlEncoded { items }) => items.len(),
                                        _ => 0,
                                    }
                                }
                                _ => 0,
                            };
                            if clicked_row < items_len {
                                app.property_editor_row = clicked_row;
                                
                                // Field selection based on X
                                let inner_width = rect.width - 2;
                                let show_description = matches!(app.selected_property_tab, PropertyTab::Params | PropertyTab::Headers);
                                
                                if show_description {
                                    let rel_x = x - rect.x - 1;
                                    if rel_x < 5 * inner_width / 100 {
                                        app.toggle_kv_param();
                                    } else if rel_x < 35 * inner_width / 100 {
                                        app.property_editor_field = crate::tui::app::PropertyEditorField::Key;
                                    } else if rel_x < 65 * inner_width / 100 {
                                        app.property_editor_field = crate::tui::app::PropertyEditorField::Value;
                                    } else {
                                        app.property_editor_field = crate::tui::app::PropertyEditorField::Description;
                                    }
                                } else {
                                     let rel_x = x - rect.x - 1;
                                     if rel_x < 5 * inner_width / 100 {
                                        app.toggle_kv_param();
                                    } else if rel_x < 50 * inner_width / 100 {
                                        app.property_editor_field = crate::tui::app::PropertyEditorField::Key;
                                    } else {
                                        app.property_editor_field = crate::tui::app::PropertyEditorField::Value;
                                    }
                                }
                            }
                        }
                    }
                    return;
                }
            }

            if let Some(rect) = app.layout_rects.response {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    app.focused_panel = FocusedPanel::Response;
                    return;
                }
            }

            if let Some(rect) = app.layout_rects.stats {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    app.focused_panel = FocusedPanel::Stats;
                    return;
                }
            }
        }
        MouseEventKind::ScrollUp => {
            let x = event.column;
            let y = event.row;

            if let Some(rect) = app.layout_rects.collections {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    app.selected_collection_index = app.selected_collection_index.saturating_sub(1);
                    return;
                }
            }
            if let Some(rect) = app.layout_rects.apis {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    match app.left_bottom_tab {
                        LeftBottomTab::Apis => app.selected_api_index = app.selected_api_index.saturating_sub(1),
                        LeftBottomTab::Environments => app.selected_env_index = app.selected_env_index.saturating_sub(1),
                    }
                    return;
                }
            }
            if let Some(rect) = app.layout_rects.details {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    if matches!(app.selected_property_tab, PropertyTab::Body) {
                        app.details_scroll = app.details_scroll.saturating_sub(1);
                    } else {
                        app.property_editor_row = app.property_editor_row.saturating_sub(1);
                    }
                    return;
                }
            }
            if let Some(rect) = app.layout_rects.response {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    app.response_scroll = app.response_scroll.saturating_sub(1);
                    return;
                }
            }
        }
        MouseEventKind::ScrollDown => {
            let x = event.column;
            let y = event.row;

            if let Some(rect) = app.layout_rects.collections {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    let visible_count = app.get_visible_collections().len();
                    if app.selected_collection_index + 1 < visible_count {
                        app.selected_collection_index += 1;
                    }
                    return;
                }
            }
            if let Some(rect) = app.layout_rects.apis {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    match app.left_bottom_tab {
                        LeftBottomTab::Apis => {
                            let visible_count = app.get_visible_items().len();
                            if app.selected_api_index + 1 < visible_count {
                                app.selected_api_index += 1;
                            }
                        }
                        LeftBottomTab::Environments => {
                            let env_vars = app.get_active_collection_env_vars();
                            if app.selected_env_index + 1 < env_vars.len() {
                                app.selected_env_index += 1;
                            }
                        }
                    }
                    return;
                }
            }
            if let Some(rect) = app.layout_rects.details {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    if matches!(app.selected_property_tab, PropertyTab::Body) {
                        app.details_scroll += 1;
                    } else {
                        let items_len = match app.selected_property_tab {
                            PropertyTab::Params => app.get_current_request().map(|r| r.params.len()).unwrap_or(0),
                            PropertyTab::Headers => app.get_current_request().map(|r| r.headers.len()).unwrap_or(0),
                            PropertyTab::Body => {
                                match app.get_current_request().map(|r| r.body.clone()) {
                                    Some(crate::core::collection::RequestBody::FormData { items }) => items.len(),
                                    Some(crate::core::collection::RequestBody::XWwwFormUrlEncoded { items }) => items.len(),
                                    _ => 0,
                                }
                            }
                            _ => 0,
                        };
                        if app.property_editor_row + 1 < items_len {
                            app.property_editor_row += 1;
                        }
                    }
                    return;
                }
            }
            if let Some(rect) = app.layout_rects.response {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                    app.response_scroll += 1;
                    return;
                }
            }
        }
        _ => {}
    }
}
