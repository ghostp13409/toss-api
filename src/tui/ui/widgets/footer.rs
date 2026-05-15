use crate::tui::app::{App, FocusedPanel, InputMode};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = Vec::new();

    // Mode Tag
    let (mode_text, mode_style) = match app.input_mode {
        InputMode::Normal => (
            " NORMAL ",
            Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Editing => (
            " INSERT ",
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Command => (
            " COMMAND ",
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Search => (
            " SEARCH ",
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Help => (
            " HELP ",
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Rename => (
            " RENAME ",
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::CreateItem => (
            " CREATE ",
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::ConfirmDelete => (
            " DELETE? ",
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::ConfirmQuit => (
            " QUIT? ",
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    };
    spans.push(Span::styled(mode_text, mode_style));
    spans.push(Span::raw(" "));

    match app.input_mode {
        InputMode::Command => {
            spans.push(Span::styled(
                ":",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(&app.command_input));
        }
        InputMode::Rename => {
            spans.extend(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Confirm | "),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Cancel"),
            ]);
        }
        InputMode::CreateItem => {
            spans.extend(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Create (Empty for default) | "),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Cancel"),
            ]);
        }
        InputMode::Search => {
            spans.extend(vec![
                Span::styled(
                    "Filter",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": "),
                Span::raw(&app.search_query),
                Span::raw(" ("),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Clear, "),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Keep)"),
            ]);
        }
        InputMode::ConfirmDelete => {
            spans.push(Span::styled(
                "ARE YOU SURE? (y/n)",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ));
        }
        InputMode::ConfirmQuit => {
            spans.push(Span::styled(
                "QUIT APPLICATION? (y/n)",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        InputMode::Help => {
            spans.push(Span::styled(
                "Press Esc/q/? to close Help",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        InputMode::Editing => {
            spans.extend(vec![
                Span::styled(
                    "ESC",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Exit Editing Mode | "),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Confirm Change"),
            ]);
        }
        InputMode::Normal => {
            let panel_shortcuts = match app.focused_panel {
                FocusedPanel::Collections | FocusedPanel::Apis => {
                    vec![
                        ("gg/G", "Top/Bottom"),
                        ("/", "Filter"),
                        ("Space", "Toggle"),
                        ("a", "Req"),
                        ("f", "Folder"),
                        ("n", "Col"),
                        ("r", "Rename"),
                        ("d", "Delete"),
                        ("e", "URL"),
                    ]
                }
                FocusedPanel::Environments => {
                    vec![
                        ("gg/G", "Top/Bottom"),
                        ("m", "Mask"),
                        ("a", "Add"),
                        ("d", "Delete"),
                        ("Enter", "Edit"),
                        ("Esc", "Back"),
                    ]
                }
                FocusedPanel::RequestBar => {
                    vec![
                        ("Tab", "Cycle"),
                        ("Enter", "Edit/Send"),
                        ("Esc", "Back"),
                        ("C-Enter", "Send"),
                    ]
                }
                FocusedPanel::Properties => {
                    vec![
                        ("h/l", "Tabs"),
                        ("j/k", "Nav Rows"),
                        ("Enter", "Focus"),
                        ("Esc", "Back"),
                    ]
                }
                FocusedPanel::Details => {
                    vec![
                        ("Enter", "Edit"),
                        ("a", "Add"),
                        ("d", "Delete"),
                        ("t", "Type"),
                        ("v", "Edit Body"),
                        ("y/p", "Cpy/Pst"),
                        ("Esc", "Back"),
                    ]
                }
                FocusedPanel::Response | FocusedPanel::Stats => {
                    vec![
                        ("gg/G", "Top/Bottom"),
                        ("y/Y", "Copy"),
                        ("h/j/k/l", "Scroll"),
                        ("Esc", "Back"),
                    ]
                }
            };

            for (i, (key, action)) in panel_shortcuts.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(" | "));
                }
                spans.push(Span::styled(
                    *key,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(": "));
                spans.push(Span::raw(*action));
            }

            // Add Help shortcut at the end
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                "?",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(": Help"));
        }
    }

    let p = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(30, 30, 30)));
    f.render_widget(p, area);
}
