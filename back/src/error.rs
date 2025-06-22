#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    // #[error("Test error")]
    // Test,
    #[error("Could not create file '{file}' due to: {why}")]
    FileCreate { file: String, why: std::io::Error },

    #[error("Could not open file '{file}' due to: {why}")]
    FileOpen { file: String, why: std::io::Error },

    #[error("Could not read file '{file}' due to: {why}")]
    FileRead { file: String, why: std::io::Error },

    #[error("Could not write to file '{file}' due to: {why}")]
    FileWrite { file: String, why: std::io::Error },

    #[error("Could not remove to file '{file}' due to: {why}")]
    FileRemove { file: String, why: std::io::Error },

    #[error("Given file was too large, max size is: {}", unsafe{crate::FILE_REQ_SIZE_LIMIT})]
    FileSizeExceeded,

    #[error("Could not compress the given data due to {why}")]
    Compression { why: std::io::Error },

    #[error("Could not decompress the given data")]
    Decompression,

    #[error("The uuid '{uuid}' doen't correspond to any cache")]
    NotFound { uuid: uuid::Uuid },

    #[error("Cache with uuid: {uuid} isn't ready yet")]
    NotReady { uuid: uuid::Uuid },

    #[error("Serialization error while {context} due to {why}")]
    Serialization {
        context: String, // Extremely short description of what was atempted to do
        why: rocket::serde::json::serde_json::Error,
    },

    #[error("Could not deserialize file '{file}' due to: {why}")]
    Deserialization {
        file: String,
        why: rocket::serde::json::serde_json::Error,
    },

    #[error("The given string ({value}) could not be parsed into an uuid")]
    InvalidId { value: String },

    #[error("Wrong file type, expected: '{expected}' but got '{actual}'")]
    WrongFileType { expected: String, actual: String },

    #[error("Multiple errors occured: {0:?}")]
    Multiple(Vec<Self>)
}

#[derive(Debug, thiserror::Error)]
pub enum UuidParseError {
    #[error("Failled the regex check")]
    Regex,
    #[error("Could not convert given UUID")]
    Convert,
}
