// Structure of a .meta file
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    name: String,
    extension: String,
    size: u64,
}

impl Metadata {
    pub fn new(name: String, extension: String, size: u64) -> Self {
        Self {
            name,
            extension,
            size,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn extension(&self) -> &String {
        &self.extension
    }

    pub fn size(&self) -> &u64 {
        &self.size
    }
}
