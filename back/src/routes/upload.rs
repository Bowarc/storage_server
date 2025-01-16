use rocket::{data::ByteUnit, http::RawStr};

lazy_static! {
    static ref FILENAME_VALIDATION_REGEX: regex::Regex =
        regex::Regex::new(r"^[A-Za-z0-9_.-]{1,100}$").unwrap();
}

#[rocket::put("/<filename>", data = "<raw_data>")]
pub async fn api_upload(
    filename: &str,
    raw_data: rocket::data::Data<'_>,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> crate::response::Response {
    use {
        crate::response::ResponseBuilder,
        rocket::http::{ContentType, Status},
        std::time::Instant,
        uuid::Uuid,
    };

    let start_timer = Instant::now();

    let uuid = Uuid::new_v4();
    let wait_store = true; // Probably better to make this an endpoint like /api/upload/ and /api/upload/awaited/;

    debug!(
        "Received new upload request \nUsing id: {uuid}\nUsername: {}\nFile name: {}",
        "NO_USER", filename,
    );

    // Validation of user input
    if !FILENAME_VALIDATION_REGEX.is_match(filename) {
        error!("[{uuid}] The given filename doesn't match the validation regex");
        return ResponseBuilder::default()
            .with_status(Status::BadRequest)
            .with_content("The specified filename should only contain alphanumeric characters, underscores, dots and shouldn't be longer than 100 characters")
            .with_content_type(ContentType::Text)
            .build();
    }

    // let data_stream = raw_data.open(unsafe{crate::FILE_REQ_SIZE_LIMIT});
    let data_stream = raw_data.open(ByteUnit::max_value());

    // let capped_data = match raw_data
    //     .open(unsafe { crate::FILE_REQ_SIZE_LIMIT })
    //     .into_bytes()
    //     .await
    // {
    //     Ok(data) => data,
    //     Err(e) => {
    //         error!("[{uuid}] Could not parse given data: {e}");

    //         return ResponseBuilder::default()
    //             .with_status(Status::BadRequest)
    //             .with_content("The given body content could not be parsed")
    //             .with_content_type(ContentType::Text)
    //             .build();
    //     }
    // };

    // if !capped_data.is_complete() {
    //     error!("Data too large");
    //     return ResponseBuilder::default()
    //         .with_status(Status::PayloadTooLarge)
    //         .with_content("The given data is too large")
    //         .with_content_type(ContentType::Text)
    //         .build();
    // }

    // let data = capped_data.value;

    // No need to decode user input as it's not b64 encoded anymore

    let mut cache_handle = cache.write().await;

    let cache_entry = cache_handle.new_entry(uuid, 
        crate::cache::data::UploadInfo::new(
            "NO_USER".to_string(),
            get_file_name(filename).unwrap_or_default(),
            get_file_extension(filename).unwrap_or_default(),
        )
    );

    drop(cache_handle);

    crate::cache::Cache::store(cache_entry, data_stream).await.unwrap();
    

    // Release the lock to be able to wait the end of the 'exec' without denying other calls
    // drop(cache_handle);

    // if wait_store {
    //     debug!("[{uuid}] Waiting for cache to finish storing the data");

    //     match exec.await {
    //         Ok(()) => {
    //             // All good
    //         }
    //         Err(e) => {
    //             error!("[{uuid}] An error occured while storing the given data: {e}");
    //             return ResponseBuilder::default()
    //                 .with_status(Status::InternalServerError)
    //                 .with_content("An error occured while caching the data")
    //                 .with_content_type(ContentType::Text)
    //                 .build();
    //         }
    //         // Err(join_error) => {
    //         //     error!("[{uuid}] Something went really bad while waiting for worker task to end: {join_error}");
    //         //     return ResponseBuilder::default()
    //         //         .with_status(Status::InternalServerError)
    //         //         .with_content("Worker failled")
    //         //         .with_content_type(ContentType::Text)
    //         //         .build();
    //         // }
    //     }
    // }

    info!(
        "[{uuid}] Responded in {}",
        time::format(start_timer.elapsed(), 2)
    );
    ResponseBuilder::default()
        .with_status(Status::Created)
        .with_content(uuid.hyphenated().to_string())
        .with_content_type(ContentType::Text)
        .build()
}

fn get_file_name(name: &str) -> Option<String> {
    if !name.contains(".") {
        return None;
    }

    let dot_index = name.rfind(".").unwrap();

    Some(String::from(&name[0..dot_index]))
}

fn get_file_extension(name: &str) -> Option<String> {
    if !name.contains(".") {
        return None;
    }

    let dot_index = name.rfind(".").unwrap();

    Some(String::from(&name[(dot_index + 1)..name.len()]))
}

#[cfg(test)]
mod tests {
    use {
        crate::build_rocket,
        rocket::{http::Status, local::asynchronous::Client},
        std::str::FromStr,
    };

    #[rocket::async_test]
    async fn test_upload() {
        let client = Client::tracked(build_rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client
            .put("/test.file")
            .header(rocket::http::ContentType::Text)
            .body("This is normal file content")
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
        let _uuid = uuid::Uuid::from_str(&suuid).unwrap();
    }
}
