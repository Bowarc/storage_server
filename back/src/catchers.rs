use rocket::{
    http::{ContentType, Status},
    serde::json::serde_json::json,
};

#[rocket::catch(400)]
pub fn upload_400(_req: &rocket::Request<'_>) -> crate::response::Response {
    crate::response::ResponseBuilder::default()
        .with_status(Status::PayloadTooLarge)
        .with_content("Could not understand the given data.")
        .with_content_type(ContentType::Text)
        .build()
}

#[rocket::catch(413)]
pub fn upload_413(_req: &rocket::Request<'_>) -> crate::response::Response {
    crate::response::ResponseBuilder::default()
        .with_status(Status::PayloadTooLarge)
        .with_content(format!("Data too large, {} max", unsafe {
            crate::FILE_REQ_SIZE_LIMIT
        }))
        .with_content_type(ContentType::Text)
        .build()
}

#[rocket::catch(403)]
pub fn root_403() -> String {
    "403".to_string()
}
