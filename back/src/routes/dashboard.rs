#[rocket::get("/cache_list")]
pub async fn cache_list(
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
    remote_addr: std::net::SocketAddr,
) -> String {
    debug!("{remote_addr} has requested the cache list");
    let data = cache
        .read()
        .await
        .data
        .iter()
        .map(|cache_entry: &std::sync::Arc<shared::data::CacheEntry>| {
            rocket::serde::json::to_string(&**cache_entry).unwrap()
        })
        .collect::<Vec<String>>();
    format!("{data:?}")
}
