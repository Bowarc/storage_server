#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub uuid: uuid::Uuid,
    pub metadata: Metadata,
    pub is_ready: std::sync::atomic::AtomicBool,
    pub data_size: std::sync::atomic::AtomicUsize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Metadata {
    pub username: String,
    pub file_name: String,
    pub file_ext: String,
    // ...
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UploadData {
    pub metadata: Metadata,
    pub file: String,
}

impl CacheEntry {
    pub fn new(uuid: uuid::Uuid, metadata: Metadata) -> Self {
        Self {
            uuid,
            metadata,
            is_ready: std::sync::atomic::AtomicBool::new(false),
            data_size: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    pub fn is_ready(&self) -> bool {
        self.is_ready.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn set_ready(&mut self, rdy: bool) {
        self.is_ready
            .store(rdy, std::sync::atomic::Ordering::Relaxed)
    }
    pub fn data_size(&self) -> usize {
        self.data_size.load(std::sync::atomic::Ordering::Relaxed)
    }
}
