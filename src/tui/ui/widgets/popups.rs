use crate::tui::app::{App, FocusedPanel, PendingItemType};
use crate::tui::ui::utils::{centered_rect, get_method_color};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

pub fn render_error_popup(f: &mut Frame, error: &str) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let p = Paragraph::new(error)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(p, area);
}

pub fn render_create_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 10, f.area());
    f.render_widget(Clear, area);

    let title = match app.pending_item_type {
        Some(PendingItemType::Collection) => " Create Collection ",
        Some(PendingItemType::Folder) => " Create Folder ",
        Some(PendingItemType::Request) => " Create Request ",
        None => " Create Item ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let p = Paragraph::new(app.rename_input.as_str()).block(block);
    f.render_widget(p, area);
}

pub fn render_quit_confirmation(f: &mut Frame, _app: &App) {
    let area = centered_rect(30, 10, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Confirm Quit ")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let p = Paragraph::new("Quit application? (y/n)")
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(p, area);
}

pub fn render_delete_confirmation(f: &mut Frame, app: &App) {
    let area = centered_rect(30, 10, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Confirm Delete ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let text = if app.focused_panel == FocusedPanel::Collections {
        "Delete entire collection? (y/n)"
    } else {
        "Delete selected item? (y/n)"
    };

    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(p, area);
}

pub fn render_search_popup(f: &mut Frame, app: &App, sidebar_area: Rect) {
    // Position search popup at the bottom of the sidebar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(sidebar_area);

    let area = chunks[1];
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Filter (/) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let p = Paragraph::new(app.search_query.as_str()).block(block);
    f.render_widget(p, area);
}

pub fn render_rename_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(40, 10, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Rename ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let p = Paragraph::new(app.rename_input.as_str()).block(block);
    f.render_widget(p, area);
}

pub fn render_help_popup(f: &mut Frame, _app: &App) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help / Shortcuts ")
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        );

    let help_text = vec![
        Line::from(vec![Span::styled(
            " Global Shortcuts ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]),
        Line::from(vec![
            Span::styled("  q             ", Style::default().fg(Color::Cyan)),
            Span::raw(": Quit application"),
        ]),
        Line::from(vec![
            Span::styled("  ?             ", Style::default().fg(Color::Cyan)),
            Span::raw(": Toggle this help menu"),
        ]),
        Line::from(vec![
            Span::styled("  Tab           ", Style::default().fg(Color::Cyan)),
            Span::raw(": Next panel"),
        ]),
        Line::from(vec![
            Span::styled("  S-Tab         ", Style::default().fg(Color::Cyan)),
            Span::raw(": Previous panel"),
        ]),
        Line::from(vec![
            Span::styled("  :             ", Style::default().fg(Color::Cyan)),
            Span::raw(": Enter command mode"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl + s/Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": Send request"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            " Navigation ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]),
        Line::from(vec![
            Span::styled("  j / k     ", Style::default().fg(Color::Cyan)),
            Span::raw(": Move down / up"),
        ]),
        Line::from(vec![
            Span::styled("  h / l     ", Style::default().fg(Color::Cyan)),
            Span::raw(": Move left / right (or back / drill down)"),
        ]),
        Line::from(vec![
            Span::styled("  Enter     ", Style::default().fg(Color::Cyan)),
            Span::raw(": Select / Drill down / Edit"),
        ]),
        Line::from(vec![
            Span::styled("  Esc       ", Style::default().fg(Color::Cyan)),
            Span::raw(": Back / Cancel"),
        ]),
        Line::from(vec![
            Span::styled("  gg / G    ", Style::default().fg(Color::Cyan)),
            Span::raw(": Top / Bottom"),
        ]),
        Line::from(vec![
            Span::styled("  C-u / C-d ", Style::default().fg(Color::Cyan)),
            Span::raw(": Page up / down"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            " Panel Direct Access ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]),
        Line::from(vec![
            Span::styled("  C / R / E ", Style::default().fg(Color::Cyan)),
            Span::raw(": Collections / Request Bar / Response"),
        ]),
        Line::from(vec![
            Span::styled("  P / H / U ", Style::default().fg(Color::Cyan)),
            Span::raw(": Params / Headers / Auth"),
        ]),
        Line::from(vec![
            Span::styled("  B / S / T ", Style::default().fg(Color::Cyan)),
            Span::raw(": Body / Scripts / Stats"),
        ]),
        Line::from(vec![
            Span::styled("  V / A     ", Style::default().fg(Color::Cyan)),
            Span::raw(": Variables (Envs) / APIs list"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            " Actions ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]),
        Line::from(vec![
            Span::styled("  a / f / n ", Style::default().fg(Color::Cyan)),
            Span::raw(": Add Request / Folder / Collection"),
        ]),
        Line::from(vec![
            Span::styled("  d / r     ", Style::default().fg(Color::Cyan)),
            Span::raw(": Delete / Rename"),
        ]),
        Line::from(vec![
            Span::styled("  Space     ", Style::default().fg(Color::Cyan)),
            Span::raw(": Toggle folder or KV enabled state"),
        ]),
        Line::from(vec![
            Span::styled("  /         ", Style::default().fg(Color::Cyan)),
            Span::raw(": Filter (Collections / APIs)"),
        ]),
        Line::from(vec![
            Span::styled("  e         ", Style::default().fg(Color::Cyan)),
            Span::raw(": Focus Request Bar (URL)"),
        ]),
        Line::from(vec![
            Span::styled("  t         ", Style::default().fg(Color::Cyan)),
            Span::raw(": Cycle Auth / Body type (in Details)"),
        ]),
        Line::from(vec![
            Span::styled("  y / p     ", Style::default().fg(Color::Cyan)),
            Span::raw(": Copy / Paste (Body / Response)"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            " Command Mode Actions ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]),
        Line::from(vec![
            Span::styled("  :import <path> ", Style::default().fg(Color::Cyan)),
            Span::raw(": Import a Postman collection"),
        ]),
        Line::from(vec![
            Span::styled("  :parse <path>  ", Style::default().fg(Color::Cyan)),
            Span::raw(": Parse project from path (default: . )"),
        ]),
        Line::from(vec![
            Span::styled("  :env create    ", Style::default().fg(Color::Cyan)),
            Span::raw(": Auto-generate variables (baseUrl)"),
        ]),
        Line::from(vec![
            Span::styled("  :q / :quit     ", Style::default().fg(Color::Cyan)),
            Span::raw(": Quit application"),
        ]),
    ];

    let p = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

pub fn render_method_search(f: &mut Frame, app: &App) {
    let area = centered_rect(20, 30, f.area());
    f.render_widget(Clear, area); // Clear the background

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search box
            Constraint::Min(0),    // Results
        ])
        .split(area);

    // Search Box
    let search_block = Block::default()
        .title(" Search Method ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let search_text = Paragraph::new(app.method_search_query.as_str()).block(search_block);
    f.render_widget(search_text, chunks[0]);

    // Results logic
    let all_methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE"];
    let filtered_methods: Vec<&str> = all_methods
        .into_iter()
        .filter(|m| m.contains(&app.method_search_query.to_uppercase()))
        .collect();

    let list_items: Vec<ListItem> = filtered_methods
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let color = get_method_color(m);
            let style = if i == 0 {
                // Highlight the top match
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::REVERSED | Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };
            ListItem::new(*m).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM));
    f.render_widget(list, chunks[1]);
}

pub fn render_autocomplete(f: &mut Frame, app: &App) {
    let options = app.get_autocomplete_options();
    if options.is_empty() {
        return;
    }

    let (cursor_x, cursor_y) = app.last_cursor_pos;

    // Width of the dropdown based on longest option or query
    let width = options.iter().map(|s| s.len()).max().unwrap_or(10) as u16 + 4;
    let height = (options.len().min(8) + 2) as u16;

    // Adjust position if it goes off screen
    let x = if cursor_x + width > f.area().width {
        f.area().width.saturating_sub(width)
    } else {
        cursor_x
    };

    let y = if cursor_y + height + 1 > f.area().height {
        // Show above cursor if no space below
        cursor_y.saturating_sub(height)
    } else {
        cursor_y + 1
    };

    let area = Rect::new(x, y, width, height);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Variables ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let style = if i == app.autocomplete_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::REVERSED | Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!(" {} ", opt)).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
