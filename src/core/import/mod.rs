pub mod postman;

pub fn import_collection<P: AsRef<std::path::Path>>(
    path: P,
) -> anyhow::Result<crate::core::collection::Collection> {
    postman::import_postman(path)
}
