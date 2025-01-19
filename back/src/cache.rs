use std::io::Write;
use std::sync::Arc;

use data::CacheEntry;
use rocket::data::{ByteUnit, DataStream, ToByteUnit};
use tokio_util::compat::FuturesAsyncWriteCompatExt;
use zstd::DEFAULT_COMPRESSION_LEVEL;

use crate::error::CacheError;

pub mod data;

#[cfg(not(test))]
const CACHE_DIRECTORY: &str = "./cache";
#[cfg(test)]
const CACHE_DIRECTORY: &str = "../cache"; // For some reason, tests launch path is ./back
const COMPRESSION_LEVEL: i32 = DEFAULT_COMPRESSION_LEVEL; // 3, 1..=22 (zstd)

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

    pub fn new_entry(
        &mut self,
        uuid: uuid::Uuid,
        upload_info: data::UploadInfo,
    ) -> Arc<CacheEntry> {
        let entry = Arc::new(CacheEntry::new(uuid, upload_info));
        self.inner.push(entry.clone());

        entry
    }

    pub async fn store<'r>(
        entry: Arc<CacheEntry>,
        data_stream: DataStream<'r>,
    ) -> Result<(), crate::error::CacheError> {
        assert!(!entry.is_ready());

        let id = entry.uuid().hyphenated().to_string();

        let (res, exec_time) = time::timeit_async(|| store(entry, data_stream)).await;

        let compressed_data_length = res?;

        debug!(
            "[{id}] Cache was successfully compresed ({} -> {}) in {}",
            ByteUnit::Byte(0),
            ByteUnit::Byte(compressed_data_length),
            time::format(exec_time, 2)
        );
        Ok(())
    }
    // Write a cache
    // pub async fn store<'r>(
    //     &mut self,
    //     entry: Arc<CacheEntry>,
    //     data_stream: DataStream<'r>,
    // ) -> Result<(), crate::error::CacheError> {
    //     use {
    //         data::CacheEntry,
    //         rocket::{data::ByteUnit, tokio},
    //         std::sync::Arc,
    //     };

    //     // assert!(entry)

    //     // Compress and store the given cache entry

    //     // tokio::spawn(async move {
    //     // let original_data_length = data.len() as u64;
    //     let (res, exec_time) = time::timeit_async(|| store(entry, data_stream)).await;

    //     let compressed_data_length = res?;

    //     let id = uuid.hyphenated().to_string();

    //     debug!(
    //         "[{id}] Cache was successfully compresed ({} -> {}) in {}",
    //         ByteUnit::Byte(0),
    //         ByteUnit::Byte(compressed_data_length),
    //         time::format(exec_time, 2)
    //     );

    //     Ok(())
    //     // })
    // }

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

    pub async fn load_stream(
        &self,
        uuid: uuid::Uuid,
    ) -> Result<(data::UploadInfo, Box<dyn std::io::Read + Send>), crate::error::CacheError> {
        use crate::error::CacheError;
        // Load and decompress the given cache entry

        let entry = self
            .inner
            .iter()
            .find(|e| e.uuid() == uuid)
            .ok_or(CacheError::NotFound)?;

        if !entry.is_ready() {
            return Err(CacheError::NotReady);
        }

        let id = uuid.hyphenated().to_string();

        let file_path = format!("{CACHE_DIRECTORY}/{id}.data");

        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(&file_path)
            .map_err(|_| CacheError::FileOpen(file_path))?;

        // {
        //     use std::io::Read as _;

        //     let mut output = Vec::new();
        //     let mut decoder = zstd::stream::Decoder::new(file).unwrap();

        //     let mut buffer = [0; 100_000];

        //     let mut total_read = 0;
        //     let mut total_write = 0;
        //     loop {
        //         let read = decoder.read(&mut buffer).unwrap();

        //         if read == 0 {
        //             println!("Decoding EOF");
        //             break;
        //         }
        //         total_read += read;

        //         total_write += output.write(&buffer[..read]).unwrap();
        //     }
        //     println!("total_read: {total_read}\ntotal_write: {total_write}");
        // }
        // panic!();
        let decoder =
            zstd::stream::Decoder::new(file).map_err(|e| CacheError::FileRead(e.to_string()))?;

        Ok((entry.upload_info().clone(), Box::new(decoder)))
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
            error!("Could not transform id '{id}' to a usable uuid due to: {e}");
            CacheError::InvalidId(id.to_string())
        })?,
        metadata,
        true,
    )))
}

// This tries to create the .meta and .data files
// If it fails to create one of the two, it deletes the one created
pub fn sync_cache_files(
    meta_path: String,
    data_path: String,
) -> Result<(std::fs::File, std::fs::File), CacheError> {
    use std::fs::{remove_file, OpenOptions};

    let meta_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&meta_path)
        .map_err(|e| {
            CacheError::FileCreate(format!(
                "Failed to create meta file with path: '{meta_path}' due to: {e}"
            ))
        })?;

    let data_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&data_path).map_err(|e|{
            if let Err(remove_e) = remove_file(&meta_path){
                return CacheError::FileOpen(format!("Failed to delete meta file with path '{meta_path}' due to: {remove_e} while cleaning after a fail to create data file with path '{data_path}' due to: {e}"));
            }

             CacheError::FileCreate(format!(
                "Failed to create data file with path: '{data_path}' due to: {e}"
            ))
        })?;

    Ok((meta_file, data_file))
}

