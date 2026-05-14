use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Paragraph,
};
use crate::tui::app::{App, FocusedPanel, InputMode};

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.input_mode == InputMode::Command {
        format!(":{}", app.command_input)
    } else if app.input_mode == InputMode::Rename {
        "Enter: Confirm | Esc: Cancel".to_string()
    } else if app.input_mode == InputMode::CreateItem {
        "Enter: Create (Empty for default) | Esc: Cancel".to_string()
    } else if app.input_mode == InputMode::Search {
        format!("Filter: {} (Esc: Clear, Enter: Keep)", app.search_query)
    } else if app.input_mode == InputMode::ConfirmDelete {
        "ARE YOU SURE? (y/n)".to_string()
    } else {
        match app.focused_panel {
            FocusedPanel::Collections | FocusedPanel::Apis => {
                "gg/G: Top/Bottom | V: Variables | /: Filter | Space: Toggle | a: Req | f: Folder | n: Collection | r: Rename | d: Delete".to_string()
            }
            FocusedPanel::Environments => {
                "gg/G: Top/Bottom | A: APIs | m: Mask | a: Add | d: Delete | Enter: Edit | Esc: Back".to_string()
            }
            FocusedPanel::RequestBar => "[Request] Tab: Cycle Controls | Enter: Trigger | Esc: Back".to_string(),
            FocusedPanel::Properties => "h/l: Switch Tabs | j/k: Nav Rows | Enter: Edit | a: Add | d: Delete | Esc: Back".to_string(),
            FocusedPanel::Details => "gg/G: Top/Bottom | Ctrl+u/d: PgUp/Dn | Esc: Back | Enter: Edit | Arrows: Nav Field".to_string(),
            FocusedPanel::Response | FocusedPanel::Stats => "gg/G: Top/Bottom | Ctrl+u/d: PgUp/Dn | h/j/k/l: Scroll | Esc: Back".to_string(),
        }
    };

    let p = Paragraph::new(text).style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_widget(p, area);
}
