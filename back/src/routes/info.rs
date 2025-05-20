use crate::error::CacheError;

#[rocket::get("/info/<id>")]
pub async fn info(
    id: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> crate::response::Response {
    use crate::{error::UuidParseError, response::Response};
    use rocket::http::{ContentType, Status};

    let uuid = match super::download_route::parse_id(id) {
        Ok(uuid) => uuid,
        Err(UuidParseError::Regex) => {
            error!("[{id}] Given id doesn't match expected character range");
            return Response::builder()
                .with_status(Status::BadRequest)
                .with_content("Given id doesn't match expected character range")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(UuidParseError::Convert) => {
            error!("[{id}] Invalid id");
            return Response::builder()
                .with_status(Status::BadRequest)
                .with_content(format!("Invalid id: {id}"))
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    let entry = match cache.read().await.get_entry(uuid).await {
        Ok(entry) => entry,
        Err(CacheError::NotFound { .. }) => {
            return Response::builder()
                .with_status(Status::NotFound)
                .with_content(format!("Invalid id: {id}"))
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(e) => {
            error!("[{id}] Cache entry load error: {e}");
            return Response::builder()
                .with_status(Status::InternalServerError)
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    let json = match rocket::serde::json::serde_json::to_string(&*entry) {
        Ok(s) => s,
        Err(e) => {
            error!("[{id}] Failed to serialize due to: {e}");
            return Response::builder()
                .with_status(Status::InternalServerError)
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    Response::builder()
        .with_content(json)
        .with_content_type(ContentType::JSON)
        .build()
}
