#[rocket::get("/cache_list")]
pub async fn _cache_list(
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
    remote_addr: std::net::SocketAddr,
) -> String {
    debug!("{remote_addr} has requested the cache list");
    let data = cache
        .read()
        .await
        .inner
        .iter()
        .map(|cache_entry: &std::sync::Arc<crate::cache::data::CacheEntry>| {
            rocket::serde::json::to_string(&**cache_entry).unwrap()
        })
        .collect::<Vec<String>>();
    format!("{data:?}")
}
