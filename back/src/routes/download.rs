use {
    crate::response::{JsonApiResponse, JsonApiResponseBuilder},
    rocket::{http::Status, serde::json::serde_json::json},
    std::str::FromStr,
};

#[rocket::get("/download/<id>")]
pub async fn download(
    id: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> JsonApiResponse {
    debug!("Download request of: {id}");

    // Only contains numbers, lowercase letters or dashes
    if !id
        .chars()
        .all(|c| c.is_digit(10) || c == '-' || c.is_ascii_lowercase())
        || id.len() != /* uuid default length */ 37
    {
        error!("Given id doesn't match expected character range");
        return JsonApiResponseBuilder::default()
            .with_status(Status::BadRequest)
            .with_json(json!({
                "result": "denied",
                "message": "Given id doesn't match expected character range"
            }))
            .build();
    }

    let Ok(id) = uuid::Uuid::from_str(id) else {
        error!("Could not understand given id: {id}");
        return JsonApiResponseBuilder::default()
            .with_json(json!( {
                "result": "denied",
                "message": format!("could not understand given id: {id}")
            }))
            .with_status(Status::BadRequest)
            .build();
    };
    let (meta, data) = match cache.read().await.load(id).await {
        Ok(meta_data) => meta_data,
        Err(e) => {
            error!("[{id}] Could not load cache due to: {e}");
            return JsonApiResponseBuilder::default()
                .with_json(json!({
                    "result": "failled",
                    "message": format!("Id not found")
                }))
                .with_status(Status::BadRequest)
                .build();
        }
    };

    // let data_b64 = String::from_utf8(data).unwrap();
    let data_b64 = rbase64::encode(&data);

    JsonApiResponseBuilder::default()
        .with_json(json!({
            "metadata": meta,
            "file": data_b64
        }))
        .with_status(Status::Ok)
        .build()
}
