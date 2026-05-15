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
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(path)?;
        let collections = serde_json::from_str(&content)?;
        Ok(collections)
    }
}
