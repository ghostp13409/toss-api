use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Text};
use std::sync::LazyLock;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tui_syntax_highlight::Highlighter;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

pub fn format_content(content: &str, content_type: Option<&str>) -> String {
    let ct = content_type.unwrap_or("").to_lowercase();

    if ct.contains("json") {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                return pretty;
            }
        }
    } else if ct.contains("html")
        || content.trim_start().starts_with("<!DOCTYPE")
        || content.trim_start().starts_with("<html")
    {
        return format_html(content);
    }

    content.to_string()
}

pub fn highlight_content(content: &str, content_type: Option<&str>) -> Text<'static> {
    let ct = content_type.unwrap_or("").to_lowercase();
    let extension = if ct.contains("json") {
        "json"
    } else if ct.contains("html") {
        "html"
    } else if ct.contains("xml") {
        "xml"
    } else {
        "txt"
    };

    let syntax = SYNTAX_SET
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    // Using a default dark theme
    let theme = &THEME_SET.themes["base16-ocean.dark"];
    let highlighter = Highlighter::new(theme.clone());

    match highlighter.highlight_lines(content.lines(), syntax, &SYNTAX_SET) {
        Ok(text) => text,
        Err(_) => Text::raw(content.to_string()),
    }
}

pub fn apply_env_vars(text: &mut Text<'static>) {
    for line in &mut text.lines {
        let mut new_spans = Vec::new();
        for span in std::mem::take(&mut line.spans) {
            let content = span.content.to_string(); // Need to convert to String for char iteration
            let style = span.style;

            let mut last_pos = 0;
            let chars: Vec<char> = content.chars().collect();
            let mut i = 0;

            let mut found_any = false;
            while i < chars.len() {
                if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
                    if i > last_pos {
                        new_spans.push(Span::styled(
                            chars[last_pos..i].iter().collect::<String>(),
                            style,
                        ));
                    }

                    let mut found_end = false;
                    for j in i + 2..chars.len().saturating_sub(1) {
                        if chars[j] == '}' && chars[j + 1] == '}' {
                            new_spans.push(Span::styled(
                                chars[i..j + 2].iter().collect::<String>(),
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ));
                            i = j + 2;
                            last_pos = i;
                            found_end = true;
                            found_any = true;
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

            if last_pos < chars.len() {
                new_spans.push(Span::styled(
                    chars[last_pos..].iter().collect::<String>(),
                    style,
                ));
            }

            if !found_any {
                new_spans.pop();
                new_spans.push(Span::styled(content, style));
            }
        }
        line.spans = new_spans;
    }
}

fn format_html(html: &str) -> String {
    let mut formatted = String::new();
    let mut indent: usize = 0;

    let mut parts = Vec::new();
    let mut current = String::new();

    for c in html.chars() {
        if c == '<' {
            if !current.trim().is_empty() {
                parts.push((false, current.trim().to_string()));
            }
            current = "<".to_string();
        } else if c == '>' {
            current.push('>');
            parts.push((true, current.clone()));
            current.clear();
        } else {
            current.push(c);
        }
    }

    for (is_tag, part) in parts {
        if is_tag {
            if part.starts_with("</") {
                indent = indent.saturating_sub(1);
                formatted.push_str(&"  ".repeat(indent));
                formatted.push_str(&part);
                formatted.push('\n');
            } else if part.ends_with("/>") || part.starts_with("<!") || part.starts_with("<?") {
                formatted.push_str(&"  ".repeat(indent));
                formatted.push_str(&part);
                formatted.push('\n');
            } else {
                formatted.push_str(&"  ".repeat(indent));
                formatted.push_str(&part);
                formatted.push('\n');
                indent += 1;
            }
        } else {
            formatted.push_str(&"  ".repeat(indent));
            formatted.push_str(&part);
            formatted.push('\n');
        }
    }

    if formatted.is_empty() {
        return html.to_string();
    }
    formatted
}
