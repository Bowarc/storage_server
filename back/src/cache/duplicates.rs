// While this map doesn't help in reducing storage time, it helps with taking less space on disk
type Hash = String;

#[derive(Debug, Default)]
pub struct DuplicateMap {
    // hashbrown could be used here
    inner: std::collections::HashMap<Hash, Vec<uuid::Uuid>>,
}

impl DuplicateMap {
    pub fn init_from_cache_dir() -> Self {
        use {rocket::serde::json::serde_json, std::fs::OpenOptions};

        let Ok(map_file) = OpenOptions::new()
            .read(true)
            .open(super::fs::duplicates_path())
            .map_err(|e| {
                error!("Failed to load duplicate map from fs due to: {e}\nFalling back to default")
            })
        else {
            return Default::default();
        };

        let map = serde_json::from_reader(map_file).unwrap_or_default();

        Self { inner: map }
    }

    pub fn write_to_file(&self) -> Result<(), crate::error::CacheError> {
        use {
            crate::error::CacheError,
            rocket::serde::json::serde_json::to_string,
            std::{fs::OpenOptions, io::Write as _},
        };

        let path = super::fs::duplicates_path();

        let mut file = OpenOptions::new()
            .create(true) // In case it does not yet exist
            .write(true)
            .truncate(true) // Rewrite all
            .open(path.clone())
            .map_err(|e| CacheError::FileOpen {
                file: path.display().to_string(),
                why: e,
            })?;

        let json = to_string(&self.inner).map_err(|e| CacheError::Serialization {
            context: "serializing duplicate map".to_string(),
            why: e,
        })?;

        file.write_all(json.as_bytes())
            .map_err(|e| CacheError::FileWrite {
                file: path.display().to_string(),
                why: e,
            })?;

        Ok(())
    }

    pub fn get(&self, hash: &Hash) -> Option<&[uuid::Uuid]> {
        self.inner.get(hash).map(|v| v.as_slice())
    }
    pub fn add(&mut self, hash: Hash, uuid: uuid::Uuid) -> Result<(), crate::error::CacheError> {
        self.inner.entry(hash).or_default().push(uuid);
        self.write_to_file()
    }

    // Removes and return every hash(key) that this uuid(value) is associated to
    // (Should be one at max)
    pub fn remove(&mut self, uuid: &uuid::Uuid) -> Result<Vec<String>, crate::error::CacheError> {
        // Smallvec ? https://docs.rs/smallvec
        let mut keys_to_remove = Vec::new();
        let mut out = Vec::new();

        for (key, vec) in self.inner.iter_mut() {
            let base_len = vec.len();
            vec.retain(|u| u != uuid);

            if base_len == vec.len() {
                continue;
            }

            out.push(key.clone());

            if vec.is_empty() {
                keys_to_remove.push(key.clone());
            }
        }

        for key in keys_to_remove {
            self.inner.remove(&key);
        }

        self.write_to_file().map(|_| out)
    }
}

/// This function moves or remove the data file depending on if it already exists
// Can't do it in store_data since we only have access to the original bytes
// which is pre compression, so much more than if we read after
// FIXME: This is dirty and REALLY ugly
pub async fn handle_duplicates(
    data_file_path: &mut std::path::PathBuf,
    uuid: &uuid::Uuid,
    duplicate_map: &std::sync::Arc<rocket::tokio::sync::Mutex<DuplicateMap>>,
) -> Result<(), crate::error::CacheError> {
    use {
        crate::error::CacheError,
        rocket::tokio::{
            fs::{remove_file, rename},
            sync::Mutex,
        },
        std::sync::LazyLock,
    };
    let (r, duration) = time::timeit_async(async || hash_file(data_file_path.clone()).await).await;
    debug!("Hash: {:?} in {}", r, time::format(duration, -1));
    let hash = r?;

    // Quick and dirty way to avoid all possibilities of data races on file moves
    static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

    let guard = MUTEX.lock().await;

    let is_duplicate = {
        let mut duplicate_map_handle = duplicate_map.lock().await;

        duplicate_map_handle.add(hash.clone(), *uuid)?;

        // Get corresponding uuid list from hash
        let is_dup = duplicate_map_handle
            .get(&hash)
            .map(|uuid_list| uuid_list.len() > 1);

        // The hash should be registered since the minimum element list would be the current download
        if is_dup.is_none() {
            duplicate_map_handle.add(hash.clone(), *uuid)?
        }

        is_dup.unwrap_or(false)
    };

    // let new_dp = fs::CACHE_DIRECTORY.join(hash.clone());
    let new_data_file_path = super::fs::data_path(&hash);

    if is_duplicate {
        remove_file(data_file_path.clone())
            .await
            .map_err(|e| CacheError::FileRemove {
                file: data_file_path.display().to_string(),
                why: e,
            })?;
    } else {
        rename(data_file_path.clone(), &new_data_file_path)
            .await
            .map_err(|e| CacheError::FileRename {
                file: data_file_path.display().to_string(),
                why: e,
            })?;
    }

    *data_file_path = new_data_file_path;

    drop(guard);

    Ok(())
}

async fn hash_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, crate::error::CacheError> {
    use {
        crate::error::CacheError,
        rocket::tokio::{fs::File, io::AsyncReadExt as _},
        sha2::{Digest as _, Sha256},
    };

    let path = path.as_ref();

    let mut file = File::open(path).await.map_err(|e| CacheError::FileOpen {
        file: path.display().to_string(),
        why: e,
    })?;

    let mut hasher = Sha256::default();
    let mut buffer = [0; 8192];

    while let Ok(bytes_read) = file.read(&mut buffer).await {
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
