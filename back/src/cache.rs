pub mod data;

#[cfg(not(test))]
const CACHE_DIRECTORY: &str = "./cache";
#[cfg(test)]
const CACHE_DIRECTORY: &str = "../cache"; // For some reason, tests launch path is ./back
const COMPRESSION_LEVEL: i32 = zstd::DEFAULT_COMPRESSION_LEVEL; // 3, 1..=22 (zstd)

#[derive(Default)]
pub struct Cache {
    pub inner: Vec<std::sync::Arc<data::CacheEntry>>,
    // Zip archive instead of path ?
}

impl Cache {
    pub fn new() -> Option<Self> {
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

                read_cache(id, path.clone())
                    .map_err(|e| error!("Could not load cache for id: '{id}' due to: {e}"))
                    .ok()
            })
            .collect::<Vec<Arc<data::CacheEntry>>>();

        Some(Self { inner })
    }

    pub fn new_entry(
        &mut self,
        uuid: uuid::Uuid,
        upload_info: data::UploadInfo,
    ) -> std::sync::Arc<data::CacheEntry> {
        use {data::CacheEntry, std::sync::Arc};
        let entry = Arc::new(CacheEntry::new(uuid, upload_info));
        self.inner.push(entry.clone());

        entry
    }

    pub async fn get_entry(
        &self,
        uuid: uuid::Uuid,
    ) -> Result<std::sync::Arc<data::CacheEntry>, crate::error::CacheError> {
        use crate::error::CacheError;

        Ok(self
            .inner
            .iter()
            .find(|e| e.uuid() == uuid)
            .ok_or(CacheError::NotFound { uuid })?
            // Could use .as_ref but it would require keeping the cache lock alive as look as we use the reference and i don't like that
            .clone())
    }

    pub async fn store(
        entry: std::sync::Arc<data::CacheEntry>,
        data_stream: rocket::data::DataStream<'_>,
    ) -> Result<(), crate::error::CacheError> {
        use rocket::data::ByteUnit;

        assert!(!entry.is_ready());

        let id = entry.uuid().hyphenated().to_string();

        let (res, exec_time) = time::timeit_async(|| store(entry.clone(), data_stream)).await;

        let original_data_length = res?;

        debug!(
            "[{id}] Cache was successfully compresed ({} -> {}) in {}",
            ByteUnit::Byte(original_data_length),
            ByteUnit::Byte(entry.data_size()),
            time::format(exec_time, 2)
        );
        Ok(())
    }

    // Load a stored cache
    pub fn load(
        entry: std::sync::Arc<data::CacheEntry>,
    ) -> Result<(data::UploadInfo, Box<dyn std::io::Read + Send>), crate::error::CacheError> {
        use crate::error::CacheError;
        // Load and decompress the given cache entry

        let uuid = entry.uuid();

        if !entry.is_ready() {
            return Err(CacheError::NotReady { uuid });
        }

        let id = uuid.hyphenated().to_string();

        let file_path = format!("{CACHE_DIRECTORY}/{id}.data");

        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(&file_path)
            .map_err(|e| CacheError::FileOpen {
                file: file_path.clone(),
                why: e,
            })?;

        let decoder = zstd::stream::Decoder::new(file).map_err(|e| CacheError::FileOpen {
            file: file_path,
            why: e,
        })?;

        Ok((entry.upload_info().clone(), Box::new(decoder)))
    }
}

