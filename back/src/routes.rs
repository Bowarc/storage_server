#[path = "routes/delete.rs"]
mod delete_route;
#[path = "routes/download.rs"]
mod download_route;
#[path = "routes/info.rs"]
mod info_route;
#[path = "routes/upload.rs"] // Naming conflict in main when registering route
mod upload_route;

#[allow(unused_imports)] // Used by main.rs
pub use delete_route::*;
#[allow(unused_imports)] // Used by main.rs
pub use download_route::*;
#[allow(unused_imports)] // Used by main.rs
pub use info_route::*;
#[allow(unused_imports)] // Used by main.rs
pub use upload_route::*;

// Here are routes that are managed by the front end router, so just serve the page and let it do its things
macro_rules! front_route {
    // Here I could match on list of tokens with $($path:..)
    // But I prefer keeping things simple
    ($name:ident, $path:literal) => {
        #[rocket::get($path)]
        pub async fn $name(ip_addr: rocket_client_addr::ClientAddr) -> super::response::Response {
            root(ip_addr).await
        }
    };
}

front_route!(home, "/home");
front_route!(upload, "/upload");
front_route!(contact, "/contact");
front_route!(_404, "/404");

#[rocket::get("/")]
pub async fn root(ip_addr: rocket_client_addr::ClientAddr) -> super::response::Response {
    use rocket::http::ContentType;

    static_file_response("index.html", ContentType::HTML, ip_addr, false).await
}

#[rocket::get("/front.js")]
pub async fn front_js(ip_addr: rocket_client_addr::ClientAddr) -> super::response::Response {
    use rocket::http::ContentType;

    static_file_response("/front.js", ContentType::JavaScript, ip_addr, true).await
}

#[rocket::get("/front_bg.wasm")]
pub async fn front_bg_wasm(ip_addr: rocket_client_addr::ClientAddr) -> super::response::Response {
    use rocket::http::ContentType;

    static_file_response("/front_bg.wasm", ContentType::WASM, ip_addr, true).await
}

#[rocket::get("/index.html")]
pub async fn index_html(ip_addr: rocket_client_addr::ClientAddr) -> super::response::Response {
    use rocket::http::ContentType;

    static_file_response("/index.html", ContentType::HTML, ip_addr, false).await
}

#[rocket::get("/favicon.ico")]
pub async fn favicon_ico(ip_addr: rocket_client_addr::ClientAddr) -> super::response::Response {
    use rocket::http::ContentType;

    static_file_response("favicon.ico", ContentType::Icon, ip_addr, true).await
}

// The goal of this method is to not use rocket's FileServer
// because I wanna make sure of what file I allow serving
macro_rules! static_dir_server {
    ($path:literal, $dir:literal, $func_name:ident, $allowed_files:expr) => {
        #[rocket::get($path)]
        pub async fn $func_name(
            file: &str,
            ip_addr: rocket_client_addr::ClientAddr,
        ) -> super::response::Response {
            use super::response::Response;
            use rocket::http::Status;

            const ALLOWED_FILES: &[&str] = $allowed_files;

            if !ALLOWED_FILES.contains(&file) {
                return Response::builder().with_status(Status::NotFound).build();
            }

            serve_static(concat!("/", $dir), file, ip_addr, true).await
        }
    };
}

static_dir_server!(
    "/css/<file>",
    "css",
    static_css,
    &[
        "contact.css",
        "upload.css",
        "notification.css",
        "home.css",
        "light_switch.css",
        "style.css",
        "theme.css",
        "not_found.css",
    ]
);
static_dir_server!(
    "/resources/<file>",
    "resources",
    static_resource,
    &[
        "delete.png",
        "upload.png",
        "github.webp",
        "storage_server.drawio.png",
        "storage_server.drawio100px.png",
        "storage_server.drawio200px.png",
    ]
);

// Serve a static file
// This assumes that the file is allowed to be served
pub async fn serve_static(
    path: &str,
    file: &str,
    ip_addr: rocket_client_addr::ClientAddr,
    cache: bool,
) -> super::response::Response {
    use rocket::http::ContentType;

    #[inline]
    fn ext(file_name: &str) -> Option<&str> {
        if !file_name.contains(".") {
            return None;
        }

        let dot_index = file_name.rfind(".").unwrap();

        Some(&file_name[(dot_index + 1)..file_name.len()])
    }

    // Try to build content type using the file extension
    let content_type = ext(file)
        .and_then(ContentType::from_extension)
        .unwrap_or_else(|| {
            error!("Could not infer content type of file: {file}, requested in {path}");
            ContentType::Any
        });

    // Serve local file
    static_file_response(&format!("{path}/{file}"), content_type, ip_addr, cache).await
}

async fn static_file_response(
    path: &str,
    content_type: rocket::http::ContentType,
    ip_addr: rocket_client_addr::ClientAddr,
    cache: bool,
) -> super::response::Response {
    use super::response::Response;
    use rocket::http::Status;
    use tokio::fs::File;

    match File::open(format!("./static/{path}")).await {
        Ok(file) => {
            // trace!("Static file query from {ip_addr}: {path}");
            let mut response = Response::builder()
                .with_status(Status::Ok)
                .with_content(file)
                .with_content_type(content_type);

            if cache {
                response = response.with_header("Cache-Control", "max-age=3600")
                // Ask the browser to cache the request for 1 hour, might help for server load
            }

            response.build()
        }
        Err(e) => {
            warn!("Static file query from {ip_addr}: {path} failed due to: {e}");
            Response::builder().with_status(Status::NotFound).build()
        }
    }
}
