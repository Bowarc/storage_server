use rocket::{http::ContentType, response::Redirect, tokio::io::AsyncReadExt, uri};
use uuid::Uuid;

use crate::response::{Response, ResponseBuilder};

use {
    crate::response::{JsonApiResponse, JsonApiResponseBuilder},
    rocket::{http::Status, serde::json::serde_json::json},
    std::str::FromStr,
};

lazy_static! {
    static ref EXTENSION_VALIDATION_REGEX: regex::Regex =
        regex::Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$").unwrap();
}

#[rocket::get("/api/download/<id>")]
pub async fn api_download(
    id: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> JsonApiResponse {
    debug!("Download request of: {id}");

    // Only contains numbers, lowercase letters or dashes
    if !EXTENSION_VALIDATION_REGEX.is_match(&id) {
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
        error!("Invalid id: {id}");
        return JsonApiResponseBuilder::default()
            .with_json(json!( {
                "result": "denied",
                "message": format!("Invalid id: {id}")
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

#[rocket::get("/<id>")]
pub async fn api_download_get_proxy(
    id: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> rocket::response::Redirect {
    info!("Proxy request of {id}");

    let uuid = Uuid::from_str(id).unwrap();

    let meta = cache.write().await.get_meta(uuid).await.unwrap();

    let filename = format!("{}.{}", meta.file_name, meta.file_ext);
    info!("Redirecting to {filename}");

    Redirect::to(uri!(api_download_get(id, filename)))
}

#[rocket::get("/<id>/<filename>")]
pub async fn api_download_get(
    id: &str,
    filename: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> Response {
    info!("Request of {id} w/ filename: {filename}");

    let uuid = Uuid::from_str(id).unwrap();

    let (meta, data) = cache.write().await.load(uuid).await.unwrap();

    if &format!("{}.{}", meta.file_name, meta.file_ext) != filename {
        return ResponseBuilder::default()
            .with_status(Status::BadRequest)
            .with_content(format!(
                "The given filename is not correct, did you meant {}.{}?",
                meta.file_name, meta.file_ext
            ))
            .with_content_type(ContentType::Plain)
            .build();
    }

    ResponseBuilder::default()
        .with_status(Status::Ok)
        .with_content(data)
        .with_content_type(ContentType::Bytes).build()
}
#[rocket::head("/<id>")]
pub async fn api_download_head(
    id: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> String {
    info!("Request of HEAD {id}");
    let uuid = Uuid::from_str(id).unwrap();

    let meta = cache.write().await.get_meta(uuid).await.unwrap();

    format!("{}.{}", meta.file_name, meta.file_ext)
}

#[cfg(test)]
mod tests {
    use {
        crate::build_rocket,
        rocket::{http::Status, local::asynchronous::Client},
        std::str::FromStr,
    };

    #[rocket::async_test]
    async fn test_download_proxy_GET() {
        let base_filename = "test.file";

        let uuid = {
            // Setup
            let client = Client::tracked(build_rocket().await)
                .await
                .expect("valid rocket instance");

            let response = client
                .put(format!("/{base_filename}"))
                .header(rocket::http::ContentType::Plain)
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
            let uuid = uuid::Uuid::from_str(&suuid).unwrap();
            uuid
        };

        let client = Client::tracked(build_rocket().await)
            .await
            .expect("valid rocket instance");

        let response = client
            .get(format!("/{uuid}", uuid = uuid.hyphenated()))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::SeeOther); // Redirect
        assert_eq!(
            response.headers().get_one("location").unwrap(),
            format!("/{uuid}/{base_filename}")
        );
    }

    #[rocket::async_test]
    async fn test_download_GET() {
        let logcfg = logger::LoggerConfig::new()
            .set_level(log::LevelFilter::Trace)
            .add_filter("rocket", log::LevelFilter::Warn);
        logger::init(logcfg, None);

        let base_filename = "test.file";

        let uuid = {
            // Setup
            let client = Client::tracked(build_rocket().await)
                .await
                .expect("valid rocket instance");
            let response = client
                .put(format!("/{base_filename}"))
                .header(rocket::http::ContentType::Plain)
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
            let uuid = uuid::Uuid::from_str(&suuid).unwrap();
            uuid
        };
    }
}
