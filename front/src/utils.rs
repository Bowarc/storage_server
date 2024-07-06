#![allow(dead_code)]

use {gloo::console::log, js_sys::Date};

#[derive(Debug)]
pub enum FetchState<T> {
    NotFetching,
    Fetching,
    Success(T),
    Failed(wasm_bindgen::JsValue),
}

pub fn time_since(date: Date) -> String {
    let s =
        |n: i32, time: &str| -> String { format!("{n} {time}{}", if n > 1 { "s" } else { "" }) };

    let seconds = (Date::new_0().get_time() - date.get_time() / 1000.) as i32;

    let mut interval = seconds / 31536000;
    if interval > 1 {
        return s(interval, "year");
    }
    interval = seconds / 2592000;
    if interval > 1 {
        return s(interval, "month");
    }
    interval = seconds / 86400;
    if interval > 1 {
        return s(interval, "day");
    }
    interval = seconds / 3600;
    if interval > 1 {
        return s(interval, "hour");
    }
    interval = seconds / 60;
    if interval > 1 {
        return s(interval, "minute");
    }

    s(interval, "second")
}

pub fn add_script(path: &str, id: &str) {
    let Some(window) = web_sys::window() else {
        log!(format!(
            "Could not set script {id} due to: Could not get the window"
        ));
        return;
    };
    let Some(document) = window.document() else {
        log!(format!(
            "Could not set script {id} due to: Could not get the document"
        ));
        return;
    };

    let script = document.create_element("script").unwrap();
    script.set_attribute("src", path).unwrap();
    script.set_attribute("defer", "").unwrap();
    script.set_id(id);
    document.body().unwrap().append_child(&script).unwrap();
}

pub fn remove_script(id: &str) {
    let Some(window) = web_sys::window() else {
        log!(format!(
            "Could not remove script {id} due to: Could not get the window"
        ));
        return;
    };
    let Some(document) = window.document() else {
        log!(format!(
            "Could not remove script {id} due to: Could not get the document"
        ));
        return;
    };

    if let Some(script_element) = document.get_element_by_id(id) {
        let parent_node = script_element.parent_node().unwrap();
        parent_node.remove_child(&script_element).unwrap();
    }
}

// async fn fetch_dashboard(url: &'static str) -> Result<DashboardData, FetchError> {
//     let mut opts = RequestInit::new();
//     opts.method("GET");
//     opts.mode(RequestMode::Cors);

//     console::log!(opts.clone());

//     let request = Request::new_with_str_and_init(url, &opts)?;

//     console::log!(request.url());
//     let window = gloo::utils::window();
//     let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
//     let resp: Response = resp_value.dyn_into().unwrap();

//     let json_data = JsFuture::from(resp.json()?).await?;
//     console::log!(json_data.clone());

//     let x = json_data
//         .into_serde::<Vec<String>>()
//         .unwrap()
//         .iter()
//         .map(|s| serde_json::from_str(&s).unwrap())
//         .collect::<Vec<shared::data::CacheEntry>>();

//     Ok(DashboardData { cache_list: x })
// }
