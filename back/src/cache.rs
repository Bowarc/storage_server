pub mod data;

#[cfg(not(test))]
const CACHE_DIRECTORY: &'static str = "./cache";
#[cfg(test)]
const CACHE_DIRECTORY: &'static str = "../cache"; // For some reason, tests launch path is ./back
const COMPRESSION_LEVEL: i32 = 5; // 1..=11

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
                let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
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

    // Write a cache
    pub fn store(
        &mut self,
        uuid: uuid::Uuid,
        upload_info: data::UploadInfo,
        data: Vec<u8>,
    ) -> rocket::tokio::task::JoinHandle<Result<(), crate::error::CacheError>> {
        use {
            data::CacheEntry,
            rocket::{data::ByteUnit, tokio},
            std::sync::Arc,
        };

        // Compress and store the given cache entry
        let entry = Arc::new(CacheEntry::new(uuid, upload_info));
        self.inner.push(entry.clone());

        tokio::spawn(async move {
            let original_data_length = data.len() as u64;
            let (res, exec_time) = time::timeit_async(|| store(entry, data)).await;

            let compressed_data_length = res?;

            let id = uuid.hyphenated().to_string();

            debug!(
                "[{id}] Cache was successfully compresed ({} -> {}) in {}",
                ByteUnit::Byte(original_data_length),
                ByteUnit::Byte(compressed_data_length),
                time::format(exec_time, 2)
            );

            Ok(())
        })
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
            .ok_or(CacheError::NotFound)?
            // Could use .as_ref but it would require keeping the cache lock alive as look as we use the reference and i don't like that
            .clone())
    }

    // Load a stored cache
    pub async fn load(
        &self,
        uuid: uuid::Uuid,
    ) -> Result<(data::UploadInfo, Vec<u8>), crate::error::CacheError> {
        use {
            crate::error::CacheError,
            brotli::BrotliDecompress,
            rocket::tokio::{fs, io::AsyncReadExt},
        };

        // Load and decompress the given cache entry

        let entry = self
            .inner
            .iter()
            .find(|e| e.uuid() == uuid)
            .ok_or(CacheError::NotFound)?;

        if !entry.is_ready() {
            return Err(CacheError::NotReady);
        }

        let mut data_compressed = Vec::new();

        let id = uuid.hyphenated().to_string();

        let file_path = format!("{CACHE_DIRECTORY}/{id}.data");

        if let Err(e) = fs::File::open(&file_path)
            .await
            .map_err(|_| CacheError::FileOpen(file_path))?
            .read_to_end(&mut data_compressed)
            .await
        {
            error!("[{id}] Unable to read the data file: {e}");
            return Err(CacheError::FileRead(format!("{e}")));
        }

        let mut data = Vec::new();
        if let Err(e) = BrotliDecompress(&mut std::io::Cursor::new(data_compressed), &mut data) {
            error!("[{id}] Decompression failed: {e}");
            return Err(CacheError::Decompression);
        }

        Ok((entry.upload_info().clone(), data))
    }
}

