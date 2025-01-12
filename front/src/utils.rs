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

pub async fn copy_to_clipboard(text: &str) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsValue;

    async fn method_1(text: &str) -> Result<(), JsValue>{
        use wasm_bindgen::JsValue;
        use web_sys::window;
        let Some(window) = window() else{
            return Err(JsValue::from("Could not copy requested content to clipboard due to: Could not get a handle to the window"));
        };

        // let Some(clipboard) = window.navigator().clipboard() else{
        //     return Message::Error("Could not copy requested content to clipboard due to: Could not get a handle to the clipbard".to_string());
        // };

        let clipboard = window.navigator().clipboard();

        if let Err(e) = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(text)).await {
            return Err(JsValue::from(&format!("Could not copy requested content to clipboard due to: {e:?}")));
        }

        Ok(())
    }

    fn method_2(text: &str) -> Result<(), JsValue>{
        use wasm_bindgen::JsCast;
        use wasm_bindgen::JsValue;
        use web_sys::window;
        use gloo::console::log;

        let Some(window) = window()else{
            return Err(JsValue::from("Could not get a handle to the window"))
        };

        log!("Got window");
        let Some(document) = window.document() else{
            return Err(JsValue::from("Could not get a handle to the window's document"))
        };

        let html_document: web_sys::HtmlDocument = document.dyn_into()?;
        log!("Got document");

        let Some(body) = html_document.body() else{
            return Err(JsValue::from("Could not get a handle to the window's document's body"))
        };

        let text_area: web_sys::HtmlTextAreaElement =
            html_document.create_element("textarea")?.dyn_into()?;
        log!("Got text_area");

        text_area.set_text_content(Some(text));

        text_area.set_attribute("style", "position: fixed")?;
        log!("text_area has content and attributes");

        body.append_child(&text_area)?;
        log!("text_area has been planted");

        text_area.select();
        log!("text_area has been selected");


        if !html_document.exec_command("copy")? {
            gloo::console::log!("hehe no");
        }
        log!("copy executed");

        body.remove_child(&text_area)?;
        log!("text_area: cleared");
        Ok(())
    }

    // method_1(text).await
    method_2(text)

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
