use rocket::http::ContentType;

use crate::response::{Response, ResponseBuilder};

use {
    crate::response::{JsonApiResponse, JsonApiResponseBuilder},
    rocket::{http::Status, serde::json::serde_json::json},
};

lazy_static! {
    static ref EXTENSION_VALIDATION_REGEX: regex::Regex =
        regex::Regex::new(r"^[A-Za-z0-9_.]{1,100}$").unwrap();
}

#[rocket::put("/<filename>", data = "<raw_data>")]
pub async fn api_upload(
    filename: &str,
    raw_data: rocket::data::Data<'_>,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> Response {
    use {std::time::Instant, uuid::Uuid};
    let start_timer = Instant::now();

    let id = Uuid::new_v4();
    let wait_store = true; // Probably betterto make this an endpoint like /api/upload/ and /api/upload/awaited/;

    // Validation of user input
    if !EXTENSION_VALIDATION_REGEX.is_match(&filename) {
        return ResponseBuilder::default()
            .with_status(Status::BadRequest)
            .with_content("The specified filename should only contain alphanumeric characters, underscores, dots and shouldn't be longer than 100 characters")
            .with_content_type(ContentType::Text)
            .build();
    }

    let capped_data = match raw_data
        .open(unsafe { crate::JSON_REQ_LIMIT })
        .into_bytes()
        .await
    {
        Ok(data) => data,
        Err(e) => {
            error!("[{id}] Could not parse given data: {e}");

            return ResponseBuilder::default()
                .with_status(Status::BadRequest)
                .with_content("The given body content could not be parsed")
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    if !capped_data.is_complete() {
        error!("Data too large");
        return ResponseBuilder::default()
            .with_status(Status::PayloadTooLarge)
            .with_content("The given data is too large")
            .with_content_type(ContentType::Text)
            .build();
    }

    let data = capped_data.to_vec();

    debug!(
        "Received new upload request on /json\nUsing id: {id}\nUsername: {}\nFile name: {}\nFile size: {}",
        "NO_USER",
        filename,
        data.len()
    );

    // No need to decode user input as it's not b64 encoded anymore

    let mut cache_handle = cache.write().await;

    let exec = cache_handle.store(
        id,
        shared::data::Metadata {
            username: "NO_USER".to_string(),
            file_name: get_file_name(filename).unwrap_or_default(),
            file_ext: get_file_extension(filename).unwrap_or_default(),
        },
        data,
    );

    // Release the lock to be able to wait the end of the 'exec' without denying other calls
    drop(cache_handle);

    if wait_store {
        debug!("[{id}] Waiting for cache to finish storing the data");

        match exec.await {
            Ok(Ok(())) => {
                // All good
            }
            Ok(Err(e)) => {
                error!("[{id}] An error occured while storing the given data: {e}");
                return ResponseBuilder::default()
                    .with_status(Status::InternalServerError)
                    .with_content("An error occured while caching the data")
                    .with_content_type(ContentType::Text)
                    .build();
            }
            Err(join_error) => {
                error!("[{id}] Something went really bad while waiting for worker task to end: {join_error}");
                return ResponseBuilder::default()
                    .with_status(Status::InternalServerError)
                    .with_content("Worker failled")
                    .with_content_type(ContentType::Text)
                    .build();
            }
        }
    }

    info!(
        "[{id}] Responded in {}",
        time::format(start_timer.elapsed(), 2)
    );
    ResponseBuilder::default()
        .with_status(Status::Created)
        .with_content(id.hyphenated().to_string())
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
    async fn test_upload_PUT() {
        let client = Client::tracked(build_rocket().await)
            .await
            .expect("valid rocket instance");
        let response = client
            .put("/test.file")
            .header(rocket::http::ContentType::Plain)
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
