use crate::core::collection::KVParam;
use crate::tui::app::{App, FocusedPanel, PropertyEditorField, PropertyTab, RequestBarPart};
use crate::tui::ui::syntax::{apply_env_vars, format_content, highlight_content};
use crate::tui::ui::utils::{
    create_block, get_method_enum_color, highlight_env_vars, title_with_key,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs, Wrap},
};

pub fn render_right_column(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Request Bar
            Constraint::Length(3),      // Properties Tabs
            Constraint::Percentage(40), // Details
            Constraint::Min(0),         // Response area
        ])
        .split(area);

    render_request_bar(f, app, chunks[0]);
    render_properties_tabs(f, app, chunks[1]);
    render_details_area(f, app, chunks[2]);

    let response_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(80), // Response Body
            Constraint::Percentage(20), // Stats
        ])
        .split(chunks[3]);

    let response_block = create_block(
        title_with_key("E", "Response"),
        app.focused_panel == FocusedPanel::Response,
    );

    let formatted_body = format_content(&app.response_body, app.response_content_type.as_deref());
    let trimmed_body = formatted_body.trim_end();
    let highlighted_body = highlight_content(trimmed_body, app.response_content_type.as_deref());

    let response_area_inner = response_block.inner(response_area[0]);
    let response_height = response_area_inner.height;
    let line_count = highlighted_body.lines.len() as u16;

    if line_count <= response_height {
        app.response_scroll = 0;
    } else {
        let max_scroll = line_count.saturating_sub(response_height);
        if app.response_scroll > max_scroll {
            app.response_scroll = max_scroll;
        }
    }

    let response_content = Paragraph::new(highlighted_body)
        .block(response_block)
        .scroll((app.response_scroll, app.response_horizontal_scroll))
        .wrap(Wrap { trim: false });
    f.render_widget(response_content, response_area[0]);

    let stat_block = create_block(
        title_with_key("T", "Stat"),
        app.focused_panel == FocusedPanel::Stats,
    );

    let mut stat_lines = Vec::new();

    if let Some(status) = &app.response_status {
        let style = if status.contains("200") || status.starts_with('2') {
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else if status.starts_with('3') {
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else if status.starts_with('4') {
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else if status.starts_with('5') || status == "ERROR" {
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };
        stat_lines.push(Line::from(vec![
            Span::raw(" Status: "),
            Span::styled(format!(" {} ", status), style),
        ]));
        stat_lines.push(Line::raw(""));
    }

    if app.response_stats.is_empty() {
        if app.response_status.is_none() {
            stat_lines.push(Line::raw("No Data"));
        }
    } else {
        for line in app.response_stats.lines() {
            stat_lines.push(Line::raw(line.to_string()));
        }
    }

    let stat_area_inner = stat_block.inner(response_area[1]);
    let stat_height = stat_area_inner.height;
    let stat_line_count = stat_lines.len() as u16;

    if stat_line_count <= stat_height {
        app.stats_scroll = 0;
    } else {
        let max_scroll = stat_line_count.saturating_sub(stat_height);
        if app.stats_scroll > max_scroll {
            app.stats_scroll = max_scroll;
        }
    }

    let stat_content = Paragraph::new(stat_lines)
        .block(stat_block)
        .scroll((app.stats_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(stat_content, response_area[1]);
}

pub fn render_request_bar(f: &mut Frame, app: &App, area: Rect) {
    let is_bar_focused = app.focused_panel == FocusedPanel::RequestBar;

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title_with_key("R", "Request"))
        .border_style(if is_bar_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        });

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // Method
            Constraint::Min(0),     // URL
            Constraint::Length(10), // Send Button
        ])
        .split(block.inner(area));

    // Method Badge
    let method_color = get_method_enum_color(app.method);
    let method_style = if is_bar_focused && app.active_request_part == RequestBarPart::Method {
        Style::default()
            .fg(method_color)
            .add_modifier(Modifier::REVERSED | Modifier::BOLD)
    } else {
        Style::default()
            .fg(method_color)
            .add_modifier(Modifier::BOLD)
    };
    let method_text = Paragraph::new(format!(" {:?} ", app.method)).style(method_style);

    // URL
    let url_style = if is_bar_focused && app.active_request_part == RequestBarPart::Url {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };
    let url_text = Paragraph::new(highlight_env_vars(app.url.as_str())).style(url_style);

    // Send Button
    let send_style = if is_bar_focused && app.active_request_part == RequestBarPart::Send {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::REVERSED | Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };
    let send_button = Paragraph::new(" [ Send ] ").style(send_style);

    f.render_widget(block, area);
    f.render_widget(method_text, layout[0]);
    f.render_widget(url_text, layout[1]);
    f.render_widget(send_button, layout[2]);
}

