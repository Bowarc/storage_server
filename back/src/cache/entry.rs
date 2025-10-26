#[derive(Debug, serde::Serialize)]
/// Having multiple CacheEntry instance of the same uuid is NOT allowed and would dismiss all
/// security
pub struct CacheEntry {
    uuid: uuid::Uuid,
    upload_info: super::UploadInfo,
    size: super::Size,

    #[serde(skip_serializing)]
    file_lock: std::sync::Arc<parking_lot::RwLock<()>>,
}

// Getters / Setters, easier to read if they are separated
impl CacheEntry {
    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
}

// Init methods
impl CacheEntry {
    pub fn from_file(path: std::path::PathBuf) -> Result<Self, crate::error::CacheError> {
        use {
            super::Metadata,
            crate::error::CacheError,
            rocket::serde::json::serde_json,
            std::{
                fs::{File, OpenOptions},
                str::FromStr as _,
            },
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

        Ok(Self {
            uuid,
            upload_info: super::UploadInfo::new(
                metadata.name().clone(),
                metadata.extension().clone(),
            ),
            size: *metadata.size(),

            file_lock: Default::default(),
        })
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
            std::path::PathBuf,
        };

        let start_time = std::time::Instant::now();

        let meta_path = super::fs::meta_path(&uuid);
        // Data path needs to be mutable since since it may be swapped for an already exising file
        // in the duplicate detection
        // Could also return a new one but eh
        let mut data_path = super::fs::temp_data_path(&uuid);

        let cleanup_files = |meta_path: PathBuf, data_path: PathBuf| async {
            match futures::join!(remove_file(meta_path), remove_file(data_path)) {
                (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
                    error!("[{uuid}] Failed to cleanup after error due to: {e}")
                }
                (Err(e1), Err(e2)) => {
                    error!("[{uuid}] Failed to cleanup after error due to: {e1} AND {e2}")
                }
                _ => (),
            }
        };

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
                    // We know that the files were created, so any error here are important
                    cleanup_files(meta_path, data_path).await;
                    return Err(e);
                }
            }
        };

        // Make sure the file is not a duplicate, in what case we'll remove the data file we just created
        // and use the exising one
        // FIXME: The implementation of this is really ugly
        if let Err(e) =
            super::duplicates::handle_duplicates(&mut data_path, &uuid, &duplicate_map).await
        {
            cleanup_files(meta_path, data_path).await;

            return Err(e);
        }

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
            cleanup_files(meta_path, data_path).await;
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
            size: data_size,

            file_lock: Default::default(),
        })
    }

    // Load a stored cache entry
    pub async fn load(
        &self,
    ) -> Result<(super::UploadInfo, Box<dyn std::io::Read + Send>), crate::error::CacheError> {
        use {crate::error::CacheError, std::fs::OpenOptions, zstd::stream::Decoder};

        struct DecoderWrapper<'t, T, U> {
            decoder: zstd::stream::Decoder<'t, T>,
            _file_lock: U,
        }

        impl<'t, T, U> std::io::Read for DecoderWrapper<'t, T, U>
        where
            T: std::io::Read + Send + std::io::BufRead,
        {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.decoder.read(buf)
            }
        }

        let (lock, duration) = time::timeit(|| self.file_lock.read_arc());

        debug!(
            "Download of cache {}, acquired lock in {}",
            self.uuid,
            time::format(duration, 1)
        );

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

        Ok((
            self.upload_info.clone(),
            Box::new(DecoderWrapper {
                decoder,
                _file_lock: lock,
            }),
        ))
    }

    /// Delete a cache entry
    pub async fn delete(
        &self,
        duplicate_map: std::sync::Arc<rocket::tokio::sync::Mutex<super::DuplicateMap>>,
    ) -> Result<(), crate::error::CacheError> {
        // #![allow(clippy::await_holding_lock)]
        // This was for the file_lock but it's fixed using the arc guard

        use {crate::error::CacheError, tokio::fs::remove_file};

        let (lock, duration) = time::timeit(|| self.file_lock.write_arc());

        debug!(
            "Deletion of cache {}, acquired lock in {}",
            self.uuid,
            time::format(duration, 1)
        );

        // warn!("Tried to delete a file currently accessed !\n{:?}", self.file_lock.read());
        // return Err(CacheError::NotReady { uuid: self.uuid });

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

        let meta_path = super::fs::meta_path(&self.uuid);
        let data_path = super::fs::data_path(metadata.data_file_name());

        let res = match futures::join!(remove_file(&meta_path), remove_file(&data_path)) {
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
        };

        // Make sure the lock is kept and not optimized out by the compiler
        drop(lock);

        res
    }
}
