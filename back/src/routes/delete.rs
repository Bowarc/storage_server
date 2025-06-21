/// This route is the main way to delete a cache
///
///     As input, it only requires the cache's uuid
///
///     It returns Ok if the deletion was sucessfull, the error otherwise
///
#[rocket::delete("/<uuidw>")]
pub async fn api_delete(
    uuidw: Option<super::UuidWrapper>,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,

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

    let entry = match cache.read().await.get_entry(uuid).await {
        Ok(entry) => entry,
        Err(e) => {
            error!("Failed to get entry due to: {e}");
            return Response::builder()
                .with_status(Status::InternalServerError)
                .build();
        }
    };

    if let Err(e) = crate::cache::Cache::delete(entry).await {
        error!("Failed to delete {uuid} due to: {e}");

        return Response::builder()
            .with_status(Status::InternalServerError)
            .build();
    };

    debug!("Successfully deleted {uuid}");

    Response::builder().build()
}
