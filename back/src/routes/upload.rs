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
    cache: &rocket::State<crate::cache::CacheEntryMap>,
    duplicate_map: &rocket::State<
        std::sync::Arc<rocket::tokio::sync::Mutex<crate::cache::DuplicateMap>>,
    >,
    addr: rocket_client_addr::ClientAddr,
) -> crate::response::Response {
    use {
        crate::{cache::CacheEntry, response::Response},
        rocket::{
            data::ByteUnit,
            http::{ContentType, Status},
        },
        std::time::Instant,
        uuid::Uuid,
    };

    let start_timer = Instant::now();

    let uuid = loop {
        let uuid = Uuid::new_v4();
        if cache.get(&uuid).is_none() {
            break uuid;
        }
    };

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

    // File size check are done in the store data function in cache.rs
    let data_stream = raw_data.open(ByteUnit::max_value());

    let entry = match CacheEntry::store_new(
        uuid,
        crate::cache::UploadInfo::new(
            get_file_name(filename).unwrap_or_default(),
            get_file_extension(filename).unwrap_or_default(),
        ),
        data_stream,
        std::sync::Arc::clone(duplicate_map),
    )
    .await
    {
        Ok(entry) => entry,
        Err(e) => {
            error!("[{uuid}] An error occured while storing the given data: {e}");
            return Response::builder()
                .with_status(Status::InternalServerError)
                .with_content("An error occured while caching the data")
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    let x = cache.insert(entry.uuid(), entry);
    if x.is_some() {
        for _ in 0..10 {
            error!("uuid conflict please fix: {uuid}");
        }
    }

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
        return Some(name.to_string());
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
        rocket::{
            http::{Header, Status},
            local::asynchronous::Client,
        },
        std::str::FromStr,
    };

    #[rocket::async_test]
    async fn test_upload() {
        let client = Client::tracked(build_rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client
            .put("/test.file")
            .body("This is normal file content")
            .header(Header::new("x-forwarded-for", "0.0.0.0"))
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
