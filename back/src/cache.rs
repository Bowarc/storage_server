/*
    Fs: 3 file types
    - Data files:
        Name is a uuid with no extension
        Raw content of a file, compressed using zstd with COMPRESSION_LEVEL level
    - Meta files:
        The name is a uuid (not related to the data file) with .meta at the end
        Stores data about an uploaded file.
        One meta file per upload but multiple meta file can point to the same data file if content are duplicates
        File structure is metadata::Metadata
    - Duplicate file:
        On disk duplicate tracking storage.
        A serialized version of the duplicates::DuplicateMap struct.
*/

mod duplicates;
mod entry;
mod fs;
mod metadata;
mod size;
mod upload_info;

pub use duplicates::DuplicateMap;
pub use entry::CacheEntry;
pub use metadata::Metadata;
pub use size::Size;
pub use upload_info::UploadInfo;

const COMPRESSION_LEVEL: i32 = zstd::DEFAULT_COMPRESSION_LEVEL; // 3, 1..=22 (zstd)

pub type CacheEntryList = Vec<std::sync::Arc<CacheEntry>>;

pub fn init_cache_list_from_cache_dir() -> Option<CacheEntryList> {
    use {
        std::{path::PathBuf, str::FromStr as _, sync::Arc},
        uuid::Uuid,
    };

    let files = fs::read_cache_dir().ok()?;

    // The default one is bad
    let display_path = |path: PathBuf| -> String { path.display().to_string().replace("\\", "/") };

    let inner = files
        .flatten()
        .flat_map(|entry| {
            let metadata = entry
                .metadata()
                .map_err(|e| {
                    error!(
                        "Could not read metadata from cache item '{p}' due to: {e}",
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

            if path.extension().and_then(|ext| ext.to_str()) != Some("meta") {
                // Not a meta file, don't care
                return None;
            }

            let Some(id) = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|s| Uuid::from_str(s).ok())
            else {
                warn!(
                    "Could not extract id from cache file '{}'",
                    display_path(path)
                );
                return None;
            };

            CacheEntry::from_file(path)
                .map_err(|e| error!("Could not load cache for id: '{id}' due to: {e}"))
                .map(Arc::new)
                .ok()
        })
        .collect::<Vec<Arc<CacheEntry>>>();

    debug!("Loaded {} cache entries", inner.len());

    Some(inner)
}

/// Takes an incomming data stream, compresses and stores it in a given 'data' file.
/// Returns the file size before compression and the resulting file size
async fn stream_to_file(
    uuid: &uuid::Uuid,
    mut original_data: rocket::data::DataStream<'_>,
    data_file: &mut std::fs::File,
) -> Result<Size, crate::error::CacheError> {
    use {
        crate::error::CacheError,
        rocket::data::{ByteUnit, ToByteUnit as _},
        rocket::tokio::io::AsyncReadExt as _,
        std::io::Write as _,
        zstd::stream::Encoder,
    };

    let mut encoder = Encoder::new(data_file, COMPRESSION_LEVEL)
        .map_err(|e| CacheError::Compression { why: e })?;

    // I am not too happy with that allocation, but since we're in an async context
    // (and i don't really know how async task works, so idk if thread local is usable here), we don't have much choice
    // The other way could be to make it on the stack, but if we do that, we would be very limited in size
    // since tokio's threads don't have a big stack size
    const BUFFER_SIZE: usize = 500_000; // 500kb
    #[rustfmt::skip]
    let mut buffer = vec![0; BUFFER_SIZE];

    let byte_size_limit: ByteUnit = unsafe {
        // SAFETY:
        //     This static is ONLY EVER mutated at the program's init, before the webserer is even running
        crate::FILE_REQ_SIZE_LIMIT.bytes()
    };

    let mut total_read = 0;
    loop {
        let read = original_data
            .read(&mut buffer)
            .await
            .map_err(|e| CacheError::Compression { why: e })?;

        if read == 0 {
            break;
        }

        total_read += read;

        encoder
            .write_all(&buffer[..read])
            .map_err(|e| CacheError::Compression { why: e })?;

        if total_read > byte_size_limit {
            error!("Max size reached");
            return Err(CacheError::FileSizeExceeded);
        }
    }

    let data_file = encoder
        .finish()
        .map_err(|e| CacheError::Compression { why: e })?;

    // This is a bit ugly, but since `decoder.finish()` also writes things to the file
    // using the total written bytes count as 'total file size' yields incorrect results.
    // Since I don't know how to predict how many more bytes are written in that `encoder.finish()`
    // I directly use the file metadata.
    let file_size = {
        let metadata = data_file.metadata().map_err(|e| CacheError::FileRead {
            file: format!("(data file for uuid ({uuid})"),
            why: e,
        })?;

        metadata.len()
    };

    debug!(
        "totals:\nRead: {}\nWrote: {}\nDelta: {}%",
        total_read,
        file_size,
        {
            let d = -100. * (1. - (file_size as f64 / total_read as f64));
            if d > 0. {
                format!("+{d:.3}")
            } else {
                format!("{d:.3}")
            }
        }
    );

    Ok(Size::new(total_read as u64, file_size))
}