pub fn render_properties_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        Line::from(vec![
            Span::styled(
                "[P]",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Params "),
        ]),
        Line::from(vec![
            Span::styled(
                "[H]",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Headers "),
        ]),
        Line::from(vec![
            Span::styled(
                "[U]",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Auth "),
        ]),
        Line::from(vec![
            Span::styled(
                "[B]",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Body "),
        ]),
        Line::from(vec![
            Span::styled(
                "[S]",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Scripts "),
        ]),
    ];
    let selected_idx = match app.selected_property_tab {
        PropertyTab::Params => 0,
        PropertyTab::Headers => 1,
        PropertyTab::Auth => 2,
        PropertyTab::Body => 3,
        PropertyTab::Scripts => 4,
    };

    let tabs = Tabs::new(titles)
        .block(create_block(
            " Properties ".to_string(),
            app.focused_panel == FocusedPanel::Properties,
        ))
        .select(selected_idx)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    f.render_widget(tabs, area);
}

pub fn render_details_area(f: &mut Frame, app: &mut App, area: Rect) {
    match app.selected_property_tab {
        PropertyTab::Params => {
            let params = app
                .get_current_request()
                .map(|r| r.params.clone())
                .unwrap_or_default();
            render_kv_editor(
                f,
                app,
                area,
                title_with_key("P", "Query Parameters"),
                &params,
                true,
            );
        }
        PropertyTab::Headers => {
            let headers = app
                .get_current_request()
                .map(|r| r.headers.clone())
                .unwrap_or_default();
            render_kv_editor(f, app, area, title_with_key("H", "Headers"), &headers, true);
        }
        PropertyTab::Auth => {
            let auth = app
                .get_current_request()
                .map(|r| r.auth.clone())
                .unwrap_or(crate::core::collection::Auth::None);
            let title = match auth {
                crate::core::collection::Auth::None => " Auth: None (Press 't' to change) ",
                crate::core::collection::Auth::Bearer { .. } => {
                    " Auth: Bearer (Press 't' to change) "
                }
                crate::core::collection::Auth::Basic { .. } => {
                    " Auth: Basic (Press 't' to change) "
                }
                crate::core::collection::Auth::ApiKey { .. } => {
                    " Auth: ApiKey (Press 't' to change) "
                }
            };
            let block = create_block(
                title_with_key("U", title),
                app.focused_panel == FocusedPanel::Details,
            );

            let mut kv_params = Vec::new();
            match auth {
                crate::core::collection::Auth::None => {
                    f.render_widget(Paragraph::new(" No authentication ").block(block), area);
                }
                crate::core::collection::Auth::Bearer { token } => {
                    kv_params.push(crate::core::collection::KVParam {
                        key: "Token".to_string(),
                        value: token,
                        enabled: true,
                        description: None,
                    });
                    render_kv_editor(f, app, area, title_with_key("U", title), &kv_params, false);
                }
                crate::core::collection::Auth::Basic { username, password } => {
                    kv_params.push(crate::core::collection::KVParam {
                        key: "Username".to_string(),
                        value: username,
                        enabled: true,
                        description: None,
                    });
                    kv_params.push(crate::core::collection::KVParam {
                        key: "Password".to_string(),
                        value: password,
                        enabled: true,
                        description: None,
                    });
                    render_kv_editor(f, app, area, title_with_key("U", title), &kv_params, false);
                }
                crate::core::collection::Auth::ApiKey {
                    key,
                    value,
                    in_header,
                } => {
                    kv_params.push(crate::core::collection::KVParam {
                        key: "Key".to_string(),
                        value: key,
                        enabled: true,
                        description: None,
                    });
                    kv_params.push(crate::core::collection::KVParam {
                        key: "Value".to_string(),
                        value: value,
                        enabled: true,
                        description: None,
                    });
                    kv_params.push(crate::core::collection::KVParam {
                        key: "In Header".to_string(),
                        value: in_header.to_string(),
                        enabled: true,
                        description: None,
                    });
                    render_kv_editor(f, app, area, title_with_key("U", title), &kv_params, false);
                }
            }
        }
        PropertyTab::Body => {
            let body = app
                .get_current_request()
                .map(|r| r.body.clone())
                .unwrap_or(crate::core::collection::RequestBody::None);
            let title = match body {
                crate::core::collection::RequestBody::None => " Body: None (Press 't' to change) ",
                crate::core::collection::RequestBody::Raw { .. } => {
                    " Body: Raw (Press 't' to change) "
                }
                crate::core::collection::RequestBody::FormData { .. } => {
                    " Body: Form Data (Press 't' to change) "
                }
                crate::core::collection::RequestBody::XWwwFormUrlEncoded { .. } => {
                    " Body: x-www-form-urlencoded (Press 't' to change) "
                }
            };

            match body {
                crate::core::collection::RequestBody::None => {
                    let block = create_block(
                        title_with_key("B", title),
                        app.focused_panel == FocusedPanel::Details,
                    );
                    f.render_widget(Paragraph::new(" No body ").block(block), area);
                }
                crate::core::collection::RequestBody::Raw {
                    content,
                    content_type,
                } => {
                    let block = create_block(
                        title_with_key("B", title),
                        app.focused_panel == FocusedPanel::Details,
                    );

                    let formatted_body = format_content(&content, Some(content_type.as_str()));
                    let mut highlighted_body =
                        highlight_content(&formatted_body, Some(content_type.as_str()));
                    apply_env_vars(&mut highlighted_body);

                    let p = Paragraph::new(highlighted_body)
                        .block(block)
                        .scroll((app.details_scroll as u16, 0))
                        .wrap(Wrap { trim: false });
                    f.render_widget(p, area);
                }
                crate::core::collection::RequestBody::FormData { items } => {
                    render_kv_editor(f, app, area, title_with_key("B", title), &items, false);
                }
                crate::core::collection::RequestBody::XWwwFormUrlEncoded { items } => {
                    render_kv_editor(f, app, area, title_with_key("B", title), &items, false);
                }
            }
        }
        PropertyTab::Scripts => {
            let block = create_block(
                title_with_key("S", "Scripts"),
                app.focused_panel == FocusedPanel::Details,
            );
            f.render_widget(
                Paragraph::new(" Scripts editor coming soon "),
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 1,
                }),
            );
            f.render_widget(block, area);
        }
    }
}

pub fn render_kv_editor<'a, T: Into<ratatui::text::Line<'a>>>(
    f: &mut Frame,
    app: &mut App,
    area: Rect,
    title: T,
    items: &[KVParam],
    show_description: bool,
) {
    let block = create_block(title, app.focused_panel == FocusedPanel::Details);

    let mut header_cells = vec![
        Cell::from("").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Key").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Value").style(Style::default().add_modifier(Modifier::BOLD)),
    ];
    if show_description {
        header_cells
            .push(Cell::from("Description").style(Style::default().add_modifier(Modifier::BOLD)));
    }
    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_row_focused =
                app.focused_panel == FocusedPanel::Details && app.property_editor_row == i;

            let check = if item.enabled { "[x]" } else { "[ ]" };

            let mut cells = vec![
                Cell::from(check),
                Cell::from(highlight_env_vars(item.key.as_str())),
                Cell::from(highlight_env_vars(item.value.as_str())),
            ];
            if show_description {
                cells.push(Cell::from(highlight_env_vars(
                    item.description.as_deref().unwrap_or(""),
                )));
            }

            if is_row_focused {
                let field_idx = match app.property_editor_field {
                    PropertyEditorField::Key => 1,
                    PropertyEditorField::Value => 2,
                    PropertyEditorField::Description => {
                        if show_description {
                            3
                        } else {
                            2
                        }
                    }
                };
                cells[field_idx] = cells[field_idx]
                    .clone()
                    .style(Style::default().add_modifier(Modifier::REVERSED));
            }

            Row::new(cells)
        })
        .collect();

    let constraints = if show_description {
        vec![
            Constraint::Percentage(5),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
        ]
    } else {
        vec![
            Constraint::Percentage(5),
            Constraint::Percentage(45),
            Constraint::Percentage(50),
        ]
    };

    let table = Table::new(rows, constraints)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    app.details_table_state
        .select(Some(app.property_editor_row));
    f.render_stateful_widget(table, area, &mut app.details_table_state);
}
