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

#[rocket::get("/css/contact.css")]
pub async fn contactCSS(remote_addr: SocketAddr) -> Response{
    file_response("/css/contact.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/gitcard.css")]
pub async fn gitcardCSS(remote_addr: SocketAddr) -> Response{
    file_response("/css/gitcard.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/home.css")]
pub async fn homeCSS(remote_addr: SocketAddr) -> Response{
    file_response("/css/home.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/presentation.css")]
pub async fn presentationCSS(remote_addr: SocketAddr) -> Response{
    file_response("/css/presentation.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/style.css")]
pub async fn styleCSS(remote_addr: SocketAddr) -> Response{
    file_response("/css/style.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/theme.css")]
pub async fn themeCSS(remote_addr: SocketAddr) -> Response{
    file_response("/css/theme.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/css/worms.css")]
pub async fn wormsCSS(remote_addr: SocketAddr) -> Response{
    file_response("/css/worms.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/front.js")]
pub async fn frontJS(remote_addr: SocketAddr) -> Response{
    file_response("/front.js", ContentType::JavaScript, remote_addr)
}

#[rocket::get("/front_bg.wasm")]
pub async fn front_bgWASM(remote_addr: SocketAddr) -> Response{
    file_response("/front_bg.wasm", ContentType::WASM, remote_addr)
}

#[rocket::get("/index.html")]
pub async fn indexHTML(remote_addr: SocketAddr) -> Response{
    file_response("/index.html", ContentType::HTML, remote_addr)
}

#[rocket::get("/lib/live/live.js")]
pub async fn liveJS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/live/live.js", ContentType::JavaScript, remote_addr)
}

#[rocket::get("/lib/prism/custom.css")]
pub async fn customCSS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/prism/custom.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/lib/prism/gruvbox-dark.css")]
pub async fn gruvboxdarkCSS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/prism/gruvbox-dark.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/lib/prism/prism-rust.min.js")]
pub async fn prism_rust_minJS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/prism/prism-rust.min.js", ContentType::JavaScript, remote_addr)
}

#[rocket::get("/lib/prism/prism.css")]
pub async fn prismCSS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/prism/prism.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/lib/prism/prism.min.js")]
pub async fn prism_minJS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/prism/prism.min.js", ContentType::JavaScript, remote_addr)
}

#[rocket::get("/lib/prism/synthwave84.css")]
pub async fn synthwave84CSS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/prism/synthwave84.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/lib/prism/xonokai.css")]
pub async fn xonokaiCSS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/prism/xonokai.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/lib/zoom/zoom.css")]
pub async fn zoomCSS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/zoom/zoom.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/lib/zoom/zoom.js")]
pub async fn zoomJS(remote_addr: SocketAddr) -> Response{
    file_response("/lib/zoom/zoom.js", ContentType::JavaScript, remote_addr)
}

#[rocket::get("/resources/bash.webp")]
pub async fn bashWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/bash.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/c.webp")]
pub async fn cWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/c.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/cpp.webp")]
pub async fn cppWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/cpp.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/csharp.webp")]
pub async fn csharpWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/csharp.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/css.webp")]
pub async fn cssWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/css.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/git.webp")]
pub async fn gitWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/git.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/github.webp")]
pub async fn githubWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/github.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/html.webp")]
pub async fn htmlWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/html.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/java.webp")]
pub async fn javaWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/java.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/javascript.webp")]
pub async fn javascriptWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/javascript.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/kotlin.webp")]
pub async fn kotlinWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/kotlin.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/php.webp")]
pub async fn phpWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/php.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/pwsh.webp")]
pub async fn pwshWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/pwsh.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/pwsh2.webp")]
pub async fn pwsh2WEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/pwsh2.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/python.webp")]
pub async fn pythonWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/python.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/rust.webp")]
pub async fn rustWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/rust.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/ssh.webp")]
pub async fn sshWEBP(remote_addr: SocketAddr) -> Response{
    file_response("/resources/ssh.webp", ContentType::WEBP, remote_addr)
}

#[rocket::get("/resources/storage_server.drawio.png")]
pub async fn storage_server_drawioPNG(remote_addr: SocketAddr) -> Response{
    file_response("/resources/storage_server.drawio.png", ContentType::PNG, remote_addr)
}

#[rocket::get("/resources/storage_server.drawio100px.png")]
pub async fn storage_server_drawio100pxPNG(remote_addr: SocketAddr) -> Response{
    file_response("/resources/storage_server.drawio100px.png", ContentType::PNG, remote_addr)
}

#[rocket::get("/resources/storage_server.drawio200px.png")]
pub async fn storage_server_drawio200pxPNG(remote_addr: SocketAddr) -> Response{
    file_response("/resources/storage_server.drawio200px.png", ContentType::PNG, remote_addr)
}

#[rocket::get("/resources/zig.webp")]
pub async fn zigWEBP(remote_addr: SocketAddr) -> Response{
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
    trace!("New static file query from {remote_addr}: {file_name}");
    let mut buffer = Vec::new();
    let _size = std::fs::File::open(format!("./static/{file_name}"))
        .ok()?
        .read_to_end(&mut buffer)
        .ok()?;
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
