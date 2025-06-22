use std::{io::Read, str::FromStr};

// RAM loaded cache
#[derive(Debug, serde::Serialize)]
pub struct CacheEntry {
    uuid: uuid::Uuid,
    upload_info: super::UploadInfo,
    is_ready: std::sync::atomic::AtomicBool,
    size: std::sync::atomic::AtomicU64,
}

impl CacheEntry {
    pub fn new(uuid: uuid::Uuid, upload_info: super::UploadInfo) -> Self {
        use std::sync::atomic::{AtomicBool, AtomicU64};

        Self {
            uuid,
            upload_info,
            is_ready: AtomicBool::new(false),
            size: AtomicU64::new(0),
        }
    }

    pub fn from_metadata(uuid: uuid::Uuid, metadata: super::Metadata, ready: bool) -> Self {
        use std::sync::atomic::{AtomicBool, AtomicU64};

        Self {
            uuid,
            upload_info: super::UploadInfo::new(
                metadata.name().clone(),
                metadata.extension().clone(),
            ),
            is_ready: AtomicBool::new(ready),
            size: AtomicU64::new(*metadata.size()),
        }
    }

    pub fn from_file(path: std::path::PathBuf) -> Result<Self, crate::error::CacheError> {
        let Some(uuid) = path
            .file_stem()
            .and_then(|os_str| os_str.to_str().map(|s| uuid::Uuid::from_str(s).ok()))
            .flatten()
        else {
            return Err(crate::error::CacheError::InvalidId {
                value: format!("{:?}", path.file_name()),
            });
        };

        if path.extension().and_then(|os_str| os_str.to_str()) != Some("meta") {
            return Err(crate::error::CacheError::WrongFileType {
                expected: String::from("meta"),
                actual: path
                    .extension()
                    .and_then(|os_str| os_str.to_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("Unknown file extension: {path:?}")),
            });
        }

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(path.clone())
            .map_err(|e| crate::error::CacheError::FileOpen {
                file: path.display().to_string(),
                why: e,
            })?;

        let metadata =
            rocket::serde::json::serde_json::from_reader::<std::fs::File, super::Metadata>(file)
                .map_err(|e| crate::error::CacheError::Deserialization {
                    file: path.display().to_string(),
                    why: e,
                })?;

        Ok(Self::from_metadata(uuid, metadata, true))
    }

    pub fn build_metadata(&self) -> super::Metadata {
        super::Metadata::new(
            self.upload_info.name().clone(),
            self.upload_info.extension().clone(),
            self.data_size(),
        )
    }

    pub fn upload_info(&self) -> &super::UploadInfo {
        &self.upload_info
    }

    pub fn is_ready(&self) -> bool {
        use std::sync::atomic::Ordering;

        self.is_ready.load(Ordering::Acquire)
    }

    pub fn set_ready(&self, ready: bool) {
        use std::sync::atomic::Ordering;

        self.is_ready.store(ready, Ordering::Release)
    }

    pub fn data_size(&self) -> u64 {
        use std::sync::atomic::Ordering;

        self.size.load(Ordering::Acquire)
    }

    pub fn set_data_size(&self, size: u64) {
        use std::sync::atomic::Ordering;

        self.size.store(size, Ordering::Release)
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }
    
    pub async fn store(
        self: std::sync::Arc<Self>,
        data_stream: rocket::data::DataStream<'_>,
    ) -> Result<(), crate::error::CacheError> {
        use rocket::data::ByteUnit;

        assert!(!self.is_ready());

        let id = self.uuid().hyphenated().to_string();

        let (res, exec_time) = time::timeit_async(|| super::store(self.clone(), data_stream)).await;

        let original_data_length = res?;

        debug!(
            "[{id}] Cache was successfully compresed ({} -> {}) in {}",
            ByteUnit::Byte(original_data_length),
            ByteUnit::Byte(self.data_size()),
            time::format(exec_time, 2)
        );
        Ok(())
    }

    // Load a stored cache entry
    pub fn load(
        self: std::sync::Arc<Self>
    ) -> Result<(super::UploadInfo, Box<dyn std::io::Read + Send>), crate::error::CacheError> {
        use crate::error::CacheError;
        // Load and decompress the given cache self

        let uuid = self.uuid();

        if !self.is_ready() {
            return Err(CacheError::NotReady { uuid });
        }

        let file_path = super::data_path(&uuid);

        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(&file_path)
            .map_err(|e| CacheError::FileOpen {
                file: file_path.display().to_string(),
                why: e,
            })?;

        let decoder = zstd::stream::Decoder::new(file).map_err(|e| CacheError::FileOpen {
            file: file_path.display().to_string(),
            why: e,
        })?;

        Ok((self.upload_info.clone(), Box::new(decoder)))
    }

    /// Delete a cache entry
    pub async fn delete(self: std::sync::Arc<Self>) -> Result<(), crate::error::CacheError> {
        use {crate::error::CacheError, tokio::fs::remove_file};

        let meta_path = super::meta_path(&self.uuid);
        let data_path = super::data_path(&self.uuid);

        match futures::join!(remove_file(&meta_path), remove_file(&data_path),) {
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

