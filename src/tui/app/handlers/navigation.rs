use super::super::enums::*;
use super::super::state::App;

impl App {
    pub fn focus_request_bar(&mut self) {
        self.focused_panel = FocusedPanel::RequestBar;
        self.active_request_part = RequestBarPart::Url;
        self.input_mode = InputMode::Normal;
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            let mut new_pos = self.cursor_position;
            while new_pos > 0 {
                new_pos -= 1;
                if self.url.is_char_boundary(new_pos) {
                    break;
                }
            }
            self.cursor_position = new_pos;
        }
    }

    pub fn move_cursor_right(&mut self, max: usize) {
        if self.cursor_position < max {
            let mut new_pos = self.cursor_position;
            while new_pos < max {
                new_pos += 1;
                if self.url.is_char_boundary(new_pos) {
                    break;
                }
            }
            self.cursor_position = new_pos;
        }
    }

    pub fn insert_char(&mut self, target: &mut String, c: char) {
        let pos = self.cursor_position.min(target.len());
        target.insert(pos, c);
        self.cursor_position = pos + c.len_utf8();
    }

    pub fn delete_char(&mut self, target: &mut String) {
        if self.cursor_position > 0 {
            let mut prev_pos = self.cursor_position;
            while prev_pos > 0 {
                prev_pos -= 1;
                if target.is_char_boundary(prev_pos) {
                    break;
                }
            }
            target.remove(prev_pos);
            self.cursor_position = prev_pos;
        }
    }

    pub fn delete_char_forward(&mut self, target: &mut String) {
        if self.cursor_position < target.len() {
            let mut next_pos = self.cursor_position;
            let mut i = 0;
            while i < 4 && next_pos < target.len() {
                next_pos += 1;
                if target.is_char_boundary(next_pos) {
                    break;
                }
                i += 1;
            }
            target.remove(self.cursor_position);
        }
    }

    pub fn insert_char_rename(&mut self, c: char) {
        let pos = self.cursor_position.min(self.rename_input.len());
        self.rename_input.insert(pos, c);
        self.cursor_position = pos + c.len_utf8();
    }

    pub fn delete_char_rename(&mut self) {
        if self.cursor_position > 0 {
            let mut prev_pos = self.cursor_position;
            while prev_pos > 0 {
                prev_pos -= 1;
                if self.rename_input.is_char_boundary(prev_pos) {
                    break;
                }
            }
            self.rename_input.remove(prev_pos);
            self.cursor_position = prev_pos;
        }
    }

    pub fn delete_char_forward_rename(&mut self) {
        if self.cursor_position < self.rename_input.len() {
            self.rename_input.remove(self.cursor_position);
        }
    }

    pub fn insert_char_url(&mut self, c: char) {
        let pos = self.cursor_position.min(self.url.len());
        self.url.insert(pos, c);
        self.cursor_position = pos + c.len_utf8();
    }

    pub fn delete_char_url(&mut self) {
        if self.cursor_position > 0 {
            let mut prev_pos = self.cursor_position;
            while prev_pos > 0 {
                prev_pos -= 1;
                if self.url.is_char_boundary(prev_pos) {
                    break;
                }
            }
            self.url.remove(prev_pos);
            self.cursor_position = prev_pos;
        }
    }

    pub fn delete_char_forward_url(&mut self) {
        if self.cursor_position < self.url.len() {
            self.url.remove(self.cursor_position);
        }
    }

    pub fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Collections => {
                if self.left_bottom_tab == LeftBottomTab::Apis {
                    FocusedPanel::Apis
                } else {
                    FocusedPanel::Environments
                }
            }
            FocusedPanel::Apis | FocusedPanel::Environments => FocusedPanel::RequestBar,
            FocusedPanel::RequestBar => FocusedPanel::Details,
            FocusedPanel::Properties => FocusedPanel::Details,
            FocusedPanel::Details => FocusedPanel::Response,
            FocusedPanel::Response => FocusedPanel::Stats,
            FocusedPanel::Stats => FocusedPanel::Collections,
        };
        self.input_mode = InputMode::Normal;
    }

    pub fn prev_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Collections => FocusedPanel::Stats,
            FocusedPanel::Apis | FocusedPanel::Environments => FocusedPanel::Collections,
            FocusedPanel::RequestBar => {
                if self.left_bottom_tab == LeftBottomTab::Apis {
                    FocusedPanel::Apis
                } else {
                    FocusedPanel::Environments
                }
            }
            FocusedPanel::Properties => FocusedPanel::RequestBar,
            FocusedPanel::Details => FocusedPanel::RequestBar,
            FocusedPanel::Response => FocusedPanel::Details,
            FocusedPanel::Stats => FocusedPanel::Response,
        };
        self.input_mode = InputMode::Normal;
    }

    pub fn next_property_tab(&mut self) {
        self.selected_property_tab = match self.selected_property_tab {
            PropertyTab::Params => PropertyTab::Headers,
            PropertyTab::Headers => PropertyTab::Auth,
            PropertyTab::Auth => PropertyTab::Body,
            PropertyTab::Body => PropertyTab::Scripts,
            PropertyTab::Scripts => PropertyTab::Params,
        };
        self.property_editor_row = 0;
        self.details_scroll = 0;
    }

    pub fn prev_property_tab(&mut self) {
        self.selected_property_tab = match self.selected_property_tab {
            PropertyTab::Params => PropertyTab::Scripts,
            PropertyTab::Headers => PropertyTab::Params,
            PropertyTab::Auth => PropertyTab::Headers,
            PropertyTab::Body => PropertyTab::Auth,
            PropertyTab::Scripts => PropertyTab::Body,
        };
        self.property_editor_row = 0;
        self.details_scroll = 0;
    }

    pub fn clamp_selections(&mut self) {
        let collections = self.get_visible_collections();
        if !collections.is_empty() {
            self.selected_collection_index = self
                .selected_collection_index
                .min(collections.len().saturating_sub(1));
        } else {
            self.selected_collection_index = 0;
        }

        let items = self.get_visible_items();
        if !items.is_empty() {
            self.selected_api_index = self.selected_api_index.min(items.len().saturating_sub(1));
        } else {
            self.selected_api_index = 0;
        }
    }

    pub fn pop_up(&mut self) {
        if self.show_method_search {
            self.show_method_search = false;
            self.method_search_query.clear();
            return;
        }
        if self.error_message.is_some() {
            self.error_message = None;
            return;
        }
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Details => FocusedPanel::RequestBar,
            FocusedPanel::Properties => FocusedPanel::RequestBar,
            FocusedPanel::Response => FocusedPanel::Details,
            FocusedPanel::Stats => FocusedPanel::Response,
            _ => self.focused_panel,
        };
        self.input_mode = InputMode::Normal;
    }

    pub fn reset_scroll(&mut self) {
        self.response_scroll = 0;
        self.response_horizontal_scroll = 0;
        self.details_scroll = 0;
    }
}
