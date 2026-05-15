use crate::tui::app::{App, FocusedPanel, PropertyEditorField};
use crate::tui::ui::utils::{
    create_block, get_method_enum_color, highlight_env_vars, title_with_key,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, List, ListItem, Row, Table},
};

pub fn render_left_column(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Collections
            Constraint::Percentage(50), // APIs / Environments
        ])
        .split(area);

    let visible_collections = app.get_visible_collections();
    let collections_items: Vec<ListItem> = visible_collections
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let indent = "  ".repeat(item.item_type_depth());
            let is_selected = app.focused_panel == FocusedPanel::Collections
                && i == app.selected_collection_index;

            match &item.item_type {
                crate::tui::app::VisibleItemType::Collection { expanded } => {
                    let icon = if *expanded { "▼" } else { "▶" };
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::REVERSED | Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    ListItem::new(format!("{}{} {} {}", indent, icon, "📦", item.name)).style(style)
                }
                crate::tui::app::VisibleItemType::Folder { expanded } => {
                    let icon = if *expanded { "▼" } else { "▶" };
                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!("{}{} {} {}", indent, icon, "📁", item.name)).style(style)
                }
                crate::tui::app::VisibleItemType::Request { method, .. } => {
                    let color = get_method_enum_color(*method);
                    let style = if is_selected {
                        Style::default()
                            .fg(color)
                            .add_modifier(Modifier::REVERSED | Modifier::BOLD)
                    } else {
                        Style::default().fg(color)
                    };
                    ListItem::new(format!("{}{} {:?}  {}", indent, "  ", method, item.name))
                        .style(style)
                }
            }
        })
        .collect();

    let collections_list = List::new(collections_items).block(create_block(
        title_with_key("C", "Collections"),
        app.focused_panel == FocusedPanel::Collections,
    ));
    app.collections_state
        .select(Some(app.selected_collection_index));
    f.render_stateful_widget(collections_list, chunks[0], &mut app.collections_state);

    render_left_bottom_panel(f, app, chunks[1]);
}

pub fn render_left_bottom_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let is_apis = app.left_bottom_tab == crate::tui::app::LeftBottomTab::Apis;
    let is_envs = app.left_bottom_tab == crate::tui::app::LeftBottomTab::Environments;

    let api_style = if is_apis {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let env_style = if is_envs {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let key_style = Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::BOLD);

    let block_title = Line::from(vec![
        Span::raw(" "),
        Span::styled("A", key_style),
        Span::styled(" APIs ", api_style),
        Span::raw("| "),
        Span::styled("V", key_style),
        Span::styled(" Variables ", env_style),
    ]);

    let block = create_block(
        block_title,
        app.focused_panel == FocusedPanel::Apis || app.focused_panel == FocusedPanel::Environments,
    );

    match app.left_bottom_tab {
        crate::tui::app::LeftBottomTab::Apis => {
            let visible_items = app.get_visible_items();
            let api_items: Vec<ListItem> = visible_items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let indent = "  ".repeat(item.item_type_depth());
                    let is_selected =
                        app.focused_panel == FocusedPanel::Apis && i == app.selected_api_index;

                    match &item.item_type {
                        crate::tui::app::VisibleItemType::Collection { .. } => {
                            ListItem::new(format!("{}📦 {}", indent, item.name))
                        }
                        crate::tui::app::VisibleItemType::Folder { expanded } => {
                            let icon = if *expanded { "▼" } else { "▶" };
                            let style = if is_selected {
                                Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
                            } else {
                                Style::default()
                            };
                            ListItem::new(format!("{}{} {} {}", indent, icon, "📁", item.name))
                                .style(style)
                        }
                        crate::tui::app::VisibleItemType::Request { method, .. } => {
                            let color = get_method_enum_color(*method);
                            let style = if is_selected {
                                Style::default()
                                    .fg(color)
                                    .add_modifier(Modifier::REVERSED | Modifier::BOLD)
                            } else {
                                Style::default().fg(color)
                            };
                            ListItem::new(format!("{}{} {:?}  {}", indent, "  ", method, item.name))
                                .style(style)
                        }
                    }
                })
                .collect();

            let apis_list = List::new(api_items).block(block);
            app.apis_state.select(Some(app.selected_api_index));
            f.render_stateful_widget(apis_list, area, &mut app.apis_state);
        }
        crate::tui::app::LeftBottomTab::Environments => {
            let env_vars = app.get_active_collection_env_vars();
            let header = Row::new(vec![
                Cell::from("Key").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Value").style(Style::default().add_modifier(Modifier::BOLD)),
            ])
            .height(1)
            .bottom_margin(0);

            let rows: Vec<Row> = env_vars
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let is_row_selected = app.focused_panel == FocusedPanel::Environments
                        && i == app.selected_env_index;

                    let is_editing_this_value = app.input_mode
                        == crate::tui::app::InputMode::Editing
                        && app.focused_panel == FocusedPanel::Environments
                        && i == app.selected_env_index
                        && app.property_editor_field == crate::tui::app::PropertyEditorField::Value;

                    let display_value = if app.mask_env_values && !is_editing_this_value {
                        "*".repeat(item.value.len())
                    } else {
                        item.value.clone()
                    };

                    let mut cells = vec![
                        Cell::from(highlight_env_vars(item.key.as_str())),
                        Cell::from(highlight_env_vars(&display_value)),
                    ];

                    if is_row_selected {
                        let field_idx = match app.property_editor_field {
                            PropertyEditorField::Key => 0,
                            PropertyEditorField::Value => 1,
                            _ => 0,
                        };
                        cells[field_idx] = cells[field_idx]
                            .clone()
                            .style(Style::default().add_modifier(Modifier::REVERSED));
                    }
                    Row::new(cells)
                })
                .collect();

            let table = Table::new(
                rows,
                [Constraint::Percentage(40), Constraint::Percentage(60)],
            )
            .header(header)
            .block(block);

            app.environments_table_state
                .select(Some(app.selected_env_index));
            f.render_stateful_widget(table, area, &mut app.environments_table_state);
        }
    }
}
