#[inline]
pub async fn inner_404(
    addr: String,
    method: rocket::http::Method,
    uri: &rocket::http::uri::Origin<'_>,
    c_type: Option<&rocket::http::ContentType>,
) -> crate::response::Response {
    use crate::response::Response;
    warn!(
        "[{addr}] has hit a 404 with {} at {} {}",
        method,
        uri,
        c_type.map(|t| format!("({t})")).unwrap_or_default()
    );
    Response::redirect(&rocket::uri!(crate::routes::_404).to_string())
}

#[rocket::catch(404)]
pub async fn root_404(req: &rocket::Request<'_>) -> super::response::Response {
    use rocket::{outcome::Outcome, request::FromRequest as _};
    use rocket_client_addr::ClientAddr;

    let addr_string = if let Outcome::Success(addr) = ClientAddr::from_request(req).await {
        addr.get_ipv4_string()
            .unwrap_or_else(|| addr.get_ipv6_string())
    } else {
        "UNKNOWN ADDRESS".to_string()
    };

    inner_404(addr_string, req.method(), req.uri(), req.content_type()).await
}

#[rocket::catch(400)]
pub fn upload_400(_req: &rocket::Request<'_>) -> crate::response::Response {
    use rocket::http::{ContentType, Status};

    crate::response::ResponseBuilder::default()
        .with_status(Status::PayloadTooLarge)
        .with_content("Could not understand the given data.")
        .with_content_type(ContentType::Text)
        .build()
}

#[rocket::catch(413)]
pub fn upload_413(_req: &rocket::Request<'_>) -> crate::response::Response {
    use rocket::http::{ContentType, Status};

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
