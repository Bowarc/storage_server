#[cfg(not(test))]
lazy_static! {
    static ref CACHE_DIRECTORY: std::path::PathBuf =
        std::str::FromStr::from_str("./cache").unwrap();
}
#[cfg(test)]
lazy_static! {
    static ref CACHE_DIRECTORY: std::path::PathBuf =
        std::str::FromStr::from_str("../cache").unwrap();
}

pub fn meta_path(uuid: &uuid::Uuid) -> std::path::PathBuf {
    CACHE_DIRECTORY.join(format!("{}.meta", uuid.as_hyphenated()))
}

pub fn data_path(name: &str) -> std::path::PathBuf {
    CACHE_DIRECTORY.join(name)
}

pub fn temp_data_path(uuid: &uuid::Uuid) -> std::path::PathBuf {
    CACHE_DIRECTORY.join(format!("{}.temp_data", uuid.as_hyphenated()))
}

pub fn duplicates_path() -> std::path::PathBuf {
    CACHE_DIRECTORY.join("duplicates.json")
}

pub fn read_cache_dir() -> Result<std::fs::ReadDir, crate::error::CacheError> {
    std::fs::read_dir(CACHE_DIRECTORY.clone()).map_err(|e| {
        error!("Could not open cache dir due to: {e}");
        crate::error::CacheError::CacheDirRead {
            dir: CACHE_DIRECTORY.display().to_string(),
            why: e,
        }
    })
}

// This tries to create the .meta and .data files
// If it fails to create one of the two, it deletes the one created
pub fn create_cache_files(
    meta_path: std::path::PathBuf,
    data_path: std::path::PathBuf,
) -> Result<(std::fs::File, std::fs::File), crate::error::CacheError> {
    use {
        crate::error::CacheError,
        std::fs::{remove_file, OpenOptions},
    };

    let meta_file = OpenOptions::new()
        .create_new(true) // If they already exist, this fails
        .write(true)
        .open(&meta_path)
        .map_err(|e| CacheError::FileCreate {
            file: meta_path.display().to_string(),
            why: e,
        })?;

    let data_file = OpenOptions::new()
        .create_new(true) // If they already exist, this fails
        .write(true)
        .open(&data_path)
        .map_err(|e| {
            if let Err(remove_e) = remove_file(&meta_path) {
                return CacheError::FileRemove {
                    file: meta_path.display().to_string(),
                    why: remove_e,
                };
            }

            CacheError::FileCreate {
                file: data_path.display().to_string(),
                why: e,
            }
        })?;

    Ok((meta_file, data_file))
}
