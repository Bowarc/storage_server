use crate::error::CacheError;

#[rocket::get("/info/<uuidw>")]
pub async fn info(
    uuidw: super::download_route::UuidWrapper,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
) -> crate::response::Response {
    use crate::response::Response;
    use rocket::http::{ContentType, Status};

    let uuid = *uuidw;

    let entry = match cache.read().await.get_entry(uuid).await {
        Ok(entry) => entry,
        Err(CacheError::NotFound { .. }) => {
            return Response::builder()
                .with_status(Status::NotFound)
                .with_content(format!("Invalid id: {uuid}"))
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(e) => {
            error!("[{uuid}] Cache entry load error: {e}");
            return Response::builder()
                .with_status(Status::InternalServerError)
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    let json = match rocket::serde::json::serde_json::to_string(&*entry) {
        Ok(s) => s,
        Err(e) => {
            error!("[{uuid}] Failed to serialize due to: {e}");
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
