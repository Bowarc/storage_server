mod entry;
mod metadata;
mod upload_info;

pub use entry::*;
pub use metadata::*;
pub use upload_info::*;

#[cfg(not(test))]
const CACHE_DIRECTORY: &str = "./cache";
#[cfg(test)]
const CACHE_DIRECTORY: &str = "../cache"; // For some reason, tests launch path is ./back
const COMPRESSION_LEVEL: i32 = zstd::DEFAULT_COMPRESSION_LEVEL; // 3, 1..=22 (zstd)

pub type CacheEntryList = Vec<std::sync::Arc<CacheEntry>>;

pub fn init_cache_list_from_cache_dir() -> Option<CacheEntryList> {
    use std::sync::Arc;

    let files = std::fs::read_dir(CACHE_DIRECTORY)
        .map_err(|e| error!("Could not open cache dir due to: {e}"))
        .ok()?;

    // The default one is bad
    let display_path =
        |path: std::path::PathBuf| -> String { path.display().to_string().replace("\\", "/") };

    let inner = files
        .flatten()
        .flat_map(|entry| {
            use std::str::FromStr as _;

            let metadata = entry
                .metadata()
                .map_err(|e| {
                    error!(
                        "Could not read metadata from cache file '{p}' due to: {e}",
                        p = display_path(entry.path())
                    )
                })
                .ok()?;

            if !metadata.is_file() {
                warn!(
                    "Cache loading skipping '{p}' as it's not a file",
                    p = display_path(entry.path())
                );
                return None;
            }
            let path = entry.path();
            let Some(id) = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|s| uuid::Uuid::from_str(s).ok())
            else {
                warn!(
                    "Could not extract id from cache file '{}'",
                    display_path(path)
                );
                return None;
            };
            let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
                warn!(
                    "Could not extract extension from cache file '{}'",
                    display_path(path)
                );
                return None;
            };

            if ext != "meta" {
                // Not a meta file, don't care
                return None;
            }

            CacheEntry::from_file(path)
                .map_err(|e| error!("Could not load cache for id: '{id}' due to: {e}"))
                .map(Arc::new)
                .ok()
        })
        .collect::<Vec<Arc<CacheEntry>>>();

    Some(inner)
}

fn meta_path(uuid: &uuid::Uuid) -> std::path::PathBuf {
    use std::{path::PathBuf, str::FromStr};

    let mut p = PathBuf::from_str(CACHE_DIRECTORY).unwrap();
    p.push(format!("{}.meta", uuid.as_hyphenated()));
    p
}

fn data_path(uuid: &uuid::Uuid) -> std::path::PathBuf {
    use std::{path::PathBuf, str::FromStr};

    let mut p = PathBuf::from_str(CACHE_DIRECTORY).unwrap();
    p.push(format!("{}.data", uuid.as_hyphenated()));
    p
}

// This tries to create the .meta and .data files
// If it fails to create one of the two, it deletes the one created
fn create_cache_files(
    meta_path: std::path::PathBuf,
    data_path: std::path::PathBuf,
) -> Result<(std::fs::File, std::fs::File), crate::error::CacheError> {
    use {
        crate::error::CacheError,
        std::fs::{remove_file, OpenOptions},
    };

    let meta_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&meta_path)
        .map_err(|e| CacheError::FileCreate {
            file: meta_path.display().to_string(),
            why: e,
        })?;

    let data_file = OpenOptions::new()
        .create_new(true)
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

