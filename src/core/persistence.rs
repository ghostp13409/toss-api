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
            return Ok(Self::get_default_collections());
        }
        let content = fs::read_to_string(path)?;
        let collections: Vec<Collection> = serde_json::from_str(&content)?;
        if collections.is_empty() {
            return Ok(Self::get_default_collections());
        }
        Ok(collections)
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
