// While this map doesn't help in reducing storage time, it helps with taking less space on disk

type Hash = String;

#[derive(Debug, Default)]
pub struct DuplicateMap {
    inner: std::collections::HashMap<Hash, Vec<uuid::Uuid>>,
}

impl DuplicateMap {
    pub fn init_from_cache_dir() -> Self {
        use rocket::serde::json::serde_json::from_reader;
        use std::fs::{read_dir, OpenOptions};

        let Ok(files) = read_dir(super::CACHE_DIRECTORY)
            .map_err(|e| error!("Could not open cache dir due to: {e}"))
        else {
            return Default::default();
        };

        let Some(map_file) = files
            .flatten()
            .find(|f| f.file_name() == "duplicates.json")
            .and_then(|dir_entry| OpenOptions::new().read(true).open(dir_entry.path()).ok())
        else {
            return Default::default();
        };

        let map = from_reader(map_file).unwrap_or_default();

        Self { inner: map }
    }

    pub fn write_to_file(&self) -> Result<(), crate::error::CacheError> {
        use {
            crate::error::CacheError,
            rocket::serde::json::serde_json::to_string,
            std::{fs::OpenOptions, io::Write as _},
        };

        let path = format!("{}/duplicates.json", super::CACHE_DIRECTORY);

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.clone())
            .map_err(|e| CacheError::FileOpen {
                file: path.clone(),
                why: e,
            })?;

        let json = to_string(&self.inner).map_err(|e| CacheError::Serialization {
            context: "serializing duplicate map".to_string(),
            why: e,
        })?;

        file.write_all(json.as_bytes())
            .map_err(|e| CacheError::FileWrite { file: path, why: e })?;

        Ok(())
    }

    pub fn get(&self, hash: &Hash) -> Option<&[uuid::Uuid]> {
        self.inner.get(hash).map(|v| v.as_slice())
    }
    pub fn add(&mut self, hash: Hash, uuid: uuid::Uuid) {
        self.inner.entry(hash).or_default().push(uuid);
        self.write_to_file().unwrap()
    }
    pub fn remove(&mut self, uuid: &uuid::Uuid) {
        let mut keys_to_remove = Vec::new();

        for (key, vec) in self.inner.iter_mut() {
            vec.retain(|u| u != uuid);

            if vec.is_empty() {
                keys_to_remove.push(key.clone());
            }
        }

        for key in keys_to_remove {
            self.inner.remove(&key);
        }

        self.write_to_file().unwrap()
    }
}
