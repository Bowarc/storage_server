// The data supplied with an upload request
#[derive(Debug, serde::Serialize, Clone)]
pub struct UploadInfo {
    name: String,
    extension: String,
}

impl UploadInfo {
    pub fn new(name: String, extension: String) -> Self {
        Self { name, extension }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn extension(&self) -> &String {
        &self.extension
    }
}