// try read a specific cache from file
fn read_cache(
    uuid: uuid::Uuid,
    path: std::path::PathBuf,
) -> Result<std::sync::Arc<data::CacheEntry>, crate::error::CacheError> {
    use {
        crate::error::CacheError,
        data::{CacheEntry, Metadata},
        rocket::serde::json::serde_json,
        std::{fs, sync::Arc},
    };

    // let file_content: serde_json::Value = serde_json::from_str(
    //     &fs::read_to_string(path.clone())
    //         .map_err(|e| error!("Could not open cache file '{id}' due to: {e}"))
    //         .ok()?,
    // )
    // .map_err(|e| error!("Could not deserialize cache file '{id}' due to: {e}"))
    // .ok()?;

    let metadata: Metadata =
        serde_json::from_str(&fs::read_to_string(path.clone()).map_err(|e| {
            CacheError::FileRead {
                file: path.display().to_string(),
                why: e,
            }
        })?)
        .map_err(|e| CacheError::Deserialization {
            file: path.display().to_string(),
            why: e,
        })?;

    // let Some(username) = file_content
    //     .get("username")
    //     .and_then(|val| val.as_str())
    //     .and_then(|s| Some(s.to_string()))
    // else {
    //     warn!("Could not get the username property of cache file '{id}'");
    //     return None;
    // };

    // let Some(file_ext) = file_content
    //     .get("extension")
    //     .and_then(|val| val.as_str())
    //     .and_then(|s| Some(s.to_string()))
    // else {
    //     warn!("Could not get the extension property of cache file '{id}'");
    //     return None;
    // };

    // let Some(file_name) = file_content
    //     .get("name")
    //     .and_then(|val| val.as_str())
    //     .and_then(|s| Some(s.to_string()))
    // else {
    //     warn!("Could not get the name property of cache file '{id}'");
    //     return None;
    // };

    // let Some(data_size) = file_content
    //     .get("data size")
    //     .and_then(|val| val.as_number())
    //     .and_then(|n| n.as_u64())
    //     .and_then(|n| Some(n as usize))
    // else {
    //     warn!("Could not get the data size property of cache file '{id}'");
    //     return None;
    // };

    // Some(Arc::new(CacheEntry {
    //     uuid: Uuid::from_str(id)
    //         .map_err(|e| format!("Could not transform id '{id}' to a usable uuid due to: {e}"))
    //         .ok()?,
    //     metadata: Metadata {
    //         username,
    //         file_name,
    //         file_ext,
    //     },
    //     is_ready: AtomicBool::new(true),
    //     data_size: AtomicUsize::new(data_size),
    // }))

    Ok(Arc::new(CacheEntry::from_metadata(uuid, metadata, true)))
}

// This tries to create the .meta and .data files
// If it fails to create one of the two, it deletes the one created
pub fn create_cache_files(
    meta_path: String,
    data_path: String,
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
            file: meta_path.clone(),
            why: e,
        })?;

    let data_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&data_path)
        .map_err(|e| {
            if let Err(remove_e) = remove_file(&meta_path) {
                return CacheError::FileRemove {
                    file: meta_path,
                    why: remove_e,
                };
            }

            CacheError::FileCreate {
                file: data_path,
                why: e,
            }
        })?;

    Ok((meta_file, data_file))
}

async fn store_data_sync(
    uuid: &uuid::Uuid,
    entry: std::sync::Arc<data::CacheEntry>,
    mut original_data: rocket::data::DataStream<'_>,
    data_file: &mut std::fs::File,
) -> Result<u64, crate::error::CacheError> {
    use {
        crate::error::CacheError, rocket::data::ToByteUnit as _,
        rocket::tokio::io::AsyncReadExt as _, std::io::Write as _,
    };

    let mut encoder = zstd::stream::Encoder::new(data_file, COMPRESSION_LEVEL).unwrap();

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
        100.- (file_size as f64 / total_read as f64) * 100.
    );

    entry.set_data_size(file_size);

    Ok(total_read as u64)
}

async fn store_meta(
    uuid: &uuid::Uuid,
    entry: std::sync::Arc<data::CacheEntry>,
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
    entry: std::sync::Arc<data::CacheEntry>,
    original_data_stream: rocket::data::DataStream<'_>,
) -> Result<u64, crate::error::CacheError> {
    let uuid = entry.uuid();
    let id = uuid.hyphenated().to_string();

    let (mut meta_file, mut data_file) = create_cache_files(
        format!("{CACHE_DIRECTORY}/{id}.meta"),
        format!("{CACHE_DIRECTORY}/{id}.data"),
    )?;

    /* -------------------------------------------------------------------------------
                                Data file

        Stores a compressed version of the given data.
    -----------------------------------s-------------------------------------------- */

    let original_data_length =
        store_data_sync(&uuid, entry.clone(), original_data_stream, &mut data_file).await?;

    /* -------------------------------------------------------------------------------
                                Meta file

        Has all the usefull infos about the data file.
        It's written at the end so the download method wont find partial data
        -- partially true, we mainly have a 'ready' atomic bool for this
    ------------------------------------------------------------------------------- */

    store_meta(&uuid, entry.clone(), &mut meta_file).await?;

    entry.set_ready(true);

    Ok(original_data_length)
}
