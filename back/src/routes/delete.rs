/// This route is the main way to delete a cache
///
///     As input, it only requires the cache's uuid
///
///     It returns Ok if the deletion was sucessfull, the error otherwise
///
#[rocket::delete("/<uuidw>")]
pub async fn api_delete(
    uuidw: Option<super::UuidWrapper>,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::CacheEntryList>>,

    // See route::download's comment
    addr: rocket_client_addr::ClientAddr,
    method: rocket::http::Method,
    uri: &rocket::http::uri::Origin<'_>,
    c_type: Option<&rocket::http::ContentType>,
) -> crate::response::Response {
    use crate::response::Response;
    use rocket::http::Status;

    // wrong route
    let Some(uuidw) = uuidw else {
        let addr_string = addr
            .get_ipv4_string()
            .unwrap_or_else(|| addr.get_ipv6_string());
        return crate::catchers::inner_404(addr_string, method, uri, c_type).await;
    };

    let uuid = *uuidw;

    info!("[{addr}] DELETE request of {uuid}");

    let Some((index, entry)) = cache
        .read()
        .await
        .iter()
        .enumerate()
        .find(|(_, entry)| entry.uuid() == uuid)
        .map(|(index, entry)| (index, entry.clone()))
    else {
        error!("Could not find entry for {uuid}");
        return Response::builder()
            .with_status(Status::InternalServerError)
            .build();
    };

    if let Err(e) = entry.delete().await {
        error!("Failed to delete {uuid} due to: {e}");

        return Response::builder()
            .with_status(Status::InternalServerError)
            .build();
    };

    debug!("Successfully deleted {uuid}");

    cache.write().await.remove(index);

    Response::builder().build()
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

        let uuid = {
            // Setup
            let client = Client::tracked(build_rocket().await)
                .await
                .expect("valid rocket instance");

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

            info!("{}", rs);

            let suuid = rs.replace("Success: ", "");
            uuid::Uuid::from_str(&suuid).unwrap()
        };

        let client = Client::tracked(build_rocket().await)
            .await
            .expect("valid rocket instance");

        let response = client
            .delete(format!("/{uuid}", uuid = uuid.hyphenated()))
            .header(Header::new("x-forwarded-for", "0.0.0.0"))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }
}
