use crate::response::ResponseBuilder;

use {
    crate::response::Response,
    rocket::http::{ContentType, Status},
    std::net::SocketAddr,
};

#[path = "routes/dashboard.rs"]
mod dashboard_route;
#[path = "routes/download.rs"]
mod download_route;
#[path = "routes/upload.rs"] // Naming conflict in main when registering route
mod upload_route;

#[allow(unused_imports)] // Used by main.rs
pub use dashboard_route::*;
#[allow(unused_imports)] // Used by main.rs
pub use download_route::*;
use rocket::tokio::io::AsyncReadExt;
#[allow(unused_imports)] // Used by main.rs
pub use upload_route::*;

#[rocket::get("/")]
pub async fn root(remote_addr: SocketAddr) -> Response {
    let _old_msg = "

        Hi, please take a look at the /examples directory to understand how to use this api
    ";

    static_file_response("index.html", ContentType::HTML, remote_addr).await
}

#[rocket::get("/front.js")]
pub async fn front_js(remote_addr: SocketAddr) -> Response {
    static_file_response("/front.js", ContentType::JavaScript, remote_addr).await
}

#[rocket::get("/front_bg.wasm")]
pub async fn front_bg_wasm(remote_addr: SocketAddr) -> Response {
    static_file_response("/front_bg.wasm", ContentType::WASM, remote_addr).await
}

#[rocket::get("/index.html")]
pub async fn index_html(remote_addr: SocketAddr) -> Response {
    static_file_response("/index.html", ContentType::HTML, remote_addr).await
}

#[rocket::get("/favicon.ico")]
pub async fn favicon_ico(remote_addr: SocketAddr) -> Response {
    static_file_response("favicon.ico", ContentType::Icon, remote_addr).await
}

// The goal of this method, is to not use FileServer (because i wanna make sure of what file i serve)
// but i can't do #[rocket::get("/<file>")] as i want to use the get root path for the download api
#[rocket::get("/resources/<file>")]
pub async fn static_resource(file: &str, remote_addr: SocketAddr) -> Response {
    #[rustfmt::skip]
    const ALLOWED_FILES: &[&'static str] = &[
        "bash.webp", "c.webp", "cpp.webp",
        "csharp.webp", "css.webp", "git.webp",
        "github.webp", "html.webp", "java.webp",
        "javascript.webp", "kotlin.webp", "php.webp",
        "pwsh.webp", "pwsh2.webp", "python.webp",
        "rust.webp", "ssh.webp", "upload.png",
        "delete.png", "storage_server.drawio.png",
        "storage_server.drawio100px.png",
        "storage_server.drawio200px.png",
        "zig.webp"
    ];

    if !ALLOWED_FILES.contains(&file) {
        return ResponseBuilder::default()
            .with_status(Status::NotFound)
            .build();
    }

    serve_static("/resources", file, remote_addr).await
}

#[rocket::get("/css/<file>")]
pub async fn static_css(file: &str, remote_addr: SocketAddr) -> Response {
    const ALLOWED_FILES: &[&'static str] = &[
        "contact.css",
        "home.css",
        "upload.css",
        "notification.css",
        "style.css",
        "theme.css",
    ];

    if !ALLOWED_FILES.contains(&file) {
        return ResponseBuilder::default()
            .with_status(Status::NotFound)
            .build();
    }

    serve_static("/css", file, remote_addr).await
}

#[rocket::get("/lib/live/<file>")]
pub async fn static_lib_live(file: &str, remote_addr: SocketAddr) -> Response {
    const ALLOWED_FILES: &[&'static str] = &["live.js"];

    if !ALLOWED_FILES.contains(&file) {
        return ResponseBuilder::default()
            .with_status(Status::NotFound)
            .build();
    }

    serve_static("/lib/live", file, remote_addr).await
}

#[rocket::get("/lib/zoom/<file>")]
pub async fn static_lib_zoom(file: &str, remote_addr: SocketAddr) -> Response {
    const ALLOWED_FILES: &[&'static str] = &["zoom.js", "zoom.css"];

    if !ALLOWED_FILES.contains(&file) {
        return ResponseBuilder::default()
            .with_status(Status::NotFound)
            .build();
    }

    serve_static("/lib/zoom", file, remote_addr).await
}

pub async fn serve_static(path: &str, file: &str, remote_addr: SocketAddr) -> Response {
    #[inline]
    fn ext(file_name: &str) -> Option<&str> {
        if !file_name.contains(".") {
            return None;
        }

        let dot_index = file_name.rfind(".").unwrap();

        Some(&file_name[(dot_index + 1)..file_name.len()])
    }

    let content_type = ext(file)
        .and_then(ContentType::from_extension)
        .unwrap_or_else(|| {
            error!("Could not infer content type of file: {file}, requested in {path}");
            ContentType::Any
        });

    info!("Serving {path}/file w/ type: {content_type:?}");

    static_file_response(&format!("{path}/{file}"), content_type, remote_addr).await
}

async fn static_file_response(
    path: &str,
    content_type: ContentType,
    remote_addr: SocketAddr,
) -> Response {
    async fn read_static(path: &str, remote_addr: SocketAddr) -> Option<Vec<u8>> {
        let mut buffer = Vec::new();

        let size = rocket::tokio::fs::File::open(format!("./static/{path}"))
            .await
            .ok()?
            .read_to_end(&mut buffer)
            .await
            .ok()?;

        trace!("Static file query from {remote_addr}: {path} ({size} bytes)");
        Some(buffer)
    }

    match read_static(path, remote_addr).await {
        Some(bytes) => ResponseBuilder::default()
            .with_status(Status::Ok)
            .with_content(bytes)
            .with_content_type(content_type).build(),
        None => {
            return ResponseBuilder::default()
                .with_status(Status::NotFound)
                .build()
        }
    }
}

#[rocket::options("/upload")]
pub async fn upload_option() -> crate::response::JsonApiResponse {
    /*
        We're currently having issues connecting a NextJs server to this storage server

        we belive that his might help
        but we have no idea what to set here and in the NextJs config

        The thing is that test_upload (in front/main.rs) works fine, and do somewaht the same thing as the NextJs

        CORS errors..
    */
    warn!("option req");
    crate::response::JsonApiResponseBuilder::default()
        .with_status(Status::NoContent)
        .with_header("Content-Type", "text/plain")
        // .with_header("Access-Control-Allow-Origin", "*")
        // .with_header("Access-Control-Allow-Method", "POST")
        // .with_header("Access-Control-Allow-Headers", "X-PINGOTHER, Content-Type")
        // .with_header("Content-Type", "text/plain")
        // .with_header("Access-Control-Allow-Origin", "*")
        // .with_header("Access-Control-Allow-Cedentials", "true")
        // .with_header("Access-Control-Expose-Headers", "*")
        // .with_header("Access-Control-Max-Age", "5")
        // .with_header("Access-Control-Allow-Method", "POST,OPTIONS,GET")
        // .with_header(
        //     "Access-Control-Allow-Headers",
        //     "Content-Type",
        // )
        .build()
}
