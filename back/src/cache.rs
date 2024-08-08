use {
    crate::error::CacheError,
    rocket::{
        serde::json::serde_json::{self, json},
        tokio,
    },
    shared::data::{CacheEntry, Metadata},
    std::sync::{atomic::Ordering, Arc},
};
#[cfg(not(test))]
const CACHE_DIRECTORY: &'static str = "./cache";
#[cfg(test)]
const CACHE_DIRECTORY: &'static str = "../cache"; // For some reason, tests launch path is ./back
const COMPRESSION_LEVEL: i32 = 5; // 1..=11

#[derive(Default)]
pub struct Cache {
    pub data: Vec<Arc<CacheEntry>>,
    // Zip archive instead of path ?
}

impl Cache {
    pub fn new() -> Option<Self> {
        use std::str::FromStr as _;
        let files = std::fs::read_dir(CACHE_DIRECTORY)
            .map_err(|e| error!("Could not open cache dir due to: {e}"))
            .ok()?;

        // The default one is bad
        let display_path =
            |path: std::path::PathBuf| -> String { path.display().to_string().replace("\\", "/") };

        let data = files
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

                let file_content: serde_json::Value = serde_json::from_str(
                    &std::fs::read_to_string(path.clone())
                        .map_err(|e| error!("Could not open cache file '{id}' due to: {e}"))
                        .ok()?,
                )
                .map_err(|e| error!("Could not deserialize cache file '{id}' due to: {e}"))
                .ok()?;

                let Some(username) = file_content
                    .get("username")
                    .and_then(|val| val.as_str())
                    .and_then(|s| Some(s.to_string()))
                else {
                    warn!("Could not get the username property of cache file '{id}'");
                    return None;
                };

                let Some(file_ext) = file_content
                    .get("extension")
                    .and_then(|val| val.as_str())
                    .and_then(|s| Some(s.to_string()))
                else {
                    warn!("Could not get the extension property of cache file '{id}'");
                    return None;
                };

                let Some(file_name) = file_content
                    .get("name")
                    .and_then(|val| val.as_str())
                    .and_then(|s| Some(s.to_string()))
                else {
                    warn!("Could not get the name property of cache file '{id}'");
                    return None;
                };

                let Some(data_size) = file_content
                    .get("data size")
                    .and_then(|val| val.as_number())
                    .and_then(|n| n.as_u64())
                    .and_then(|n| Some(n as usize))
                else {
                    warn!("Could not get the data size property of cache file '{id}'");
                    return None;
                };

                Some(Arc::new(CacheEntry {
                    id: uuid::Uuid::from_str(id)
                        .map_err(|e| {
                            format!("Could not transform id '{id}' to a usable uuid due to: {e}")
                        })
                        .ok()?,
                    metadata: Metadata {
                        username,
                        file_name,
                        file_ext,
                    },
                    is_ready: std::sync::atomic::AtomicBool::new(true),
                    data_size: std::sync::atomic::AtomicUsize::new(data_size),
                }))
            })
            .collect::<Vec<Arc<CacheEntry>>>();

        Some(Self { data })
    }
    pub fn store(
        &mut self,
        id: uuid::Uuid,
        meta: Metadata,
        data: Vec<u8>,
    ) -> tokio::task::JoinHandle<Result<(), CacheError>> {
        // Compress and store the given cache entry
        let entry = Arc::new(CacheEntry::new(id, meta));
        self.data.push(entry.clone());

        tokio::spawn(async move {
            let original_data_length = data.len();
            let (res, exec_time) = time::timeit_async(|| async { store(entry, data).await }).await;

            res.map(|compressed_data_length| {
                debug!(
                    "[{id}] Cache was successfully compresed ({} -> {}) in {}",
                    rocket::data::ByteUnit::Byte(original_data_length as u64),
                    rocket::data::ByteUnit::Byte(compressed_data_length as u64),
                    time::format(exec_time, 2)
                );
            })
        })
    }

    pub async fn get_meta(&self, id: uuid::Uuid) -> Result<Metadata, CacheError> {
        Ok(self
            .data
            .iter()
            .find(|e| e.id == id)
            .ok_or(CacheError::NotFound)?
            .metadata
            .clone())
    }

    pub async fn load(&self, id: uuid::Uuid) -> Result<(Metadata, Vec<u8>), CacheError> {
        use tokio::io::AsyncReadExt;
        // Load and decompress the given cache entry

        let entry = self
            .data
            .iter()
            .find(|e| e.id == id)
            .ok_or(CacheError::NotFound)?;

        let mut raw_compressed = Vec::new();

        let file_path = format!("{CACHE_DIRECTORY}/{id}.data");

        if let Err(e) = tokio::fs::File::open(&file_path)
            .await
            .map_err(|_| CacheError::FileOpen(file_path))?
            .read_to_end(&mut raw_compressed)
            .await
        {
            error!("[{id}] Unable to read the data file: {e}");
            return Err(CacheError::FileRead(format!("{e}")));
        }

        let mut raw = Vec::new();
        if let Err(e) =
            brotli::BrotliDecompress(&mut std::io::Cursor::new(raw_compressed), &mut raw)
        {
            error!("[{id}] Decompression failed: {e}");
            return Err(CacheError::Decompression);
        }

        Ok((entry.metadata.clone(), raw))
    }
}

