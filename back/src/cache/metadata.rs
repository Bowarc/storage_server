// Structure of a .meta file
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    name: String,
    extension: String,
    size: super::Size,
    data_file_name: String,
}

impl Metadata {
    pub fn new(name: String, extension: String, size: super::Size, data_file_name: String) -> Self {
        Self {
            name,
            extension,
            size,
            data_file_name,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn extension(&self) -> &String {
        &self.extension
    }

    pub fn size(&self) -> &super::Size {
        &self.size
    }

    pub fn data_file_name(&self) -> &String {
        &self.data_file_name
    }
}
