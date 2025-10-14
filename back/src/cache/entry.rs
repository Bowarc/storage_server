#[derive(Debug, serde::Serialize)]
/// Having multiple CacheEntry instance of the same uuid is NOT allowed and would dismiss all
/// security
pub struct CacheEntry {
    uuid: uuid::Uuid,
    upload_info: super::UploadInfo,
    is_ready: std::sync::atomic::AtomicBool,
    size: super::Size,
}

// Getters / Setters, easier to read if they are separated
impl CacheEntry {
    // pub fn build_metadata(&self) -> super::Metadata {
    //     super::Metadata::new(
    //         self.upload_info.name().clone(),
    //         self.upload_info.extension().clone(),
    //         self.size,
    //     )
    // }
    pub fn upload_info(&self) -> &super::UploadInfo {
        &self.upload_info
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready.load(std::sync::atomic::Ordering::Acquire)
    }
    pub fn set_ready(&self, ready: bool) {
        self.is_ready
            .store(ready, std::sync::atomic::Ordering::Release);
    }

    pub fn data_size(&self) -> &super::Size {
        &self.size
    }
    pub fn set_data_size(&mut self, size: super::Size) {
        self.size = size;
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

// Init methods
impl CacheEntry {
    pub fn from_metadata(uuid: uuid::Uuid, metadata: super::Metadata, ready: bool) -> Self {
        use std::sync::atomic::AtomicBool;
        Self {
            uuid,
            upload_info: super::UploadInfo::new(
                metadata.name().clone(),
                metadata.extension().clone(),
            ),
            is_ready: AtomicBool::new(ready),
            size: *metadata.size(),
        }
    }

    pub fn from_file(path: std::path::PathBuf) -> Result<Self, crate::error::CacheError> {
        use {
            super::Metadata,
            crate::error::CacheError,
            rocket::serde::json::serde_json,
            std::fs::{File, OpenOptions},
            std::str::FromStr as _,
            uuid::Uuid,
        };

        let Some(uuid) = path
            .file_stem()
            .and_then(|os_str| os_str.to_str().map(|s| Uuid::from_str(s).ok()))
            .flatten()
        else {
            return Err(CacheError::InvalidId {
                value: format!("{:?}", path.file_name()),
            });
        };

        if path.extension().and_then(|os_str| os_str.to_str()) != Some("meta") {
            return Err(CacheError::WrongFileType {
                expected: String::from("meta"),
                actual: path
                    .extension()
                    .and_then(|os_str| os_str.to_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("Unknown file extension: {path:?}")),
            });
        }

        let file = OpenOptions::new()
            .read(true)
            .open(path.clone())
            .map_err(|e| CacheError::FileOpen {
                file: path.display().to_string(),
                why: e,
            })?;

        let metadata = serde_json::from_reader::<File, Metadata>(file).map_err(|e| {
            CacheError::Deserialization {
                file: path.display().to_string(),
                why: e,
            }
        })?;

        Ok(Self::from_metadata(uuid, metadata, true))
    }
}

impl CacheEntry {
    pub fn load_meta(&self) -> Result<super::Metadata, crate::error::CacheError> {
        use {crate::error::CacheError, rocket::serde::json::serde_json, std::fs::OpenOptions};

        let meta_path = super::fs::meta_path(&self.uuid);
        let meta_file = OpenOptions::new()
            .read(true)
            .open(&meta_path)
            .map_err(|e| CacheError::FileOpen {
                file: meta_path.display().to_string(),
                why: e,
            })?;

        serde_json::from_reader(meta_file).map_err(|e| CacheError::Deserialization {
            file: meta_path.display().to_string(),
            why: e,
        })
    }

    pub async fn store_new(
        uuid: uuid::Uuid,
        upload_info: super::UploadInfo,
        data_stream: rocket::data::DataStream<'_>,
        duplicate_map: std::sync::Arc<rocket::tokio::sync::Mutex<super::DuplicateMap>>,
    ) -> Result<Self, crate::error::CacheError> {
        use {
            crate::error::CacheError,
            rocket::{data::ByteUnit, serde::json::serde_json, tokio::fs::remove_file},
            std::sync::atomic::AtomicBool,
        };

        let start_time = std::time::Instant::now();

        let meta_path = super::fs::meta_path(&uuid);
        // Data path needs to be mutable since since it may be swapped for an already exising file
        // in the duplicate detection
        let mut data_path = super::fs::temp_data_path(&uuid);

        // Create files
        let (meta_file, mut data_file) =
            super::fs::create_cache_files(meta_path.clone(), data_path.clone())?;

        // Stream the upload to the data file, returning the original and the end file sizes
        let data_size = {
            let (data_store_result, data_store_duration) = time::timeit_async(async || {
                super::stream_to_file(&uuid, data_stream, &mut data_file).await
            })
            .await;

            debug!("Data store took: {}", time::format(data_store_duration, -1));

            match data_store_result {
                Ok(size) => size,
                Err(e) => {
                    // Cleanup the files if we encounter any error
                    // FIXME: Also remove the meta file
                    // We know that the files were created, so any error here are important
                    if let Err(e) = remove_file(data_path).await {
                        error!("[{uuid}] Failed to cleanup data file after error due to: {e}");
                    }
                    return Err(e);
                }
            }
        };

        // Make sure the file is not a duplicate, in what case we'll remove the data file we just created
        // and use the exising one
        // FIXME: Error handling
        super::duplicates::handle_duplicates(&mut data_path, &uuid, &duplicate_map)
            .await
            .unwrap();

        // Build new metadata
        let metadata = super::Metadata::new(
            upload_info.name().to_string(),
            upload_info.extension().to_string(),
            data_size,
            data_path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{uuid}.data")),
        );

        // Store that newly built metadata
        if let Err(e) = serde_json::to_writer(meta_file, &metadata) {
            // FIXME: This could be it's own closure, since it may need to be used above
            match futures::join!(remove_file(meta_path), remove_file(data_path)) {
                (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
                    error!("[{uuid}] Failed to cleanup after error due to: {e}")
                }
                (Err(e1), Err(e2)) => {
                    error!("[{uuid}] Failed to cleanup after error due to: {e1} AND {e2}")
                }
                _ => (),
            }
            return Err(CacheError::Serialization {
                context: String::from("writing meta data"),
                why: e,
            });
        }

        debug!(
            "[{uuid}] Cache was successfully compresed ({} -> {}) in {}",
            ByteUnit::Byte(data_size.original()),
            ByteUnit::Byte(data_size.compressed()),
            time::format(start_time.elapsed(), 2)
        );

        Ok(Self {
            uuid,
            upload_info,
            is_ready: AtomicBool::new(true),
            size: data_size,
        })
    }

    // Load a stored cache entry
    pub async fn load(
        &self,
    ) -> Result<(super::UploadInfo, Box<dyn std::io::Read + Send>), crate::error::CacheError> {
        use {crate::error::CacheError, std::fs::OpenOptions, zstd::stream::Decoder};

        // FIXME
        //
        // Here, I would like to add some kind of lock to make sure a file currently
        // Being read is not deleted, but I don't actually know how to do it.
        //
        // Since we give back a decoder, using some kind of lock in this method wouldn't work
        // as the decoder lives longer than the method.
        //
        // Maybe some wrapper over the decoder with a lock guard given to it ?

        // Load and decompress the given cache self
        let uuid = self.uuid();

        if !self.is_ready() {
            return Err(CacheError::NotReady { uuid });
        }

        let metadata = self.load_meta()?;

        let decoder = {
            let data_path = super::fs::data_path(metadata.data_file_name());

            let file = OpenOptions::new()
                .read(true)
                .open(&data_path)
                .map_err(|e| CacheError::FileOpen {
                    file: data_path.display().to_string(),
                    why: e,
                })?;

            Decoder::new(file).map_err(|e| CacheError::FileOpen {
                file: data_path.display().to_string(),
                why: e,
            })?
        };

        Ok((self.upload_info.clone(), Box::new(decoder)))
    }

    /// Delete a cache entry
    pub async fn delete(
        &self,
        duplicate_map: std::sync::Arc<rocket::tokio::sync::Mutex<super::DuplicateMap>>,
    ) -> Result<(), crate::error::CacheError> {
        use {crate::error::CacheError, tokio::fs::remove_file};

        self.set_ready(false);

        let metadata = self.load_meta()?;

        let mut duplicate_map_guard = duplicate_map.lock().await;
        let hashes = duplicate_map_guard.remove(&self.uuid)?;

        if hashes.len() != 1 {
            return Err(CacheError::DuplicateMapLogic(
                format!("Deletion of [{}] failled: the duplicate map returned {} matches ({:?}) for uuid: {}", self.uuid, hashes.len(), hashes, self.uuid)
            ));
        }

        let data_hash = hashes.first().unwrap(); // Cannot fail

        // Meaning that the current uuid was NOT the last holder of that data hash
        if duplicate_map_guard.get(data_hash).is_some() {
            // Just remove the meta file, and leave
            let meta_path = super::fs::meta_path(&self.uuid);
            remove_file(&meta_path)
                .await
                .map_err(|e| CacheError::FileRemove {
                    file: meta_path.display().to_string(),
                    why: e,
                })?;

            return Ok(());
        }

        let data_path = super::fs::data_path(metadata.data_file_name());

        let meta_path = super::fs::meta_path(&self.uuid);
        match futures::join!(remove_file(&meta_path), remove_file(&data_path)) {
            (Ok(_), Ok(_)) => Ok(()),
            (Ok(_), Err(e)) => Err(CacheError::FileRemove {
                file: data_path.display().to_string(),
                why: e,
            }),
            (Err(e), Ok(_)) => Err(CacheError::FileRemove {
                file: meta_path.display().to_string(),
                why: e,
            }),
            (Err(e1), Err(e2)) => Err(CacheError::Multiple(vec![
                CacheError::FileWrite {
                    file: meta_path.display().to_string(),
                    why: e1,
                },
                CacheError::FileWrite {
                    file: data_path.display().to_string(),
                    why: e2,
                },
            ])),
        }
    }
}
