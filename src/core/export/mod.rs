pub mod openapi;
pub mod postman;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Postman,
    OpenApi,
}

pub fn export_collection(
    collection: &crate::core::collection::Collection,
    format: ExportFormat,
) -> anyhow::Result<String> {
    match format {
        ExportFormat::Postman => postman::export_postman(collection),
        ExportFormat::OpenApi => openapi::export_openapi(collection),
    }
}
