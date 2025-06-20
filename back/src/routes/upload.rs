lazy_static! {
    static ref FILENAME_VALIDATION_REGEX: regex::Regex =
        regex::Regex::new(r"^[a-zA-Z0-9_.-]{1,100}$").unwrap();
    // This regex is really simple:
    //
    // All basic latin letters are allowed, can be uppercase,
    // All arabic numbers are allowed
    // undercore, dot, hyphen are allowed, all other symbols are denied (I don't like spaces in file names)
    // Between 1 and 100 characters
}

#[rocket::put("/<filename>", data = "<raw_data>")]
pub async fn api_upload(
    filename: &str,
    raw_data: rocket::data::Data<'_>,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
    addr: rocket_client_addr::ClientAddr,
) -> crate::response::Response {
    use {
        crate::response::Response,
        rocket::{
            data::ByteUnit,
            http::{ContentType, Status},
        },
        std::time::Instant,
        uuid::Uuid,
    };

    let start_timer = Instant::now();

    let uuid = Uuid::new_v4();
    // let wait_store = true; // Probably better to make this an endpoint like /api/upload/ and /api/upload/awaited/;

    debug!(
        "Received new upload request from {addr}\nUsing id: {uuid}\nUsername: {}\nFile name: {}",
        "NO_USER", filename,
    );

    // Validation of user input
    if !FILENAME_VALIDATION_REGEX.is_match(filename) {
        error!("[{uuid}] The given filename doesn't match the validation regex");
        return Response::builder()
            .with_status(Status::BadRequest)
            .with_content("The specified filename should only contain alphanumeric characters, underscores, dots and shouldn't be longer than 100 characters")
            .with_content_type(ContentType::Text)
            .build();
    }

    // let data_stream = raw_data.open(unsafe{crate::FILE_REQ_SIZE_LIMIT});
    // File size check are done in the store data function in cache.rs
    let data_stream = raw_data.open(ByteUnit::max_value());

    let mut cache_handle = cache.write().await;

    let cache_entry = cache_handle.new_entry(
        uuid,
        crate::cache::data::UploadInfo::new(
            get_file_name(filename).unwrap_or_default(),
            get_file_extension(filename).unwrap_or_default(),
        ),
    );

    // Release the lock to let others use it
    drop(cache_handle);

    if let Err(e) = crate::cache::Cache::store(cache_entry, data_stream).await {
        error!("[{uuid}] An error occured while storing the given data: {e}");
        return Response::builder()
            .with_status(Status::InternalServerError)
            .with_content("An error occured while caching the data")
            .with_content_type(ContentType::Text)
            .build();
    };

    info!(
        "[{uuid}] Responded in {}",
        time::format(start_timer.elapsed(), 2)
    );

    Response::builder()
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
