#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Test error")]
    Test,

    #[error("Could not create file: {0}")]
    FileCreate(String),

    #[error("Could not write to file: {0}")]
    FileWrite(String),

    #[error("Could not open file: {0}")]
    FileOpen(String),

    #[error("Could not read file: {0}")]
    FileRead(String),

    #[error("Could not compress the given data")]
    Compression,

    #[error("Could not decompress the given data")]
    Decompression,

    #[error("The given id doesn't correspond to any saved cache")]
    NotFound,

    #[error("Could not serialize")]
    Serialization,

    #[error("Could not deserialize")]
    Deserialization,
}

#[derive(Debug, thiserror::Error)]
pub enum UuidParseError {
    #[error("Failled the regex check")]
    Regex,
    #[error("Failled the regex check")]
    Convert,
}
