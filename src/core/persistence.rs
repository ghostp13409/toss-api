use crate::core::collection::Collection;
use std::fs;
use std::path::PathBuf;

pub struct PersistenceManager {
    base_dir: PathBuf,
}

impl PersistenceManager {
    pub fn new() -> Self {
        let mut base_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base_dir.push("toss");
        if !base_dir.exists() {
            let _ = fs::create_dir_all(&base_dir);
        }
        Self { base_dir }
    }

    pub fn save_collections(
        &self,
        collections: &[Collection],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = self.base_dir.clone();
        path.push("collections.json");
        let content = serde_json::to_string_pretty(collections)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_collections(&self) -> Result<Vec<Collection>, Box<dyn std::error::Error>> {
        let mut path = self.base_dir.clone();
        path.push("collections.json");
        if !path.exists() {
            let defaults = Self::get_default_collections();
            let _ = self.save_collections(&defaults);
            return Ok(defaults);
        }
        let content = fs::read_to_string(path)?;
        let mut collections: Vec<Collection> = serde_json::from_str(&content)?;
        if collections.is_empty() {
            let defaults = Self::get_default_collections();
            let _ = self.save_collections(&defaults);
            return Ok(defaults);
        }

        Self::sync_sample_scripts(&mut collections);
        Ok(collections)
    }

    fn sync_sample_scripts(collections: &mut [Collection]) {
        let defaults = Self::get_default_collections();
        for col in collections.iter_mut() {
            if let Some(def_col) = defaults.iter().find(|d| d.name == col.name) {
                Self::merge_items_scripts(&mut col.items, &def_col.items);
            }
        }
    }

    fn merge_items_scripts(
        target_items: &mut [crate::core::collection::CollectionItem],
        source_items: &[crate::core::collection::CollectionItem],
    ) {
        use crate::core::collection::CollectionItem;
        for target in target_items.iter_mut() {
            match target {
                CollectionItem::Folder(t_folder) => {
                    if let Some(CollectionItem::Folder(s_folder)) =
                        source_items.iter().find(|s| s.name() == t_folder.name)
                    {
                        Self::merge_items_scripts(&mut t_folder.items, &s_folder.items);
                    }
                }
                CollectionItem::Request(t_req) => {
                    if let Some(CollectionItem::Request(s_req)) =
                        source_items.iter().find(|s| s.name() == t_req.name)
                    {
                        if t_req.pre_request_script.is_none() {
                            t_req.pre_request_script = s_req.pre_request_script.clone();
                        }
                        if t_req.post_response_script.is_none() {
                            t_req.post_response_script = s_req.post_response_script.clone();
                        }
                    }
                }
            }
        }
    }

    pub fn get_default_collections() -> Vec<Collection> {
        let mut collections = Vec::new();
        let httpbin_json = include_str!("../samples/httpbin.json");
        let petstore_json = include_str!("../samples/petstore.json");

        if let Ok(col) = crate::core::import::postman::import_postman_collection(httpbin_json) {
            collections.push(col);
        }
        if let Ok(col) = crate::core::import::postman::import_postman_collection(petstore_json) {
            collections.push(col);
        }
        collections
    }
}
