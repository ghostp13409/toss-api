use super::super::enums::*;
use super::super::state::{App, VisibleItem};
use crate::cli::args::Method;
use crate::core::collection::{Collection, CollectionItem, Request};

fn matches_query(target: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let t = target.to_lowercase().replace([' ', '-', '_'], "");
    let q = query.to_lowercase().replace([' ', '-', '_'], "");
    t.contains(&q)
}

impl App {
    pub fn update_active_scope_from_tree(&mut self) {
        let visible = self.get_visible_collections();
        if let Some(_item) = visible.get(self.selected_collection_index) {
            let mut current_idx = 0;
            for (i, col) in self.collections.iter().enumerate() {
                if current_idx == self.selected_collection_index {
                    self.active_collection_index = i;
                    self.active_folder_id = None;
                    return;
                }
                current_idx += 1;
                if col.expanded {
                    let mut found_id = None;
                    if self.find_container_id_at_index(
                        &col.items,
                        &mut current_idx,
                        self.selected_collection_index,
                        &mut found_id,
                    ) {
                        self.active_collection_index = i;
                        self.active_folder_id = found_id;
                        return;
                    }
                }
            }
        }
    }

    fn find_container_id_at_index(
        &self,
        items: &[CollectionItem],
        current_idx: &mut usize,
        target_idx: usize,
        found_id: &mut Option<String>,
    ) -> bool {
        for item in items {
            if *current_idx == target_idx {
                match item {
                    CollectionItem::Folder(f) => *found_id = Some(f.id.clone()),
                    _ => {}
                }
                return true;
            }
            *current_idx += 1;
            if let CollectionItem::Folder(f) = item {
                if f.expanded {
                    let prev_found = found_id.clone();
                    *found_id = Some(f.id.clone());
                    if self.find_container_id_at_index(&f.items, current_idx, target_idx, found_id)
                    {
                        return true;
                    }
                    *found_id = prev_found;
                }
            }
        }
        false
    }

