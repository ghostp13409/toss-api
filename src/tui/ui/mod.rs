use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::{
    App, FocusedPanel, InputMode, PropertyEditorField, PropertyTab, RequestBarPart,
};
use crate::tui::ui::utils::{centered_rect, title_with_key};

pub mod syntax;
pub mod utils;
pub mod widgets;

use widgets::*;

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Min(0),    // Main Content
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    // 1. Title
    let title_text = format!(" Toss {} ", env!("CARGO_PKG_VERSION"));
    let title = Paragraph::new(title_text)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(title, chunks[0]);

    // 2. Main Columns (30/70)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left Column
            Constraint::Percentage(70), // Right Column
        ])
        .split(chunks[1]);

    render_left_column(f, app, columns[0]);
    render_right_column(f, app, columns[1]);

    // 3. Footer
    render_footer(f, app, chunks[2]);

    // 4. Overlays
    if app.show_method_search {
        render_method_search(f, app);
    }
    if app.input_mode == InputMode::Rename {
        render_rename_popup(f, app);
    }
    if app.input_mode == InputMode::CreateItem {
        render_create_popup(f, app);
    }
    if app.input_mode == InputMode::ConfirmDelete {
        render_delete_confirmation(f, app);
    }
    if app.input_mode == InputMode::ConfirmQuit {
        render_quit_confirmation(f, app);
    }
    if app.input_mode == InputMode::Help {
        render_help_popup(f, app);
    }
    if app.show_search {
        render_search_popup(f, app, columns[0]);
    }
    if let Some(error) = &app.error_message {
        render_error_popup(f, error);
    }
    if app.show_autocomplete {
        render_autocomplete(f, app);
    }

    // 5. Cursor positioning
    render_cursor(f, app, &chunks, &columns);
}

fn render_cursor(f: &mut Frame, app: &mut App, chunks: &[Rect], columns: &[Rect]) {
    match app.input_mode {
        InputMode::Editing => {
            if app.focused_panel == FocusedPanel::RequestBar
                && app.active_request_part == RequestBarPart::Url
            {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(title_with_key("R", "Request"));
                let area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),      // Request Bar
                        Constraint::Length(3),      // Properties Tabs
                        Constraint::Percentage(40), // Details
                        Constraint::Min(0),         // Response area
                    ])
                    .split(columns[1])[0];

                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(10), // Method
                        Constraint::Min(0),     // URL
                        Constraint::Length(10), // Send Button
                    ])
                    .split(block.inner(area));

                let cursor_x = app.url[..app.cursor_position.min(app.url.len())]
                    .chars()
                    .count() as u16;
                let cursor_pos = (layout[1].x + cursor_x, layout[1].y);
                app.last_cursor_pos = cursor_pos;
                f.set_cursor_position(cursor_pos);
            } else if app.focused_panel == FocusedPanel::Details {
                // Find cursor position in KV editor
                if matches!(
                    app.selected_property_tab,
                    PropertyTab::Params | PropertyTab::Headers | PropertyTab::Auth
                ) {
                    let area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3),      // Request Bar
                            Constraint::Length(3),      // Properties Tabs
                            Constraint::Percentage(40), // Details
                            Constraint::Min(0),         // Response area
                        ])
                        .split(columns[1])[2];

                    let inner_area = Block::default().borders(Borders::ALL).inner(area);
                    // Header takes 1 line
                    let x = inner_area.x;
                    let y = inner_area.y + 1 + app.property_editor_row as u16;

                    let show_description = matches!(
                        app.selected_property_tab,
                        PropertyTab::Params | PropertyTab::Headers
                    );

                    let offset = if show_description {
                        match app.property_editor_field {
                            PropertyEditorField::Key => (5 * inner_area.width / 100) + 1,
                            PropertyEditorField::Value => (35 * inner_area.width / 100) + 2,
                            PropertyEditorField::Description => (65 * inner_area.width / 100) + 3,
                        }
                    } else {
                        match app.property_editor_field {
                            PropertyEditorField::Key => (5 * inner_area.width / 100) + 1,
                            PropertyEditorField::Value => (50 * inner_area.width / 100) + 2,
                            PropertyEditorField::Description => (50 * inner_area.width / 100) + 2,
                        }
                    };

                    let current_val = app.get_kv_editor_value();
                    let cursor_x = current_val[..app.cursor_position.min(current_val.len())]
                        .chars()
                        .count() as u16;
                    let cursor_pos = (x + offset + cursor_x, y);
                    app.last_cursor_pos = cursor_pos;
                    f.set_cursor_position(cursor_pos);
                }
            } else if app.focused_panel == FocusedPanel::Environments {
                let area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(50), // Collections
                        Constraint::Percentage(50), // APIs / Environments
                    ])
                    .split(columns[0])[1];
                let inner_area = Block::default().borders(Borders::ALL).inner(area);
                let x = inner_area.x;
                let y = inner_area.y + 1 + app.selected_env_index as u16;

                let offset = match app.property_editor_field {
                    PropertyEditorField::Key => 0,
                    PropertyEditorField::Value => (40 * inner_area.width / 100) + 1,
                    _ => 0,
                };
                let current_val = app.get_env_editor_value();
                let cursor_x = current_val[..app.cursor_position.min(current_val.len())]
                    .chars()
                    .count() as u16;
                let cursor_pos = (x + offset + cursor_x, y);
                app.last_cursor_pos = cursor_pos;
                f.set_cursor_position(cursor_pos);
            }
        }
        InputMode::Rename | InputMode::CreateItem => {
            let area = centered_rect(40, 10, f.area());
            let cursor_x = app.rename_input[..app.cursor_position.min(app.rename_input.len())]
                .chars()
                .count() as u16;
            f.set_cursor_position((area.x + 1 + cursor_x, area.y + 1));
        }
        InputMode::Search if app.show_search => {
            let sidebar_area = columns[0];
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(sidebar_area);
            let area = chunks[1];
            let cursor_x = app.search_query[..app.cursor_position.min(app.search_query.len())]
                .chars()
                .count() as u16;
            f.set_cursor_position((area.x + 1 + cursor_x, area.y + 1));
        }
        InputMode::Command => {
            let cursor_x = app.command_input[..app.cursor_position.min(app.command_input.len())]
                .chars()
                .count() as u16;
            f.set_cursor_position((
                chunks[2].x + 1 + cursor_x + 1, // +1 for ':'
                chunks[2].y,
            ));
        }
        _ => {
            if app.show_method_search {
                let area = centered_rect(20, 30, f.area());
                let cursor_x = app.method_search_query
                    [..app.cursor_position.min(app.method_search_query.len())]
                    .chars()
                    .count() as u16;
                f.set_cursor_position((area.x + 1 + cursor_x, area.y + 1));
            }
        }
    }
}
