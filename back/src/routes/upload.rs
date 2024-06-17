use {
    crate::response::{JsonApiResponse, JsonApiResponseBuilder},
    rocket::{http::Status, serde::json::serde_json::json},
};

#[rocket::post("/upload", format = "application/json", data = "<data>")]
pub async fn upload(
    data: rocket::serde::json::Json<shared::data::UploadData>,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> JsonApiResponse {
    // Setup
    let start_timer = std::time::Instant::now();
    let id = uuid::Uuid::new_v4();
    let metadata = data.metadata.clone();
    let file_data = &data.file;
    let wait_store = true; // Probably better to make this an endpoint like /upload/ and /upload/awaited/

    // Validation of user input
    if !regex::Regex::new(r"^[A-Za-z0-9]*$")
        .unwrap() // Should not fail
        .is_match(&metadata.file_ext)
    {
        return JsonApiResponseBuilder::default()
            .with_json(json!({"result": "denied", "message": "The specified extension should only contain alphanumeric characters"}))
            .with_status(Status::BadRequest).build();
    }

    debug!(
        "Received new upload request on /json\nUsing id: {id}\nUsername: {}\nFile name: {}\nFile ext: {}\nFile size: {}",
        metadata.username,
        metadata.file_name,
        metadata.file_ext,
        rocket::data::ByteUnit::Byte(file_data.len() as u64)
    );

    // Decode user input | Decoding makes the compression 'faster' koz it has less data to compress
    // let file_content = file_data.clone().into_bytes();
    let Ok(file_content) = rbase64::decode(file_data) else {
        error!("[{id}] Could not decode request");
        return JsonApiResponseBuilder::default()
            .with_json(
                json!({"result": "failled", "message": "Could not understand the given data."}),
            )
            .with_status(Status::BadRequest)
            .build();
    };

    let mut cache_handle = cache.write().await;

    let exec = cache_handle.store(id, metadata, file_content);

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
                return JsonApiResponseBuilder::default()
                    .with_json(
                        json!({"result": "failled", "message": "An error occured while caching the data"}),
                    )
                    .with_status(Status::InternalServerError)
                    .build();
            }
            Err(join_error) => {
                error!("[{id}] Something went really bad while waiting for worker task to end: {join_error}");
                return JsonApiResponseBuilder::default()
                    .with_json(json!({"result": "failled", "message": "Worker failled"}))
                    .with_status(Status::InternalServerError)
                    .build();
            }
        }
    }

    info!(
        "[{id}] Responded in {}",
        time::format(start_timer.elapsed(), 2)
    );

    JsonApiResponseBuilder::default()
        .with_json(json!({"result": "created", "id": id.hyphenated().to_string()}))
        .with_status(Status::Created)
        .build()
}