    pub fn toggle_folder(&mut self) {
        if self.focused_panel == FocusedPanel::Collections {
            let visible = self.get_visible_collections();
            if let Some(item) = visible.get(self.selected_collection_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_collection_index;
                for col in &mut self.collections {
                    if current_idx == target_idx {
                        col.expanded = !col.expanded;
                        return;
                    }
                    current_idx += 1;
                    if col.expanded {
                        for it in &mut col.items {
                            if Self::find_and_toggle_folder_recursive(
                                it,
                                1,
                                target_depth,
                                &target_name,
                                &mut current_idx,
                                target_idx,
                            ) {
                                return;
                            }
                        }
                    }
                }
            }
        } else {
            let visible = self.get_visible_items();
            if let Some(item) = visible.get(self.selected_api_index) {
                if let VisibleItemType::Folder { .. } = item.item_type {
                    let target_name = item.name.clone();
                    let target_depth = item.depth;
                    let mut current_idx = 0;
                    let target_idx = self.selected_api_index;
                    if let Some(col) = self.collections.get_mut(self.active_collection_index) {
                        for it in &mut col.items {
                            if Self::find_and_toggle_folder_recursive(
                                it,
                                0,
                                target_depth,
                                &target_name,
                                &mut current_idx,
                                target_idx,
                            ) {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn find_and_toggle_folder_recursive(
        item: &mut CollectionItem,
        current_depth: usize,
        target_depth: usize,
        target_name: &str,
        current_idx: &mut usize,
        target_idx: usize,
    ) -> bool {
        match item {
            CollectionItem::Folder(f) => {
                if current_depth == target_depth
                    && f.name == target_name
                    && *current_idx == target_idx
                {
                    f.expanded = !f.expanded;
                    return true;
                }
                *current_idx += 1;
                if f.expanded {
                    for sub in &mut f.items {
                        if Self::find_and_toggle_folder_recursive(
                            sub,
                            current_depth + 1,
                            target_depth,
                            target_name,
                            current_idx,
                            target_idx,
                        ) {
                            return true;
                        }
                    }
                }
            }
            CollectionItem::Request(_) => *current_idx += 1,
        }
        false
    }

    pub fn rename_item(&mut self) {
        let new_name = self.rename_input.clone();
        if self.focused_panel == FocusedPanel::Collections {
            let visible = self.get_visible_collections();
            if let Some(item) = visible.get(self.selected_collection_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_collection_index;
                for col in &mut self.collections {
                    if current_idx == target_idx {
                        col.name = new_name;
                        return;
                    }
                    current_idx += 1;
                    if col.expanded {
                        for it in &mut col.items {
                            if Self::find_and_rename_recursive(
                                it,
                                1,
                                target_depth,
                                &target_name,
                                &mut current_idx,
                                target_idx,
                                &new_name,
                            ) {
                                return;
                            }
                        }
                    }
                }
            }
        } else {
            let visible = self.get_visible_items();
            if let Some(item) = visible.get(self.selected_api_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_api_index;
                if let Some(col) = self.collections.get_mut(self.active_collection_index) {
                    for it in &mut col.items {
                        if Self::find_and_rename_recursive(
                            it,
                            0,
                            target_depth,
                            &target_name,
                            &mut current_idx,
                            target_idx,
                            &new_name,
                        ) {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn find_and_rename_recursive(
        item: &mut CollectionItem,
        current_depth: usize,
        target_depth: usize,
        target_name: &str,
        current_idx: &mut usize,
        target_idx: usize,
        new_name: &str,
    ) -> bool {
        let name = match item {
            CollectionItem::Folder(f) => &mut f.name,
            CollectionItem::Request(r) => &mut r.name,
        };
        if current_depth == target_depth && *name == target_name && *current_idx == target_idx {
            *name = new_name.to_string();
            return true;
        }
        *current_idx += 1;
        if let CollectionItem::Folder(f) = item {
            if f.expanded {
                for sub in &mut f.items {
                    if Self::find_and_rename_recursive(
                        sub,
                        current_depth + 1,
                        target_depth,
                        target_name,
                        current_idx,
                        target_idx,
                        new_name,
                    ) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn delete_item(&mut self) {
        if self.focused_panel == FocusedPanel::Collections {
            let visible = self.get_visible_collections();
            if let Some(item) = visible.get(self.selected_collection_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_collection_index;
                let mut to_remove = None;
                for (i, col) in self.collections.iter_mut().enumerate() {
                    if current_idx == target_idx {
                        to_remove = Some(i);
                        break;
                    }
                    current_idx += 1;
                    if col.expanded {
                        let mut item_to_remove = None;
                        for (j, it) in col.items.iter_mut().enumerate() {
                            if Self::find_and_delete_recursive(
                                it,
                                1,
                                target_depth,
                                &target_name,
                                &mut current_idx,
                                target_idx,
                            ) {
                                item_to_remove = Some(j);
                                break;
                            }
                        }
                        if let Some(j) = item_to_remove {
                            col.items.remove(j);
                            return;
                        }
                    }
                }
                if let Some(i) = to_remove {
                    self.collections.remove(i);
                }
            }
        } else {
            let visible = self.get_visible_items();
            if let Some(item) = visible.get(self.selected_api_index) {
                let target_name = item.name.clone();
                let target_depth = item.depth;
                let mut current_idx = 0;
                let target_idx = self.selected_api_index;
                if let Some(col) = self.collections.get_mut(self.active_collection_index) {
                    let mut item_to_remove = None;
                    for (i, it) in col.items.iter_mut().enumerate() {
                        if Self::find_and_delete_recursive(
                            it,
                            0,
                            target_depth,
                            &target_name,
                            &mut current_idx,
                            target_idx,
                        ) {
                            item_to_remove = Some(i);
                            break;
                        }
                    }
                    if let Some(i) = item_to_remove {
                        col.items.remove(i);
                    }
                }
            }
        }
    }

    fn find_and_delete_recursive(
        item: &mut CollectionItem,
        current_depth: usize,
        target_depth: usize,
        target_name: &str,
        current_idx: &mut usize,
        target_idx: usize,
    ) -> bool {
        let name = match item {
            CollectionItem::Folder(f) => &f.name,
            CollectionItem::Request(r) => &r.name,
        };
        if current_depth == target_depth && *name == target_name && *current_idx == target_idx {
            return true;
        }
        *current_idx += 1;
        if let CollectionItem::Folder(f) = item {
            if f.expanded {
                let mut sub_to_remove = None;
                for (i, sub) in f.items.iter_mut().enumerate() {
                    if Self::find_and_delete_recursive(
                        sub,
                        current_depth + 1,
                        target_depth,
                        target_name,
                        current_idx,
                        target_idx,
                    ) {
                        sub_to_remove = Some(i);
                        break;
                    }
                }
                if let Some(i) = sub_to_remove {
                    f.items.remove(i);
                    return false; // Already handled
                }
            }
        }
        false
    }

    pub fn add_collection(&mut self, name: String) {
        let name = if name.is_empty() {
            "New Collection".to_string()
        } else {
            name
        };
        self.collections.push(Collection::new(name));
    }

    pub fn add_folder(&mut self, name: String) {
        let name = if name.is_empty() {
            "New Folder".to_string()
        } else {
            name
        };
        let new_folder = crate::core::collection::Folder::new(name);
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            col.items.push(CollectionItem::Folder(new_folder));
        }
    }

    pub fn add_request(&mut self, name: String) {
        let name = if name.is_empty() {
            "New Request".to_string()
        } else {
            name
        };
        let new_req = Request::new(name, Method::Get, "https://httpbin.org/get".to_string());
        if let Some(col) = self.collections.get_mut(self.active_collection_index) {
            col.items.push(CollectionItem::Request(new_req));
        }
    }

    pub fn get_visible_collections(&self) -> Vec<VisibleItem> {
        let mut visible_items = Vec::new();
        let query = &self.search_query;

        for col in &self.collections {
            let col_matches = matches_query(&col.name, query);
            let any_child_matches = col
                .items
                .iter()
                .any(|item| Self::any_descendant_matches(item, query));

            if !query.is_empty() && self.focused_panel == FocusedPanel::Collections {
                if !col_matches && !any_child_matches {
                    continue;
                }
            }

            visible_items.push(VisibleItem {
                name: col.name.clone(),
                depth: 0,
                item_type: VisibleItemType::Collection {
                    expanded: if !query.is_empty() && any_child_matches {
                        true
                    } else {
                        col.expanded
                    },
                },
            });

            if !query.is_empty() && self.focused_panel == FocusedPanel::Collections {
                for item in &col.items {
                    Self::collect_filtered_collections_recursive(
                        item,
                        1,
                        query,
                        &mut visible_items,
                        col_matches,
                    );
                }
            } else if col.expanded {
                for item in &col.items {
                    Self::collect_visible_items_recursive(item, 1, &mut visible_items);
                }
            }
        }
        visible_items
    }

    fn any_descendant_matches(item: &CollectionItem, query: &str) -> bool {
        match item {
            CollectionItem::Request(r) => matches_query(&r.name, query),
            CollectionItem::Folder(f) => {
                if matches_query(&f.name, query) {
                    return true;
                }
                f.items
                    .iter()
                    .any(|sub| Self::any_descendant_matches(sub, query))
            }
        }
    }

    fn collect_filtered_collections_recursive(
        item: &CollectionItem,
        depth: usize,
        query: &str,
        visible_items: &mut Vec<VisibleItem>,
        parent_matches: bool,
    ) {
        match item {
            CollectionItem::Folder(f) => {
                let matches = matches_query(&f.name, query);
                let any_child_matches = f
                    .items
                    .iter()
                    .any(|sub| Self::any_descendant_matches(sub, query));

                if parent_matches || matches || any_child_matches {
                    visible_items.push(VisibleItem {
                        name: f.name.clone(),
                        depth,
                        item_type: VisibleItemType::Folder {
                            expanded: if !query.is_empty() && any_child_matches {
                                true
                            } else {
                                f.expanded
                            },
                        },
                    });
                    for sub_item in &f.items {
                        Self::collect_filtered_collections_recursive(
                            sub_item,
                            depth + 1,
                            query,
                            visible_items,
                            parent_matches || matches,
                        );
                    }
                }
            }
            CollectionItem::Request(r) => {
                if parent_matches || matches_query(&r.name, query) {
                    visible_items.push(VisibleItem {
                        name: r.name.clone(),
                        depth,
                        item_type: VisibleItemType::Request {
                            method: r.method,
                            id: r.id.clone(),
                        },
                    });
                }
            }
        }
    }

    pub fn get_visible_items(&self) -> Vec<VisibleItem> {
        let mut visible_items = Vec::new();
        let query = &self.search_query;

        if let Some(col) = self.collections.get(self.active_collection_index) {
            let items = if let Some(folder_id) = &self.active_folder_id {
                Self::find_folder_items(&col.items, folder_id).unwrap_or(&col.items)
            } else {
                &col.items
            };

            if !query.is_empty() && self.focused_panel == FocusedPanel::Apis {
                for item in items {
                    Self::collect_filtered_items_recursive(
                        item,
                        0,
                        query,
                        &mut visible_items,
                        false,
                    );
                }
            } else {
                for item in items {
                    Self::collect_visible_items_recursive(item, 0, &mut visible_items);
                }
            }
        }
        visible_items
    }

    fn collect_filtered_items_recursive(
        item: &CollectionItem,
        depth: usize,
        query: &str,
        visible_items: &mut Vec<VisibleItem>,
        parent_matches: bool,
    ) {
        match item {
            CollectionItem::Folder(f) => {
                let matches = matches_query(&f.name, query);
                let any_child_matches = f
                    .items
                    .iter()
                    .any(|sub| Self::any_descendant_matches(sub, query));

                if parent_matches || matches || any_child_matches {
                    visible_items.push(VisibleItem {
                        name: f.name.clone(),
                        depth,
                        item_type: VisibleItemType::Folder {
                            expanded: if !query.is_empty() && any_child_matches {
                                true
                            } else {
                                f.expanded
                            },
                        },
                    });
                    for sub_item in &f.items {
                        Self::collect_filtered_items_recursive(
                            sub_item,
                            depth + 1,
                            query,
                            visible_items,
                            parent_matches || matches,
                        );
                    }
                }
            }
            CollectionItem::Request(r) => {
                if parent_matches || matches_query(&r.name, query) {
                    visible_items.push(VisibleItem {
                        name: r.name.clone(),
                        depth,
                        item_type: VisibleItemType::Request {
                            method: r.method,
                            id: r.id.clone(),
                        },
                    });
                }
            }
        }
    }

    fn find_folder_items<'a>(
        items: &'a [CollectionItem],
        folder_id: &str,
    ) -> Option<&'a Vec<CollectionItem>> {
        for item in items {
            if let CollectionItem::Folder(f) = item {
                if f.id == folder_id {
                    return Some(&f.items);
                }
                if let Some(found) = Self::find_folder_items(&f.items, folder_id) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn collect_visible_items_recursive(
        item: &CollectionItem,
        depth: usize,
        visible_items: &mut Vec<VisibleItem>,
    ) {
        match item {
            CollectionItem::Folder(f) => {
                visible_items.push(VisibleItem {
                    name: f.name.clone(),
                    depth,
                    item_type: VisibleItemType::Folder {
                        expanded: f.expanded,
                    },
                });
                if f.expanded {
                    for sub_item in &f.items {
                        Self::collect_visible_items_recursive(sub_item, depth + 1, visible_items);
                    }
                }
            }
            CollectionItem::Request(r) => {
                visible_items.push(VisibleItem {
                    name: r.name.clone(),
                    depth,
                    item_type: VisibleItemType::Request {
                        method: r.method,
                        id: r.id.clone(),
                    },
                });
            }
        }
    }
}
