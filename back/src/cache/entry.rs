#[derive(Debug, serde::Serialize)]
pub struct CacheEntry {
    uuid: uuid::Uuid,
    upload_info: super::UploadInfo,
    is_ready: bool,
    size: super::Size,
}

impl CacheEntry {
    pub fn new(uuid: uuid::Uuid, upload_info: super::UploadInfo) -> Self {
        Self {
            uuid,
            upload_info,
            is_ready: false,
            size: super::Size::new(0, 0),
        }
    }

    pub fn from_metadata(uuid: uuid::Uuid, metadata: super::Metadata, ready: bool) -> Self {
        Self {
            uuid,
            upload_info: super::UploadInfo::new(
                metadata.name().clone(),
                metadata.extension().clone(),
            ),
            is_ready: ready,
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
        self.is_ready
    }

    pub fn set_ready(&mut self, ready: bool) {
        self.is_ready = ready
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

    pub async fn store(
        &mut self,
        data_stream: rocket::data::DataStream<'_>,
        duplicate_map: std::sync::Arc<rocket::tokio::sync::Mutex<super::DuplicateMap>>,
    ) -> Result<(), crate::error::CacheError> {
        use rocket::data::ByteUnit;

        assert!(!self.is_ready());

        let id = self.uuid().hyphenated().to_string();

        let (res, exec_time) =
            time::timeit_async(|| super::store(self, data_stream, duplicate_map)).await;

        let size = res?;

        debug!(
            "[{id}] Cache was successfully compresed ({} -> {}) in {}",
            ByteUnit::Byte(size.original()),
            ByteUnit::Byte(size.compressed()),
            time::format(exec_time, 2)
        );
        Ok(())
    }

    // Load a stored cache entry
    pub fn load(
        &self,
    ) -> Result<(super::UploadInfo, Box<dyn std::io::Read + Send>), crate::error::CacheError> {
        use crate::error::CacheError;
        // Load and decompress the given cache self

        let uuid = self.uuid();

        if !self.is_ready() {
            return Err(CacheError::NotReady { uuid });
        }

        let meta_path = super::meta_path(&uuid);
        let meta_file = std::fs::OpenOptions::new()
            .read(true)
            .open(&meta_path)
            .map_err(|e| CacheError::FileOpen {
                file: meta_path.display().to_string(),
                why: e,
            })?;

        let metadata: super::Metadata = rocket::serde::json::serde_json::from_reader(meta_file)
            .map_err(|e| CacheError::Deserialization {
                file: meta_path.display().to_string(),
                why: e,
            })?;

        let data_path = {
            let mut p = super::CACHE_DIRECTORY.clone();
            p.push(metadata.data_file_name());
            p
        };

        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(&data_path)
            .map_err(|e| CacheError::FileOpen {
                file: data_path.display().to_string(),
                why: e,
            })?;

        let decoder = zstd::stream::Decoder::new(file).map_err(|e| CacheError::FileOpen {
            file: data_path.display().to_string(),
            why: e,
        })?;

        Ok((self.upload_info.clone(), Box::new(decoder)))
    }

    /// Delete a cache entry
    pub async fn delete(
        self: std::sync::Arc<Self>,

        duplicate_map: std::sync::Arc<rocket::tokio::sync::Mutex<super::DuplicateMap>>,
    ) -> Result<(), crate::error::CacheError> {
        use {
            crate::error::CacheError,
            tokio::{
                fs::{remove_file, File, OpenOptions},
                io::AsyncReadExt as _,
            },
        };

        let meta_path = super::meta_path(&self.uuid);

        let mut meta_file = OpenOptions::new()
            .read(true)
            .open(&meta_path)
            .await
            .map_err(|e| CacheError::FileOpen {
                file: meta_path.display().to_string(),
                why: e,
            })?;

        let mut buffer = Vec::<u8>::new();

        meta_file
            .read_to_end(&mut buffer)
            .await
            .map_err(|e| CacheError::FileRead {
                file: meta_path.display().to_string(),
                why: e,
            })?;

        let metadata = rocket::serde::json::serde_json::from_slice::<super::Metadata>(&buffer)
            .map_err(|e| CacheError::Deserialization {
                file: meta_path.display().to_string(),
                why: e,
            })?;

        let mut duplicate_map_guard = duplicate_map.lock().await;

        let hashes = duplicate_map_guard.remove(&self.uuid)?;

        if hashes.len() != 1 {
            return Err(CacheError::DuplicateMapLogic(
                format!("There was an issue with the deletion of [{}]: the dup map returned {} matches ({:?}) for uuid:{}", self.uuid, hashes.len(), hashes, self.uuid)
            ));
        }

        let data_hash = hashes.first().unwrap(); // Cannot fail

        // Meaning that the current uuid was NOT the last holder of that data hash
        if duplicate_map_guard.get(data_hash).is_some() {
            // Just remove the meta file, and leave
            remove_file(&meta_path)
                .await
                .map_err(|e| CacheError::FileRemove {
                    file: meta_path.display().to_string(),
                    why: e,
                })?;

            return Ok(());
        }

        let data_path = {
            let mut p = super::CACHE_DIRECTORY.clone();
            p.push(metadata.data_file_name());
            p
        };

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
