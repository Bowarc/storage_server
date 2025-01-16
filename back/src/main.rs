#[allow(unused_imports)]
#[macro_use]
extern crate thiserror;

#[macro_use(trace, debug, info, warn, error)]
extern crate log;

#[macro_use(lazy_static)]
extern crate lazy_static;

mod cache;
mod catchers;
mod error;
mod response;
mod routes;

static mut FILE_REQ_SIZE_LIMIT: rocket::data::ByteUnit = rocket::data::ByteUnit::Byte(0);

// Needed for tests
pub async fn build_rocket() -> rocket::Rocket<rocket::Ignite> {
    let Some(cache) = cache::Cache::new() else {
        error!("Failled to load cache");
        std::process::exit(1)
    };

    let rocket = rocket::build()
        .manage(rocket::tokio::sync::RwLock::new(cache))
        .register("/", rocket::catchers![catchers::root_403])
        .register(
            "/upload",
            rocket::catchers![catchers::upload_400, catchers::upload_413],
        )
        .mount(
            "/",
            rocket::routes![
                routes::root,
                routes::front_js,
                routes::front_bg_wasm,
                routes::index_html,
                routes::static_resource,
                routes::static_css,
                routes::static_lib_live,
                routes::static_lib_zoom,
                routes::favicon_ico,
                routes::api_upload,
                routes::api_download,
                routes::api_download_stream,
                routes::api_download_filename,
                routes::api_download_head,
            ],
        )
        .ignite()
        .await
        .unwrap();

    // Safety:
    //  This will only be writen once and at the reads are not yet loaded because the sever is not yet launched
    unsafe {
        let Some(limit) = rocket.config().limits.get("file") else{
            panic!("Failed to retrieve the file limit from the config");
        };
        FILE_REQ_SIZE_LIMIT = limit;
    }

    rocket
}

#[rocket::main]
async fn main() {
    let filters = [("rocket", log::LevelFilter::Warn)];
    logger::init([
        logger::Config::default()
            .output(logger::Output::Stdout)
            .colored(true)
            .filters(&filters),
        logger::Config::default()
            .output(logger::Output::new_timed_file(
                "./log/.log",
                std::time::Duration::from_secs(86400), // A day
            ))
            .filters(&filters),
    ]);

    // Small print to show the start of the program log in the file
    trace!(
        "\n╭{line}╮\n│{message:^30}│\n╰{line}╯",
        line = "─".repeat(30),
        message = "Program start"
    );

    let rocket = build_rocket().await;

    display_config(rocket.config(), rocket.routes(), rocket.catchers());

    rocket.launch().await.unwrap();
}

/// Displays the config in the console
fn display_config<'a>(
    rocket_cfg: &rocket::Config,
    rocket_routes: impl Iterator<Item = &'a rocket::Route>,
    rocket_catchers: impl Iterator<Item = &'a rocket::Catcher>,
) {
    let profile = rocket_cfg.profile.as_str().as_str();
    let address = rocket_cfg.address;
    let port = rocket_cfg.port;
    let workers = rocket_cfg.workers;
    // let max_blocking = cfg.max_blocking;
    let indent = rocket_cfg.ident.as_str().unwrap_or("[ERROR] Undefined");
    let ip_headers = rocket_cfg
        .ip_header
        .as_ref()
        .map(|header| header.as_str())
        .unwrap_or("[ERROR] Undefined");
    let limits = ["bytes", "data-form", "file", "json", "msgpack", "string"]
        .iter()
        .map(|limit_name| {
            format!(
                "{limit_name}: {}",
                rocket_cfg
                    .limits
                    .get(limit_name)
                    .unwrap_or(rocket::data::ByteUnit::from(0))
            )
        })
        .collect::<Vec<String>>();
    let keep_alive_s = rocket_cfg.keep_alive;
    let shutdown_mode = &rocket_cfg.shutdown;

    let routes = rocket_routes
        .map(|route| {
            let uri = route.uri.origin.to_string();
            let name = route
                .name
                .as_ref()
                .map(std::borrow::Cow::as_ref)
                .unwrap_or("[ERROR] Undefined");
            let method = route.method.as_str();
            format!("{method:<5} {uri:<20} {name}")
        })
        .collect::<Vec<String>>();

    let catchers = rocket_catchers
        .map(|catcher| {
            let base = catcher.base.to_string();
            let name = catcher
                .name
                .as_ref()
                .map(std::borrow::Cow::as_ref)
                .unwrap_or("[ERROR] Undefined");
            let code = catcher
                .code
                .map(|code| code.to_string())
                .unwrap_or("[ERROR] Undefined".to_string());

            format!("{code:<5} {base:<20} {name}")
        })
        .collect::<Vec<String>>();

    let display_vec = |data: Vec<String>| -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        out.push_str("[\n");
        out.push_str(&data.iter().fold(String::new(), |mut output, d| {
            writeln!(output, "    {d}").unwrap(); // Writing to a string cannot fail
            output
        }));
        out.push(']');
        out
    };

    info!("\nConfig:\nUsing profile: {profile}\nAddress: {address}:{port}\nWorkers: {workers}\nIndent: {indent}\nHeaders: {ip_headers}\nLimits: {formatted_limits}\nConnection lifetime: {keep_alive_s}s\nShutdown mode: {shutdown_mode}\nRoutes: {formatted_routes}\nCatchers: {formatted_catchers}",
        formatted_limits = display_vec(limits),
        formatted_routes = display_vec(routes),
        formatted_catchers = display_vec(catchers)
    );
}