async fn store_data_sync<'r>(
    id: &str,
    entry: std::sync::Arc<data::CacheEntry>,
    mut original_data: DataStream<'r>,
    data_file: &mut std::fs::File,
) -> Result<(), crate::error::CacheError> {
    debug!("Creating the encoder");
    let mut encoder = zstd::stream::Encoder::new(data_file, COMPRESSION_LEVEL).unwrap();

    debug!("Streaming . . .");
    let mut total_read = 0;
    let mut total_wrote = 0;

    // I am not too happy with that allocation, but since we're in an async context, we don't have much choice
    // The other way could be to make it on the stack, but if we do that, we would be very limited in size
    // since tokio's threads don't have a big stack size
    // const BUFFER_SIZE: usize = 100_000; // 100kb
    const BUFFER_SIZE: usize = 500_000; // 500kb
    #[rustfmt::skip]
    // const BUFFER_SIZE: usize = 5_000_000; // 5mb
    // const BUFFER_SIZE: usize = 500_000_000; // 500mb
    // const BUFFER_SIZE: usize = 5_000_000_000; // 5gb
    let mut b = vec![0; BUFFER_SIZE]; // 500kb, heap

    let mut i = 0;
    loop {
        use rocket::tokio::io::AsyncReadExt;
        let read = original_data.read(&mut b).await.unwrap();

        if read == 0 {
            info!("EOF");
            break;
        }
        i += 1;

        encoder.write_all(&b[..read]).unwrap();

        total_read += read;
        total_wrote += read;

        if total_read > unsafe { crate::FILE_REQ_SIZE_LIMIT.bytes() } {
            error!("Max size reached");
            panic!()
        }

        // debug!(
        //     "\nRead: {}\nWrote: {}",
        //     ByteUnit::Byte(total_read as u64),
        //     ByteUnit::Byte(total_wrote as u64)
        // );
    }
    debug!("{i} loops");

    // rocket::tokio::io::copy(&mut original_data, &mut futures::io::AllowStdIo::new(encoder.auto_finish()).compat_write()).await.unwrap();
    // std::io::copy(&mut encoder, &mut dataf);

    // TODO: Redo this compression error variant to allow the use of the actual error, or string idc
    let data_file = encoder.finish().map_err(|e| CacheError::Compression)?;

    let metadata = data_file.metadata().map_err(|e| {
        CacheError::FileRead(format!(
            "Could not read the metadata of data file '{id}' due to: {e}"
        ))
    })?;

    let file_size = metadata.len();

    debug!("File size: {file_size}");

    debug!(
        "totals:\nRead: {}\nWrote: {}\nRatio: {:.3}",
        total_read,
        total_wrote,
        total_wrote as f64 / total_read as f64
    );
    // debug!("Header size: {}", file_size - total_wrote as u64);

    // Don't forget to update the cache entry
    // entry.set_data_size(compressed_data_length as u64);

    // if let Err(e) = data_file.write_all(&compressed_data_buffer).await {
    //     error!("[{id}] Error while writing data file {e}");
    //     return Err(CacheError::FileWrite(format!("data - {e}")));
    // }
    // entry.set_data_size(69);
    Ok(())
}

async fn store_meta(
    id: &str,
    entry: std::sync::Arc<data::CacheEntry>,
    meta_file: &mut std::fs::File,
) -> Result<(), crate::error::CacheError> {
    use {crate::error::CacheError, rocket::serde::json::serde_json};

    let metadata = entry.build_metadata();

    let meta = serde_json::to_string_pretty(&metadata).map_err(|e| {
        error!("[{id}] Could not create meta json object due to: {e}");
        CacheError::Serialization
    })?;

    // let meta = ; // Cannot inline with the above due to lifetime issues

    if let Err(e) = meta_file.write_all(meta.as_bytes()) {
        error!("Failled to write meta file: {e}");
        return Err(CacheError::FileWrite(format!("meta - {e}")));
    }

    Ok(())
}

async fn store<'r>(
    entry: std::sync::Arc<data::CacheEntry>,
    original_data_stream: DataStream<'r>,
) -> Result<u64, crate::error::CacheError> {
    let id = entry.uuid().hyphenated().to_string();

    // let (mut meta_file, mut data_file) = create_files(
    //     format!("{CACHE_DIRECTORY}/{id}.meta"),
    //     format!("{CACHE_DIRECTORY}/{id}.data"),
    // )
    // .await?;
    let (mut meta_file, mut data_file) = sync_cache_files(
        format!("{CACHE_DIRECTORY}/{id}.meta"),
        format!("{CACHE_DIRECTORY}/{id}.data"),
    )?;

    /* -------------------------------------------------------------------------------
                                Data file

        Stores a compressed version of the given data.
    -----------------------------------s-------------------------------------------- */

    store_data_sync(&id, entry.clone(), original_data_stream, &mut data_file).await?;

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