async fn store(entry: Arc<CacheEntry>, original_data: Vec<u8>) -> Result<usize, CacheError> {
    use tokio::io::AsyncWriteExt as _;

    let id = entry.id.as_hyphenated().to_string();

    let files = futures::join!(
        async {
            Ok(
                match tokio::fs::File::create(format!("{CACHE_DIRECTORY}/{id}.data")).await {
                    Ok(f) => f,
                    Err(e) => {
                        error!("[{id}] Could not create data file: {e}");
                        return Err(CacheError::FileCreate("data".to_string()));
                    }
                },
            )
        },
        async {
            Ok(
                match tokio::fs::File::create(format!("{CACHE_DIRECTORY}/{id}.meta")).await {
                    Ok(file) => file,
                    Err(e) => {
                        error!("[{id}] Could not create meta file: {e}");
                        return Err(CacheError::FileCreate("meta".to_string()));
                    }
                },
            )
        }
    );

    // TODO: Fix this
    let (mut data_file, mut meta_file) = match files {
        (Ok(data_file), Ok(meta_file)) => (data_file, meta_file),
        (Ok(_), Err(e)) => return Err(e),
        (Err(e), Ok(_)) => return Err(e),
        (Err(de), Err(me)) => {
            return Err(CacheError::FileCreate(format!("Data ({de})\nMeta ({me})")))
        }
    };

    /* -------------------------------------------------------------------------------
                                Data file

        Stores a compressed version of the given data.
    ------------------------------------------------------------------------------- */

    // Some files formats like png are already compressed by default, this is useless
    let encoder_params = brotli::enc::BrotliEncoderParams {
        quality: COMPRESSION_LEVEL,
        ..Default::default()
    };

    let mut compressed_data_buffer = Vec::new();
    let mut original_data_reader = std::io::Cursor::new(original_data);
    let compression_result = brotli::BrotliCompress(
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
    entry
        .data_size
        .store(compressed_data_length, Ordering::Relaxed);

    if let Err(e) = data_file.write_all(&compressed_data_buffer).await {
        error!("[{id}] Error while writing data file {e}");
        return Err(CacheError::FileWrite(format!("data - {e}")));
    }

    /* -------------------------------------------------------------------------------
                                Meta file

        Has all the usefull infos about the data file.
        It's written at the end so the download method wont find partial data
        -- partially true, we mainly have a 'ready' atomic bool for this
    ------------------------------------------------------------------------------- */

    let meta = serde_json::to_string_pretty(&json!({
        "username": entry.metadata.username,
        "name": entry.metadata.file_name,
        "extension": entry.metadata.file_ext,
        "data size": compressed_data_length
    }))
    .map_err(|e| {
        error!("[{id}] Could not create meta json object due to: {e}");
        CacheError::Serialization
    })?;

    // let meta = ; // Cannot inline with the above due to lifetime issues

    if let Err(e) = meta_file.write_all(meta.as_bytes()).await {
        error!("Failled to write meta file: {e}");
        return Err(CacheError::FileWrite(format!("meta - {e}")));
    }

    entry.is_ready.store(true, Ordering::Relaxed);

    Ok(compressed_data_length)
}
