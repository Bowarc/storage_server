lazy_static! {
    // uuid v4
    pub static ref UUID_VALIDATION_REGEX: regex::Regex = regex::Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$"
    )
    .unwrap();
}

pub struct UuidWrapper(uuid::Uuid);

impl<'p> rocket::request::FromParam<'p> for UuidWrapper {
    type Error = crate::error::UuidParseError;

    fn from_param(param: &'p str) -> Result<Self, Self::Error> {
        use {crate::error::UuidParseError, std::str::FromStr, uuid::Uuid};

        if !UUID_VALIDATION_REGEX.is_match(param) {
            return Err(UuidParseError::Regex);
        }

        Ok(UuidWrapper(
            Uuid::from_str(param).map_err(|_e| UuidParseError::Convert)?,
        ))
    }
}

impl std::ops::Deref for UuidWrapper {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

///
/// This route is the main way to download a cache's content
///
///     As input, it only requires the cache's uuid
///
///     It returns, file cache's content, decompressed and in an directly usable format.
///         As for the filename, it sets the 'Content-Disposition' header for the browser to interpret
///
#[rocket::get("/<uuidw>")]
pub async fn api_download(
    uuidw: Option<UuidWrapper>,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::CacheEntryList>>,

    // About the optional uuidw and the ugly ton of params:
    //  The routing system in rocket works a bit weirdly, since you can only have 1
    //  wilcard per route level (you can't have two routes that have "/<something>" refering to two different types)
    //
    //  This, coupled with the fact that forcing a route to comform to a type, makes an useless warn! log when
    //  it doesn't match, makes it so i have to do the error part of the routing myself
    //
    //  So.. we have this monstruosity, where I need to query the error log infos with the happy path
    //  (and make my 404 route a two function thing)
    //
    //  this also have the effect to move the path to uuid conversion to the fromparam trait
    addr: rocket_client_addr::ClientAddr,
    method: rocket::http::Method,
    uri: &rocket::http::uri::Origin<'_>,
    c_type: Option<&rocket::http::ContentType>,
) -> crate::response::Response {
    use {
        crate::{error::CacheError, response::ResponseBuilder},
        rocket::http::{ContentType, Status},
        std::time::Instant,
    };

    let start_timer = Instant::now();

    let Some(uuidw) = uuidw else {
        let addr_string = addr
            .get_ipv4_string()
            .unwrap_or_else(|| addr.get_ipv6_string());
        return crate::catchers::inner_404(addr_string, method, uri, c_type).await;
    };

    let uuid = *uuidw;

    info!("[{addr}] DOWNLOAD request of {uuid}",);

    let Some(cache_entry) = cache
        .read()
        .await
        .iter()
        .find(|entry| entry.uuid() == uuid)
        .cloned()
    else {
        error!("[{uuid}] The given uuid doesn't correspnd to any cache entry");
        return ResponseBuilder::default()
            .with_status(Status::NotFound)
            .with_content("The given id doesn't correspond to any cache entry")
            .with_content_type(ContentType::Text)
            .build();
    };

    let (meta, data_stream) = match cache_entry.load() {
        Ok(meta_data) => meta_data,
        Err(CacheError::NotReady { uuid }) => {
            error!("[{uuid}] The requested cache is not ready yet");
            return ResponseBuilder::default()
                .with_status(Status::NotFound)
                .with_content("The requested cache is not ready yet")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(CacheError::FileOpen { file, why }) => {
            error!("[{uuid}] Failled to open data file ({file}) due to: {why}");
            return ResponseBuilder::default()
                .with_status(Status::InternalServerError)
                .with_content("Could not access given cache entry")
                .with_content_type(ContentType::Text)
                .build();
        }
        Err(CacheError::FileRead { file, why }) => {
            error!("[{uuid}] Failled to read {file} due to: {why}");
            return ResponseBuilder::default()
                .with_status(Status::InternalServerError)
                .with_content_type(ContentType::Text)
                .with_content("Could not acces given id's content")
                .build();
        }
        Err(e) => {
            error!("[{uuid}] Unexpected error: {e}");
            return ResponseBuilder::default()
                .with_status(Status::InternalServerError)
                .with_content("Could not access the requested cache. The error has been logged.")
                .with_content_type(ContentType::Text)
                .build();
        }
    };

    info!(
        "[{uuid}] Responded in {}",
        time::format(start_timer.elapsed(), 2)
    );

    ResponseBuilder::default()
        .with_status(Status::Ok)
        .with_content(data_stream)
        .with_content_type(ContentType::Binary)
        .with_header(
            "Content-Disposition",
            &format!(
                "attachment; filename=\"{}.{}\"",
                meta.name(),
                meta.extension()
            ),
        )
        .build()
}

///
/// This route is the seccond way to download a cache's content
///
///     As input, it requires the cache's uuid and the file name to be the right one
///
///     It returns, file cache's content, decompressed and in an directly usable format.
///
///     This route is a proxy and mostly for curl users to be able to use '-O' (download with auto file name)
///
#[rocket::get("/<uuidw>/<filename>")]
pub async fn api_download_filename(
    uuidw: UuidWrapper,
    filename: &str,
    cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::CacheEntryList>>,
    client_addr: rocket_client_addr::ClientAddr,

    // Ewww
    method: rocket::http::Method,
    uri: &rocket::http::uri::Origin<'_>,
    c_type: Option<&rocket::http::ContentType>,
) -> crate::response::Response {
    use {
        crate::response::ResponseBuilder,
        rocket::http::{ContentType, Status},
    };

    let resp = api_download(Some(uuidw), cache, client_addr, method, uri, c_type).await;

    if resp.status() != &Status::Ok {
        // If the internal call returned an error, there is no point doing the filename verification
        return resp;
    }

    let Some(content_disposition_header) = resp.headers().get("Content-Disposition") else {
        error!("Could not find the 'Content-Disposition' header of the api_download internal call");
        return ResponseBuilder::default()
            .with_status(Status::InternalServerError)
            .build();
    };

    // This won't truncate the real file name as quotes are not allowed in filename / extensions (see upload.rs::FILENAME_VALIDATION_REGEX)
    let header_filename = content_disposition_header
        .replace("attachment; filename=\"", "")
        .replace('"', "");

    if header_filename != filename {
        error!("The user supplied filename: '{filename}' but the one stored in metadata is '{header_filename}'");
        return ResponseBuilder::default()
            .with_status(Status::BadRequest)
            .with_content(format!(
                "Incorrect file name, did you meant '{header_filename}'?"
            ))
            .with_content_type(ContentType::Text)
            .build();
    }

    resp
}

// Not sure why it's there, it seems to return only the file name
// #[rocket::head("/<id>")]
// pub async fn api_download_head(
//     id: &str,
//     cache: &rocket::State<rocket::tokio::sync::RwLock<crate::cache::Cache>>,
// ) -> String {
//     use {std::str::FromStr, uuid::Uuid};

//     info!("Request of HEAD {id}");
//     let uuid = Uuid::from_str(id).unwrap();

//     let upload_info = cache.read().await.get_entry(uuid).await.unwrap();

//     format!(
//         "{}.{}",
//         upload_info.upload_info().name(),
//         upload_info.upload_info().extension()
//     )
// }

#[cfg(test)]
mod tests {
    use {
        crate::build_rocket,
        rocket::{http::Status, local::asynchronous::Client},
        std::str::FromStr,
    };

    #[rocket::async_test]
    async fn test_download() {
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
            .get(format!("/{uuid}", uuid = uuid.hyphenated()))
            .header(Header::new("x-forwarded-for", "0.0.0.0"))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.headers().get_one("Content-Disposition").unwrap(),
            format!("attachment; filename=\"{base_filename}\"")
        );
    }

    #[rocket::async_test]
    async fn test_download_filename() {
        use rocket::http::Header;
        let base_filename = "test.file";

        let uuid = {
            // Setup
            let client = Client::tracked(build_rocket().await)
                .await
                .expect("valid rocket instance");
            let response = client
                .put(format!("/{base_filename}"))
                .body("This is a co")
                .header(Header::new("x-forwarded-for", "0.0.0.0"))
                .dispatch()
                .await;

            #[allow(deprecated)] // stfu ik
            std::thread::sleep_ms(500);

            assert_eq!(response.status(), Status::Created);
            let suuid = response
                .into_string()
                .await
                .unwrap()
                .replace("Success: ", "");

            uuid::Uuid::from_str(&suuid).unwrap()
        };

        let client = Client::tracked(build_rocket().await)
            .await
            .expect("valid rocket instance");

        let response = client
            .get(format!("/{uuid}/{base_filename}", uuid = uuid.hyphenated()))
            .header(Header::new("x-forwarded-for", "0.0.0.0"))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.headers().get_one("Content-Disposition").unwrap(),
            format!("attachment; filename=\"{base_filename}\"")
        );
    }
}
