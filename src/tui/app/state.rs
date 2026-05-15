use super::enums::*;
use crate::cli::args::Method;
use crate::core::collection::Collection;
use ratatui::widgets::{ListState, TableState};

pub struct App {
    pub input_mode: InputMode,
    pub focused_panel: FocusedPanel,
    pub left_bottom_tab: LeftBottomTab,
    pub active_request_part: RequestBarPart,
    pub show_method_search: bool,
    pub method_search_query: String,
    pub show_search: bool,
    pub search_query: String,
    pub url: String,
    pub method: Method,
    pub command_input: String,
    pub cursor_position: usize,
    pub should_quit: bool,
    pub collections: Vec<Collection>,
    pub current_request_id: Option<String>,
    pub active_collection_index: usize,
    pub active_folder_id: Option<String>,
    pub selected_collection_index: usize,
    pub selected_api_index: usize,
    pub rename_input: String,
    pub pending_item_type: Option<PendingItemType>,
    pub error_message: Option<String>,
    pub should_delete_item: bool,
    pub selected_property_tab: PropertyTab,
    pub property_editor_row: usize,
    pub property_editor_field: PropertyEditorField,
    pub response_body: String,
    pub response_content_type: Option<String>,
    pub response_status: Option<String>,
    pub response_stats: String,
    pub pending_actions: Vec<TuiAction>,
    pub response_scroll: u16,
    pub response_horizontal_scroll: u16,
    pub details_scroll: usize,
    pub collections_state: ListState,
    pub apis_state: ListState,
    pub details_table_state: TableState,
    pub environments_table_state: TableState,
    pub g_pressed: bool,
    pub mask_env_values: bool,
    pub selected_env_index: usize,
    pub show_autocomplete: bool,
    pub autocomplete_query: String,
    pub autocomplete_index: usize,
    pub last_cursor_pos: (u16, u16),
}

pub struct VisibleItem {
    pub name: String,
    pub depth: usize,
    pub item_type: VisibleItemType,
}

impl VisibleItem {
    pub fn item_type_depth(&self) -> usize {
        match self.item_type {
            VisibleItemType::Collection { .. } => self.depth,
            _ => self.depth,
        }
    }
}
