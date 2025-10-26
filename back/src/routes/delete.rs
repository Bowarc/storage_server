/// This route is the main way to delete a cache
///
///     As input, it only requires the cache's uuid
///
///     It returns Ok if the deletion was sucessfull, the error otherwise
///
#[rocket::delete("/<uuidw>")]
pub async fn api_delete(
    uuidw: Option<super::UuidWrapper>,
    cache: &rocket::State<crate::cache::CacheEntryMap>,
    duplicate_map: &rocket::State<
        std::sync::Arc<rocket::tokio::sync::Mutex<crate::cache::DuplicateMap>>,
    >,

    // See route::api_download's comment
    addr: rocket_client_addr::ClientAddr,
    method: rocket::http::Method,
    uri: &rocket::http::uri::Origin<'_>,
    c_type: Option<&rocket::http::ContentType>,
) -> crate::response::Response {
    use crate::response::Response;
    use rocket::http::{ContentType, Status};

    // wrong route
    let Some(uuidw) = uuidw else {
        let addr_string = addr
            .get_ipv4_string()
            .unwrap_or_else(|| addr.get_ipv6_string());
        return crate::catchers::inner_404(addr_string, method, uri, c_type).await;
    };

    let uuid = *uuidw;

    info!("[{addr}] DELETE request of {uuid}");

    let Some((_uuid, entry)) = cache.remove(&uuid) else {
        error!("Could not find entry for {uuid}");
        return Response::builder()
            .with_status(Status::InternalServerError)
            .build();
    };

    if let Err(e) = entry.delete(std::sync::Arc::clone(duplicate_map)).await {
        error!("Failed to delete {uuid} due to: {e}");

        cache.insert(entry.uuid(), entry);

        return Response::builder()
            .with_status(Status::InternalServerError)
            .with_content_type(ContentType::Text)
            .with_content(format!("Failed to delete {uuid}"))
            .build();
    };

    debug!("Successfully deleted {uuid}");

    Response::builder()
        .with_status(Status::NoContent)
        .with_content_type(ContentType::Text)
        .with_content(format!("{uuid} has been successfully deleted"))
        .build()
}

#[cfg(test)]
mod tests {
    use {
        crate::build_rocket,
        rocket::{http::Status, local::asynchronous::Client},
        std::str::FromStr,
    };

    #[rocket::async_test]
    async fn test_delete() {
        use rocket::http::Header;
        let base_filename = "test.file";
        let client = Client::tracked(build_rocket().await)
            .await
            .expect("valid rocket instance");

        let uuid = {
            // Setup

            let response = client
                .put(format!("/{base_filename}"))
                .header(rocket::http::ContentType::Text)
                .header(Header::new("x-forwarded-for", "0.0.0.0"))
                .body("This is a cool file content")
                .dispatch()
                .await;

            #[allow(deprecated)] // stfu ik
            std::thread::sleep_ms(500);

            assert_eq!(response.status(), Status::Created);

            let rs = response.into_string().await.unwrap();

            let suuid = rs.replace("Success: ", "");
            uuid::Uuid::from_str(&suuid).unwrap()
        };

        let response = client
            .delete(format!("/{uuid}", uuid = uuid.hyphenated()))
            .header(Header::new("x-forwarded-for", "0.0.0.0"))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NoContent);
    }
}
