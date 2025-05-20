// Structure of a .meta file
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    name: String,
    extension: String,
    size: u64,
}

impl Metadata {
    pub fn new(name: String, extension: String, size: u64) -> Self {
        Self {
            name,
            extension,
            size,
        }
    }
}

// RAM loaded cache
#[derive(Debug, serde::Serialize)]
pub struct CacheEntry {
    uuid: uuid::Uuid,
    upload_info: UploadInfo,
    is_ready: std::sync::atomic::AtomicBool,
    size: std::sync::atomic::AtomicU64,
}

impl CacheEntry {
    pub fn new(uuid: uuid::Uuid, upload_info: UploadInfo) -> Self {
        use std::sync::atomic::{AtomicBool, AtomicU64};

        Self {
            uuid,
            upload_info,
            is_ready: AtomicBool::new(false),
            size: AtomicU64::new(0),
        }
    }

    pub fn from_metadata(uuid: uuid::Uuid, metadata: Metadata, ready: bool) -> Self {
        use std::sync::atomic::{AtomicBool, AtomicU64};

        Self {
            uuid,
            upload_info: UploadInfo {
                name: metadata.name,
                extension: metadata.extension,
            },
            is_ready: AtomicBool::new(ready),
            size: AtomicU64::new(metadata.size),
        }
    }

    pub fn upload_info(&self) -> &UploadInfo {
        &self.upload_info
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
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

    pub fn build_metadata(&self) -> Metadata {
        Metadata::new(
            self.upload_info.name().clone(),
            self.upload_info.extension().clone(),
            self.data_size(),
        )
    }
}

// The data supplied with an upload request
#[derive(Debug, serde::Serialize, Clone)]
pub struct UploadInfo {
    name: String,
    extension: String,
}

impl UploadInfo {
    pub fn new(name: String, extension: String) -> Self {
        Self {
            name,
            extension,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn extension(&self) -> &String {
        &self.extension
    }
}
