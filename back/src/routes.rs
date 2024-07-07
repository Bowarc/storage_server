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
#[allow(unused_imports)] // Used by main.rs
pub use upload_route::*;

#[rocket::get("/")]
pub async fn root(remote_addr: SocketAddr) -> Response {
    let _old_msg = "

        Hi, please take a look at the /examples directory to understand how to use this api
    ";

    file_response("index.html", ContentType::HTML, remote_addr)
}

#[rocket::get("/favicon.ico")]
pub async fn favicon_ico(remote_addr: SocketAddr) -> Response {
    file_response("favicon.ico", ContentType::Icon, remote_addr)
}

#[rocket::get("/css/contact.css")]
pub async fn contact_css(remote_addr: SocketAddr) -> Response {
    file_response("/css/contact.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/home.css")]
pub async fn home_css(remote_addr: SocketAddr) -> Response {
    file_response("/css/home.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/upload.css")]
pub async fn upload_css(remote_addr: SocketAddr) -> Response {
    file_response("/css/upload.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/notification.css")]
pub async fn notification_css(remote_addr: SocketAddr) -> Response {
    file_response("/css/notification.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/style.css")]
pub async fn style_css(remote_addr: SocketAddr) -> Response {
    file_response("/css/style.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/theme.css")]
pub async fn theme_css(remote_addr: SocketAddr) -> Response {
    file_response("/css/theme.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/front.js")]
pub async fn front_js(remote_addr: SocketAddr) -> Response {
    file_response("/front.js", ContentType::JavaScript, remote_addr)
}

#[rocket::get("/front_bg.wasm")]
pub async fn front_bg_wasm(remote_addr: SocketAddr) -> Response {
    file_response("/front_bg.wasm", ContentType::WASM, remote_addr)
}

#[rocket::get("/index.html")]
pub async fn index_html(remote_addr: SocketAddr) -> Response {
    file_response("/index.html", ContentType::HTML, remote_addr)
}

#[rocket::get("/lib/live/live.js")]
pub async fn live_js(remote_addr: SocketAddr) -> Response {
    file_response("/lib/live/live.js", ContentType::JavaScript, remote_addr)
}

#[rocket::get("/lib/zoom/zoom.css")]
pub async fn zoom_css(remote_addr: SocketAddr) -> Response {
    file_response("/lib/zoom/zoom.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/lib/zoom/zoom.js")]
pub async fn zoom_js(remote_addr: SocketAddr) -> Response {
    file_response("/lib/zoom/zoom.js", ContentType::JavaScript, remote_addr)
}

#[rocket::get("/resources/bash.webp")]
pub async fn bash_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/bash.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/c.webp")]
pub async fn c_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/c.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/cpp.webp")]
pub async fn cpp_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/cpp.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/csharp.webp")]
pub async fn csharp_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/csharp.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/css.webp")]
pub async fn css_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/css.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/git.webp")]
pub async fn git_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/git.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/github.webp")]
pub async fn github_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/github.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/html.webp")]
pub async fn html_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/html.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/java.webp")]
pub async fn java_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/java.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/javascript.webp")]
pub async fn javascript_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/javascript.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/kotlin.webp")]
pub async fn kotlin_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/kotlin.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/php.webp")]
pub async fn php_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/php.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/pwsh.webp")]
pub async fn pwsh_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/pwsh.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/pwsh2.webp")]
pub async fn pwsh2_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/pwsh2.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/python.webp")]
pub async fn python_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/python.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/rust.webp")]
pub async fn rust_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/rust.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/ssh.webp")]
pub async fn ssh_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/ssh.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/upload.png")]
pub async fn upload_png(remote_addr: SocketAddr) -> Response {
    file_response("/resources/upload.png", ContentType::PNG, remote_addr)
}

#[rocket::get("/resources/delete.png")]
pub async fn delete_png(remote_addr: SocketAddr) -> Response {
    file_response("/resources/delete.png", ContentType::PNG, remote_addr)
}

#[rocket::get("/resources/storage_server.drawio.png")]
pub async fn storage_server_drawio_png(remote_addr: SocketAddr) -> Response {
    file_response(
        "/resources/storage_server.drawio.png",
        ContentType::PNG,
        remote_addr,
    )
}

#[rocket::get("/resources/storage_server.drawio100px.png")]
pub async fn storage_server_drawio100px_png(remote_addr: SocketAddr) -> Response {
    file_response(
        "/resources/storage_server.drawio100px.png",
        ContentType::PNG,
        remote_addr,
    )
}

#[rocket::get("/resources/storage_server.drawio200px.png")]
pub async fn storage_server_drawio200px_png(remote_addr: SocketAddr) -> Response {
    file_response(
        "/resources/storage_server.drawio200px.png",
        ContentType::PNG,
        remote_addr,
    )
}

#[rocket::get("/resources/zig.webp")]
pub async fn zig_webp(remote_addr: SocketAddr) -> Response {
    file_response("/resources/zig.webp", ContentType::WEBP, remote_addr)
}

fn file_response(file_name: &str, content_type: ContentType, remote_addr: SocketAddr) -> Response {
    match read_static(file_name, remote_addr) {
        Some(bytes) => Response {
            status: Status::Ok,
            content: bytes,
            content_type: content_type,
        },
        None => Response {
            status: Status::InternalServerError,
            content: Vec::new(),
            content_type: ContentType::Plain,
        },
    }
}

fn read_static(file_name: &str, remote_addr: SocketAddr) -> Option<Vec<u8>> {
    use std::io::Read as _;
    let mut buffer = Vec::new();
    let size = std::fs::File::open(format!("./static/{file_name}"))
        .ok()?
        .read_to_end(&mut buffer)
        .ok()?;
    trace!("Static file query from {remote_addr}: {file_name} ({size} bytes)");
    Some(buffer)
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