// try read a specific cache from file
fn read_cache(
    id: &str,
    path: std::path::PathBuf,
) -> Result<std::sync::Arc<data::CacheEntry>, crate::error::CacheError> {
    use {
        crate::error::CacheError,
        data::{CacheEntry, Metadata},
        rocket::serde::json::serde_json,
        std::{fs, str::FromStr as _, sync::Arc},
        uuid::Uuid,
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
            error!("Could not open cache file '{id}' due to: {e}");
            CacheError::FileRead("meta".to_string())
        })?)
        .map_err(|e| {
            error!("Could not deserialize cache file '{id}' due to: {e}");
            CacheError::Deserialization
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

    Ok(Arc::new(CacheEntry::from_metadata(
        Uuid::from_str(id).map_err(|e| {
            format!("Could not transform id '{id}' to a usable uuid due to: {e}");
            CacheError::InvalidId(id.to_string())
        })?,
        metadata,
        true,
    )))
}

// This tries to create the .meta and .data files
// If it fails to create one of the two, it deletes the one created
async fn create_files(
    meta_path: String,
    data_path: String,
) -> Result<(rocket::tokio::fs::File, rocket::tokio::fs::File), crate::error::CacheError> {
    use {
        crate::error::CacheError,
        rocket::tokio::fs::{remove_file, File},
    };

    match futures::join!(
        async {
            File::create(&meta_path)
                .await
                .map_err(|_e| CacheError::FileCreate("meta".to_string()))
        },
        async {
            File::create(&data_path)
                .await
                .map_err(|_e| CacheError::FileCreate("data".to_string()))
        }
    ) {
        (Ok(meta_file), Ok(data_file)) => Ok((meta_file, data_file)),
        (Ok(_f), Err(e)) => {
            remove_file(meta_path).await.unwrap();
            return Err(e);
        }
        (Err(e), Ok(_f)) => {
            remove_file(data_path).await.unwrap();
            return Err(e);
        }
        (Err(de), Err(me)) => {
            return Err(CacheError::FileCreate(format!("Data ({de})\nMeta ({me})")))
        }
    }
}

async fn store_data(
    id: &str,
    entry: std::sync::Arc<data::CacheEntry>,
    original_data: Vec<u8>,
    data_file: &mut rocket::tokio::fs::File,
) -> Result<(), crate::error::CacheError> {
    use {
        crate::error::CacheError,
        brotli::{enc::BrotliEncoderParams, BrotliCompress},
        rocket::tokio::io::AsyncWriteExt as _,
        std::io::Cursor,
    };

    // Some files formats like png are already compressed by default, this is useless
    let encoder_params = BrotliEncoderParams {
        quality: COMPRESSION_LEVEL,
        ..Default::default()
    };

    let mut compressed_data_buffer = Vec::new();
    let mut original_data_reader = Cursor::new(original_data);
    let compression_result = BrotliCompress(
        &mut original_data_reader,
        &mut compressed_data_buffer,
        &encoder_params,
    );

    let compressed_data_length = match compression_result {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("[{id}] Failled to compress data due to: {e}");
            return Err(CacheError::Compression);
        }
    };

    // Don't forget to update the cache entry
    entry.set_data_size(compressed_data_length as u64);

    if let Err(e) = data_file.write_all(&compressed_data_buffer).await {
        error!("[{id}] Error while writing data file {e}");
        return Err(CacheError::FileWrite(format!("data - {e}")));
    }
    Ok(())
}

async fn store_meta(
    id: &str,
    entry: std::sync::Arc<data::CacheEntry>,
    meta_file: &mut rocket::tokio::fs::File,
) -> Result<(), crate::error::CacheError> {
    use {
        crate::error::CacheError, rocket::serde::json::serde_json,
        rocket::tokio::io::AsyncWriteExt as _,
    };

    let metadata = entry.build_metadata();

    let meta = serde_json::to_string_pretty(&metadata).map_err(|e| {
        error!("[{id}] Could not create meta json object due to: {e}");
        CacheError::Serialization
    })?;

    // let meta = ; // Cannot inline with the above due to lifetime issues

    if let Err(e) = meta_file.write_all(meta.as_bytes()).await {
        error!("Failled to write meta file: {e}");
        return Err(CacheError::FileWrite(format!("meta - {e}")));
    }

    Ok(())
}

async fn store(
    entry: std::sync::Arc<data::CacheEntry>,
    original_data: Vec<u8>,
) -> Result<u64, crate::error::CacheError> {
    let id = entry.uuid().hyphenated().to_string();

    let (mut meta_file, mut data_file) = create_files(
        format!("{CACHE_DIRECTORY}/{id}.meta"),
        format!("{CACHE_DIRECTORY}/{id}.data"),
    )
    .await?;

    /* -------------------------------------------------------------------------------
                                Data file

        Stores a compressed version of the given data.
    -----------------------------------s-------------------------------------------- */

    store_data(&id, entry.clone(), original_data, &mut data_file).await?;

    /* -------------------------------------------------------------------------------
                                Meta file

        Has all the usefull infos about the data file.
        It's written at the end so the download method wont find partial data
        -- partially true, we mainly have a 'ready' atomic bool for this
    ------------------------------------------------------------------------------- */

    store_meta(&id, entry.clone(), &mut meta_file).await?;

    entry.set_ready(true);

    Ok(entry.data_size())
}
