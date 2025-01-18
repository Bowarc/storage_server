lazy_static! {
    // This regex only match uuid v4
    static ref UUID_VALIDATION_REGEX: regex::Regex = regex::Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
    )
    .unwrap();
}

#[inline]
fn parse_id(raw_id: &str) -> Result<uuid::Uuid, crate::error::UuidParseError> {
    use {crate::error::UuidParseError, std::str::FromStr, uuid::Uuid};

    if !UUID_VALIDATION_REGEX.is_match(raw_id) {
        return Err(UuidParseError::Regex);
    }

    Uuid::from_str(raw_id).map_err(|_e| UuidParseError::Convert)
}

///
/// This route is the main way to download a cache's content
///
///     As input, it only requires the cache's uuid
///
///     It returns, file cache's content, decompressed and in an directly usable format.
///         As for the filename, it sets the 'Content-Disposition' header for the browser to interpret
///
#[rocket::get("/<id>")]
pub async fn api_download(
    id: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> crate::response::Response {
    use {
        crate::{
            error::{CacheError, UuidParseError},
            response::ResponseBuilder,
        },
        rocket::http::{ContentType, Status},
        std::time::Instant,
    };

    let start_timer = Instant::now();

    info!("Request of {id}");

    let uuid = match parse_id(id) {
        Ok(uuid) => uuid,
        Err(UuidParseError::Regex) => {
            error!("[{id}] Given id doesn't match expected character range");
            return ResponseBuilder::default()
                .with_status(Status::BadRequest)
                .with_content("Given id doesn't match expected character range")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(UuidParseError::Convert) => {
            error!("[{id}] Invalid id");
            return ResponseBuilder::default()
                .with_status(Status::BadRequest)
                .with_content(format!("Invalid id: {id}"))
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    let cache_handle = cache.read().await;

    let load_res = cache_handle.load(uuid).await;

    // Keep the lock for the minimum amount of time
    drop(cache_handle);

    let (meta, data) = match load_res {
        Ok(meta_data) => meta_data,
        Err(CacheError::NotReady) => {
            error!("[{uuid}] The requested cache is not ready yet");

            return ResponseBuilder::default()
                .with_status(Status::NotFound)
                .with_content("The requested cache is not ready yet")
                .with_content_type(ContentType::Text)
                .build();
        }

        Err(CacheError::NotFound) => {
            error!("[{uuid}] The given uuid doesn't correspnd to any cache entry");
            return ResponseBuilder::default()
                .with_status(Status::NotFound)
                .with_content("The given id doesn't correspond to any cache entry")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(CacheError::FileOpen(e)) => {
            error!("[{uuid}] Failled to open data file of [{uuid}] due to: {e}");
            return ResponseBuilder::default()
                .with_status(Status::InternalServerError)
                .with_content("Could not acces given id's content")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(CacheError::FileRead(e)) => {
            error!("[{uuid}] Failled to read cache of [{uuid}] due to: {e}");
            return ResponseBuilder::default()
                .with_status(Status::InternalServerError)
                .with_content("Could not access given cache entry")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(e) => {
            error!("[{uuid}] Unexpected error: {e}");
            return ResponseBuilder::default()
                .with_status(Status::InternalServerError)
                .with_content("Could not access the requested cache. The error has been logged.")
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    info!(
        "[{uuid}] Responded in {}",
        time::format(start_timer.elapsed(), 2)
    );

    ResponseBuilder::default()
        .with_status(Status::Ok)
        .with_content(data)
        .with_content_type(ContentType::Bytes)
        .with_header(
            "Content-Disposition",
            &format!(
                "attachment; filename=\"{}.{}\"",
                meta.name(),
                meta.extension()
            ),
        )
        .build()
}

#[rocket::get("/stream/<id>")]
pub async fn api_download_stream(
    id: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> crate::response::Response {
    use {
        crate::{
            error::{CacheError, UuidParseError},
            response::ResponseBuilder,
        },
        rocket::http::{ContentType, Status},
        std::time::Instant,
    };
    let start_timer = Instant::now();

    info!("Request of {id}");

    let uuid = match parse_id(id) {
        Ok(uuid) => uuid,
        Err(UuidParseError::Regex) => {
            error!("[{id}] Given id doesn't match expected character range");
            return ResponseBuilder::default()
                .with_status(Status::BadRequest)
                .with_content("Given id doesn't match expected character range")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(UuidParseError::Convert) => {
            error!("[{id}] Invalid id");
            return ResponseBuilder::default()
                .with_status(Status::BadRequest)
                .with_content(format!("Invalid id: {id}"))
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    let cache_handle = cache.read().await;

    let load_result = cache_handle.load_stream(uuid).await;

    // Keep the lock for the minimum amount of time
    drop(cache_handle);

    let (meta, data_stream) = match load_result {
        Ok(meta_data) => meta_data,
        Err(CacheError::NotReady) => {
            error!("[{uuid}] The requested cache is not ready yet");
            return ResponseBuilder::default()
                .with_status(Status::NotFound)
                .with_content("The requested cache is not ready yet")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(CacheError::NotFound) => {
            error!("[{uuid}] The given uuid doesn't correspnd to any cache entry");
            return ResponseBuilder::default()
                .with_status(Status::NotFound)
                .with_content("The given id doesn't correspond to any cache entry")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(CacheError::FileOpen(e)) => {
            error!("[{uuid}] Failled to open data file of [{uuid}] due to: {e}");
            return ResponseBuilder::default()
                .with_status(Status::InternalServerError)
                .with_content("Could not access given cache entry")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(CacheError::FileRead(e)) => {
            error!("[{uuid}] Failled to read cache of [{uuid}] due to: {e}");
            return ResponseBuilder::default()
                .with_status(Status::InternalServerError)
                .with_content_type(ContentType::Text)
                .with_content("Could not acces given id's content")
                .build();
        }
        Err(e) => {
            error!("[{uuid}] Unexpected error: {e}");
            return ResponseBuilder::default()
                .with_status(Status::InternalServerError)
                .with_content("Could not access the requested cache. The error has been logged.")
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    info!(
        "[{uuid}] Responded in {}",
        time::format(start_timer.elapsed(), 2)
    );

    ResponseBuilder::default()
        .with_status(Status::Ok)
        .with_content(data_stream)
        .with_content_type(ContentType::Bytes)
        .with_header(
            "Content-Disposition",
            &format!(
                "attachment; filename=\"{}.{}\"",
                meta.name(),
                meta.extension()
            ),
        )
        .build()
}


///
/// This route is the seccond way to download a cache's content
///
///     As input, it requires the cache's uuid and the file name to be the right one
///
///     It returns, file cache's content, decompressed and in an directly usable format.
///
///     This route is a proxy and mostly for curl users to be able to use '-O' (download with auto file name)
///
#[rocket::get("/<id>/<filename>")]
pub async fn api_download_filename(
    id: &str,
    filename: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> crate::response::Response {
    use {
        crate::response::ResponseBuilder,
        rocket::http::{ContentType, Status},
    };

    let resp = api_download(id, cache).await;

    if resp.status() != &Status::Ok {
        // If the internal call returned an error, there is no point doing the filename verification
        return resp;
    }

    let Some(content_disposition_header) = resp.headers().get("Content-Disposition") else {
        error!("Could not find the 'Content-Disposition' header of the api_download internal call");
        return ResponseBuilder::default()
            .with_status(Status::InternalServerError)
            .build();
    };

    // This won't truncate the real file name as quotes are not allowed in filename / extensions (see upload.rs::FILENAME_VALIDATION_REGEX)
    let header_filename = content_disposition_header
        .replace("attachment; filename=\"", "")
        .replace('"', "");

    if header_filename != filename {
        error!("The user supplied filename: '{filename}' but the one stored in metadata is '{header_filename}'");
        return ResponseBuilder::default()
            .with_status(Status::BadRequest)
            .with_content(format!(
                "Incorrect file name, did you meant '{header_filename}'?"
            ))
            .with_content_type(ContentType::Text)
            .build();
    }

    resp
}

#[rocket::head("/<id>")]
pub async fn api_download_head(
    id: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> String {
    use {std::str::FromStr, uuid::Uuid};

    info!("Request of HEAD {id}");
    let uuid = Uuid::from_str(id).unwrap();

    let upload_info = cache.read().await.get_entry(uuid).await.unwrap();

    format!(
        "{}.{}",
        upload_info.upload_info().name(),
        upload_info.upload_info().extension()
    )
}

#[cfg(test)]
mod tests {
    use {
        crate::build_rocket,
        rocket::{http::Status, local::asynchronous::Client},
        std::str::FromStr,
    };

    #[rocket::async_test]
    async fn test_download() {
        let base_filename = "test.file";

        let uuid = {
            // Setup
            let client = Client::tracked(build_rocket().await)
                .await
                .expect("valid rocket instance");

            let response = client
                .put(format!("/{base_filename}"))
                .header(rocket::http::ContentType::Text)
                .body("This is a cool file content")
                .dispatch()
                .await;

            #[allow(deprecated)] // stfu ik
            std::thread::sleep_ms(500);

            assert_eq!(response.status(), Status::Created);

            let rs = response.into_string().await.unwrap();

            info!("{}", rs);

            let suuid = rs.replace("Success: ", "");
            uuid::Uuid::from_str(&suuid).unwrap()
        };

        let client = Client::tracked(build_rocket().await)
            .await
            .expect("valid rocket instance");

        let response = client
            .get(format!("/{uuid}", uuid = uuid.hyphenated()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.headers().get_one("Content-Disposition").unwrap(),
            format!("attachment; filename=\"{base_filename}\"")
        );
    }

    #[rocket::async_test]
    async fn test_download_filename() {
        logger::init(
            logger::Config::default()
                .output(logger::Output::Stdout)
                .filter("rocket", log::LevelFilter::Warn),
        );

        let base_filename = "test.file";

        let uuid = {
            // Setup
            let client = Client::tracked(build_rocket().await)
                .await
                .expect("valid rocket instance");
            let response = client
                .put(format!("/{base_filename}"))
                .header(rocket::http::ContentType::Text)
                .body("This is a co")
                .dispatch()
                .await;

            #[allow(deprecated)] // stfu ik
            std::thread::sleep_ms(500);

            assert_eq!(response.status(), Status::Created);
            let suuid = response
                .into_string()
                .await
                .unwrap()
                .replace("Success: ", "");

            uuid::Uuid::from_str(&suuid).unwrap()
        };

        let client = Client::tracked(build_rocket().await)
            .await
            .expect("valid rocket instance");

        let response = client
            .get(format!("/{uuid}/{base_filename}", uuid = uuid.hyphenated()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.headers().get_one("Content-Disposition").unwrap(),
            format!("attachment; filename=\"{base_filename}\"")
        );
    }
}
