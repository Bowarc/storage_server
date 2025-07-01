mod duplicates;
mod entry;
mod metadata;
mod size;
mod upload_info;

pub use duplicates::DuplicateMap;
pub use entry::CacheEntry;
pub use metadata::Metadata;
pub use size::Size;
pub use upload_info::UploadInfo;

// const CACHE_DIRECTORY: &std::path::PathBuf = "./cache";
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

const COMPRESSION_LEVEL: i32 = zstd::DEFAULT_COMPRESSION_LEVEL; // 3, 1..=22 (zstd)

pub type CacheEntryList = Vec<std::sync::Arc<CacheEntry>>;

pub fn init_cache_list_from_cache_dir() -> Option<CacheEntryList> {
    use {
        std::{fs::read_dir, path::PathBuf, str::FromStr as _, sync::Arc},
        uuid::Uuid,
    };

    let files = read_dir(CACHE_DIRECTORY.clone())
        .map_err(|e| error!("Could not open cache dir due to: {e}"))
        .ok()?;

    // The default one is bad
    let display_path = |path: PathBuf| -> String { path.display().to_string().replace("\\", "/") };

    let inner = files
        .flatten()
        .flat_map(|entry| {
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
                .and_then(|s| Uuid::from_str(s).ok())
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

    debug!("Loaded {} cache entries", inner.len());

    Some(inner)
}

fn meta_path(uuid: &uuid::Uuid) -> std::path::PathBuf {
    let mut p = CACHE_DIRECTORY.clone();
    p.push(format!("{}.meta", uuid.as_hyphenated()));
    p
}

fn temp_data_path(uuid: &uuid::Uuid) -> std::path::PathBuf {
    let mut p = CACHE_DIRECTORY.clone();
    p.push(format!("{}.temp_data", uuid.as_hyphenated()));
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

// Returns the file size before compression
async fn store_data(
    uuid: &uuid::Uuid,
    mut original_data: rocket::data::DataStream<'_>,
    data_file: &mut std::fs::File,
) -> Result<Size, crate::error::CacheError> {
    use {
        crate::error::CacheError,
        rocket::data::{ByteUnit, ToByteUnit as _},
        rocket::tokio::io::AsyncReadExt as _,
        std::io::Write as _,
        zstd::stream::Encoder,
    };

    let mut encoder = Encoder::new(data_file, COMPRESSION_LEVEL).unwrap();

    // I am not too happy with that allocation, but since we're in an async context
    // (and i don't really know how async task works, so idk if thread local is usable here), we don't have much choice
    // The other way could be to make it on the stack, but if we do that, we would be very limited in size
    // since tokio's threads don't have a big stack size
    const BUFFER_SIZE: usize = 500_000; // 500kb
    #[rustfmt::skip]
    let mut buffer = vec![0; BUFFER_SIZE];

    let byte_size_limit: ByteUnit = unsafe { crate::FILE_REQ_SIZE_LIMIT.bytes() };

    let mut total_read = 0;
    loop {
        let read = original_data.read(&mut buffer).await.unwrap();

        if read == 0 {
            break;
        }

        total_read += read;

        encoder.write_all(&buffer[..read]).unwrap();

        if total_read > byte_size_limit {
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
        "totals:\nRead: {}\nWrote: {}\nDelta: {}%",
        total_read,
        file_size,
        {
            let d = -(100. - (file_size as f64 / total_read as f64) * 100.);
            if d > 0. {
                format!("+{d:.3}")
            } else {
                format!("{d:.3}")
            }
        }
    );

    // Ok(total_read as u64)
    Ok(Size::new(total_read as u64, file_size))
}

async fn store_meta(
    uuid: &uuid::Uuid,
    metadata: Metadata,
    meta_file: &mut std::fs::File,
) -> Result<(), crate::error::CacheError> {
    use {crate::error::CacheError, rocket::serde::json::serde_json, std::io::Write as _};

    let meta = serde_json::to_string(&metadata).map_err(|e| CacheError::Serialization {
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

/// This function moves or remove the data file depending on if it already exists
// Can't do it in store_data since we only have access to the original bytes
// which is pre compression, so much more than if we read after
async fn handle_duplicates(
    data_file_path: &mut std::path::PathBuf,
    uuid: &uuid::Uuid,
    duplicate_map: &std::sync::Arc<rocket::tokio::sync::Mutex<DuplicateMap>>,
) {
    use {
        rocket::tokio::{
            fs::{remove_file, rename},
            sync::Mutex,
        },
        std::sync::LazyLock,
    };
    let (r, duration) = time::timeit_async(async || hash_file(data_file_path.clone()).await).await;
    debug!("Hash: {:?} in {}", r, time::format(duration, -1));
    let hash = r.unwrap();

    // Quick and dirty way to avoid all possibilities of data races on file moves
    static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

    let guard = MUTEX.lock().await;

    let is_duplicate = {
        let mut duplicate_map_handle = duplicate_map.lock().await;

        duplicate_map_handle.add(hash.clone(), *uuid).unwrap();
        duplicate_map_handle.get(&hash).unwrap().len() > 1
    };

    let new_dp = {
        let mut p = CACHE_DIRECTORY.clone();
        p.push(hash.clone());
        p
    };

    if is_duplicate {
        remove_file(data_file_path.clone()).await.unwrap();
    } else {
        rename(data_file_path.clone(), new_dp.clone())
            .await
            .unwrap();
    }

    *data_file_path = new_dp;

    drop(guard);
}

async fn hash_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<String> {
    use {
        rocket::tokio::{fs::File, io::AsyncReadExt as _},
        sha2::{Digest as _, Sha256},
    };

    let mut file = File::open(path).await?;
    let mut hasher = Sha256::default();
    let mut buffer = [0; 8192];

    while let Ok(bytes_read) = file.read(&mut buffer).await {
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

async fn store(
    entry: &mut CacheEntry,
    original_data_stream: rocket::data::DataStream<'_>,
    duplicate_map: std::sync::Arc<rocket::tokio::sync::Mutex<DuplicateMap>>,
) -> Result<Size, crate::error::CacheError> {
    use tokio::fs::remove_file;

    let uuid = entry.uuid();

    let meta_path = meta_path(&uuid);
    let mut data_path = temp_data_path(&uuid);

    let (mut meta_file, mut data_file) = create_cache_files(meta_path.clone(), data_path.clone())?;

    let (data_store_result, data_store_duration) =
        time::timeit_async(async || store_data(&uuid, original_data_stream, &mut data_file).await)
            .await;

    debug!("Data store took: {}", time::format(data_store_duration, -1));

    let data_size = match data_store_result {
        Ok(size) => size,
        Err(e) => {
            // We know that the files were created, so any error here are important
            if let Err(e) = remove_file(data_path).await {
                error!("[{uuid}] Failed to cleanup data file after error due to: {e}");
            }
            return Err(e);
        }
    };

    entry.set_data_size(data_size);

    handle_duplicates(&mut data_path, &uuid, &duplicate_map).await;

    let metadata = Metadata::new(
        entry.upload_info().name().to_string(),
        entry.upload_info().extension().to_string(),
        *entry.data_size(),
        data_path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{uuid}.data")),
    );

    if let Err(e) = store_meta(&uuid, metadata, &mut meta_file).await {
        match futures::join!(remove_file(meta_path), remove_file(data_path)) {
            (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
                error!("[{uuid}] Failed to cleanup after error due to: {e}")
            }
            (Err(e1), Err(e2)) => {
                error!("[{uuid}] Failed to cleanup after error due to: {e1} AND {e2}")
            }
            _ => (),
        }
        return Err(e);
    }

    entry.set_ready(true);

    Ok(data_size)
}