async fn store_data_sync(
    uuid: &uuid::Uuid,
    entry: std::sync::Arc<CacheEntry>,
    mut original_data: rocket::data::DataStream<'_>,
    data_file: &mut std::fs::File,
) -> Result<u64, crate::error::CacheError> {
    use {
        crate::error::CacheError, rocket::data::ToByteUnit as _,
        rocket::tokio::io::AsyncReadExt as _, std::io::Write as _, zstd::stream::Encoder,
    };

    let mut encoder = Encoder::new(data_file, COMPRESSION_LEVEL).unwrap();

    // I am not too happy with that allocation, but since we're in an async context
    // (and i don't really know how async task works, so idk if thread local is usable here), we don't have much choice
    // The other way could be to make it on the stack, but if we do that, we would be very limited in size
    // since tokio's threads don't have a big stack size
    // const BUFFER_SIZE: usize = 100_000; // 100kb
    const BUFFER_SIZE: usize = 500_000; // 500kb
    #[rustfmt::skip]
    // const BUFFER_SIZE: usize = 5_000_000; // 5mb
    // const BUFFER_SIZE: usize = 500_000_000; // 500mb
    // const BUFFER_SIZE: usize = 5_000_000_000; // 5gb
    let mut buffer = vec![0; BUFFER_SIZE];

    let mut total_read = 0;
    loop {
        let read = original_data.read(&mut buffer).await.unwrap();

        if read == 0 {
            break;
        }
        total_read += read;

        encoder.write_all(&buffer[..read]).unwrap();

        if total_read > unsafe { crate::FILE_REQ_SIZE_LIMIT.bytes() } {
            error!("Max size reached");
            return Err(CacheError::FileSizeExceeded);
        }
    }

    let data_file = encoder
        .finish()
        .map_err(|e| CacheError::Compression { why: e })?;

    let metadata = data_file.metadata().map_err(|e| CacheError::FileRead {
        file: format!("(data file for uuid ({uuid})"),
        why: e,
    })?;

    let file_size = metadata.len();

    debug!(
        "totals:\nRead: {}\nWrote: {}\nRemoved: {:.3}%",
        total_read,
        file_size,
        100. - (file_size as f64 / total_read as f64) * 100.
    );

    entry.set_data_size(file_size);

    Ok(total_read as u64)
}

async fn store_meta(
    uuid: &uuid::Uuid,
    entry: std::sync::Arc<CacheEntry>,
    meta_file: &mut std::fs::File,
) -> Result<(), crate::error::CacheError> {
    use {crate::error::CacheError, rocket::serde::json::serde_json, std::io::Write as _};

    let metadata = entry.build_metadata();

    let meta = serde_json::to_string_pretty(&metadata).map_err(|e| CacheError::Serialization {
        context: String::from("writing meta data"),
        why: e,
    })?;

    // let meta = ; // Cannot inline with the above due to lifetime issues

    if let Err(e) = meta_file.write_all(meta.as_bytes()) {
        return Err(CacheError::FileWrite {
            file: format!("(meta file for uuid ({uuid}))"),
            why: e,
        });
    }

    Ok(())
}

async fn store(
    entry: std::sync::Arc<CacheEntry>,
    original_data_stream: rocket::data::DataStream<'_>,
) -> Result<u64, crate::error::CacheError> {
    use tokio::fs::remove_file;

    let uuid = entry.uuid();

    let (mut meta_file, mut data_file) = create_cache_files(meta_path(&uuid), data_path(&uuid))?;

    async fn cleanup(uuid: &uuid::Uuid) {
        match futures::join!(remove_file(meta_path(uuid)), remove_file(data_path(uuid))) {
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
                error!("[{uuid}] Failed to cleanup after error due to: {e}")
            }
            (Err(e1), Err(e2)) => {
                error!("[{uuid}] Failed to cleanup after error due to: {e1} AND {e2}")
            }
            _ => (),
        }
    }

    let original_data_length = match futures::join!(
        store_data_sync(&uuid, entry.clone(), original_data_stream, &mut data_file),
        store_meta(&uuid, entry.clone(), &mut meta_file)
    ) {
        (Ok(length), Ok(())) => length,
        (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
            cleanup(&uuid).await;
            return Err(e);
        }
        (Err(e1), Err(e2)) => {
            cleanup(&uuid).await;
            return Err(crate::error::CacheError::Multiple(vec![e1, e2]));
        }
    };

    entry.set_ready(true);

    Ok(original_data_length)
}
