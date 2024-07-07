use rocket::{http::Status, serde::json::serde_json::json};

#[rocket::catch(400)]
pub async fn upload_400(_req: &rocket::Request<'_>) -> crate::response::JsonApiResponse {
    crate::response::JsonApiResponseBuilder::default()
        .with_json(json!({"status": 400, "message": "Could not understand the given data."}))
        .with_status(Status::BadRequest)
        .build()
}

#[rocket::catch(413)]
pub async fn upload_413(_req: &rocket::Request<'_>) -> crate::response::JsonApiResponse {
    info!("Sending reponse");
    crate::response::JsonApiResponseBuilder::default()
        .with_json(json!({"status": 413, "message": format!("Data too large, {} max", unsafe{crate::JSON_REQ_LIMIT})}))
        .with_status(Status::BadRequest)
        .build()
}

#[rocket::catch(403)]
pub async fn root_403() -> String {
    "403".to_string()
}
