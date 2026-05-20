use std::borrow::Cow;
use crate::cli::args::Method;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
};

pub fn get_method_color(method_str: &str) -> Color {
    match method_str.to_uppercase().as_str() {
        "GET" => Color::Green,
        "POST" => Color::Yellow,
        "PUT" => Color::Blue,
        "PATCH" => Color::Magenta,
        "DELETE" => Color::Red,
        _ => Color::Reset,
    }
}

pub fn get_method_enum_color(method: Method) -> Color {
    match method {
        Method::Get => Color::Green,
        Method::Post => Color::Yellow,
        Method::Put => Color::Blue,
        Method::Patch => Color::Magenta,
        Method::Delete => Color::Red,
    }
}

pub fn highlight_env_vars<'a>(text: &'a str) -> Line<'static> {
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut start = 0;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
            // Push previous raw text
            if i > start {
                spans.push(Span::raw(chars[start..i].iter().collect::<String>()));
            }

            let mut found_end = false;
            for j in i + 2..chars.len().saturating_sub(1) {
                if chars[j] == '}' && chars[j + 1] == '}' {
                    spans.push(Span::styled(
                        chars[i..j + 2].iter().collect::<String>(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                    i = j + 2;
                    start = i;
                    found_end = true;
                    break;
                }
            }

            if !found_end {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    if start < chars.len() {
        spans.push(Span::raw(chars[start..].iter().collect::<String>()));
    }

    if spans.is_empty() && !text.is_empty() {
        spans.push(Span::raw(text.to_string()));
    }

    Line::from(spans)
}

pub fn title_with_key<'a, S>(key: &'a str, title: S) -> Line<'a>
where
    S: Into<Cow<'a, str>>,
{
    Line::from(vec![
        Span::raw(" "),
        Span::styled(
            key,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::raw(title),
        Span::raw(" "),
    ])
}

pub fn create_block<'a, T>(title: T, focused: bool) -> Block<'a>
where
    T: Into<ratatui::text::Line<'a>>,
{
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn extract_json_value(line: &str) -> (Option<String>, String) {
    let line = line.trim();

    if let Some(colon_pos) = line.find(':') {
        let key_part = line[..colon_pos].trim();
        let value_part = line[colon_pos + 1..].trim();
        
        let key = if key_part.starts_with('"') && key_part.ends_with('"') && key_part.len() >= 2 {
            Some(key_part[1..key_part.len() - 1].to_string())
        } else {
            None
        };

        // Remove trailing comma if present
        let value_part = value_part.trim_end_matches(',');

        let value = if value_part.starts_with('"') && value_part.ends_with('"') && value_part.len() >= 2 {
            // String value
            value_part[1..value_part.len() - 1].to_string()
        } else {
            value_part.to_string()
        };

        return (key, value);
    }

    // Fallback: if it's just a string in quotes (array element)
    let line = line.trim_end_matches(',');
    if line.starts_with('"') && line.ends_with('"') && line.len() >= 2 {
        return (None, line[1..line.len() - 1].to_string());
    }

    (None, line.to_string())
}
