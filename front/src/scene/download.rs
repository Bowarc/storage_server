use std::io::Read;

use base64::Engine;
use gloo::console::log;
use web_sys::window;
use yew::{html, TargetCast as _};

pub struct Download {
    input_text: String,
}

pub enum Message {
    InputChanged(String),
    StartDownload,
    DownloadFinished(DownloadData),
    DownloadFailled(String),
}

#[derive(serde::Deserialize)]
struct DownloadData {
    file: String,
    metadata: DownloadMetaData,
}

#[derive(serde::Deserialize)]
struct DownloadMetaData {
    file_ext: String,
    file_name: String,
    username: String,
}

impl yew::Component for Download {
    type Message = Message;

    type Properties = ();

    fn create(_ctx: &yew::prelude::Context<Self>) -> Self {
        Self {
            input_text: String::default(),
        }
    }

    fn update(&mut self, ctx: &yew::prelude::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::InputChanged(new_text) => {
                self.input_text = new_text;
                true
            }
            Message::StartDownload => {
                use gloo::utils::format::JsValueSerdeExt as _;

                let id = self.input_text.clone();

                ctx.link().send_future(async move {
                    use wasm_bindgen::JsCast as _;

                    let mut reqinit = {
                        let mut r = web_sys::RequestInit::new();
                        r.method("GET");
                        r.mode(web_sys::RequestMode::Cors);
                        r
                    };

                    let request = match web_sys::Request::new_with_str_and_init(
                        &format!("/download/{id}"),
                        &reqinit,
                    ) {
                        Ok(request) => request,
                        Err(e) => {
                            return Message::DownloadFailled(
                                e.as_string()
                                    .unwrap_or(format!("Unable to retrieve the error: {e:?}")),
                            )
                        }
                    };

                    request
                        .headers()
                        .set("Content-Type", "application/json")
                        .unwrap();

                    let window = gloo::utils::window();
                    let resp_value =
                        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
                            .await
                            .unwrap();
                    let resp: web_sys::Response =
                        match resp_value.dyn_into() {
                            Ok(response) => response,
                            Err(e) => {
                                return Message::DownloadFailled(e.as_string().unwrap_or(format!(
                                    "Unable to read the response due to: {e:?}"
                                )))
                            }
                        };

                    let status = resp.status();

                    let resp_json =
                        match resp.json() {
                            Ok(resp_json) => resp_json,
                            Err(e) => {
                                return Message::DownloadFailled(e.as_string().unwrap_or(format!(
                                    "Unable to parse the response due to: {e:?}"
                                )))
                            }
                        };

                    let json_data =
                        match wasm_bindgen_futures::JsFuture::from(resp_json).await {
                            Ok(json_data) => json_data,
                            Err(e) => {
                                return Message::DownloadFailled(e.as_string().unwrap_or(format!(
                                    "Unable to read response data due to: {e:?}"
                                )))
                            }
                        };

                    match status {
                        200 => {
                            log!("200");
                            let data = json_data.into_serde::<DownloadData>().unwrap();

                            log!(format!(
                                "{}.{} from {}",
                                data.metadata.file_name,
                                data.metadata.file_ext,
                                data.metadata.username
                            ));
                            return Message::DownloadFinished(data);
                        }
                        400 => {
                            log!("400");
                            let data = json_data
                                .into_serde::<std::collections::HashMap<String, String>>()
                                .unwrap();

                            let result = data.get("result").unwrap();
                            let message = data.get("message").unwrap();

                            log!(format!("Failled to download, result: {result}"));
                            return Message::DownloadFailled(message.clone());
                        }
                        _ => {
                            log!("Unrecognized status code")
                        }
                    }

                    log!(format!("{json_data:?}"));

                    Message::DownloadFailled("TODO".to_string())
                });
                true
            }
            Message::DownloadFinished(data) => {
                use base64::engine::general_purpose::STANDARD;
                use wasm_bindgen::JsCast as _;
                log!(format!("Download finished, data size: {}", data.file.len()));
                let mut buffer = Vec::new();

                STANDARD.decode_vec(data.file, &mut buffer).unwrap();

                let array = js_sys::Uint8Array::new_with_length(buffer.len() as u32);
                array.copy_from(&buffer);

                // Create a Blob from the Uint8Array
                let blob_parts = js_sys::Array::new();
                blob_parts.push(&array.buffer());
                let mut options = web_sys::BlobPropertyBag::new();
                options.type_("data:text/plain;charset=utf-8");
                let blob =
                    web_sys::Blob::new_with_u8_array_sequence_and_options(&blob_parts, &options)
                        .unwrap();

                // Create a URL for the Blob
                let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

                // Create an anchor element
                let document = web_sys::window().unwrap().document().unwrap();
                let a = document.create_element("a").unwrap();
                let a: web_sys::HtmlElement = a.dyn_into::<web_sys::HtmlElement>().unwrap();

                // Set the href and download attributes
                a.set_attribute("href", &url).unwrap();
                a.set_attribute(
                    "download",
                    &format!("{}.{}", data.metadata.file_name, data.metadata.file_ext),
                ).unwrap();

                // Append the anchor to the body, click it, and remove it
                document.body().unwrap().append_child(&a).unwrap();
                a.click();
                document.body().unwrap().remove_child(&a).unwrap();

                // Revoke the object URL to free up resources
                web_sys::Url::revoke_object_url(&url).unwrap();

                // let document = window().document().unwrap();

                // let link: web_sys::HtmlElement =
                //     document.create_element("a").unwrap().dyn_into().unwrap();

                // // let file_data = std::str::from_utf8(&buffer).unwrap();

                // link.set_attribute("style", "display: none").unwrap();
                // link.set_attribute(
                //     "href",
                //     &format!(
                //         "data:text/plain;charset=utf-8, {:?}",
                //         buffer.bytes()
                //     ),
                // )
                // .unwrap();
                // link.set_attribute(
                //     "download",
                //     &format!("{}.{}", data.metadata.file_name, data.metadata.file_ext),
                // )
                // .unwrap();

                // document.body().unwrap().append_child(&link).unwrap();

                // link.click();

                false
            }
            Message::DownloadFailled(e) => {
                log!(format!("Download failled with: {e}"));
                false
            }
        }
    }

    fn view(&self, ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        html! {<>
            { "Download page" }
            <br/>
            <input
                // type="text"
                oninput={ctx.link().callback(move |e: yew::InputEvent| {
                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                    Message::InputChanged(input.value())
                })}
            />
            <br/>
            {&self.input_text}
            <br/>
            <button onclick={ctx.link().callback(|_| Message::StartDownload)}>
                { "Download !" }
            </button>
            // <a href="https://www.flaticon.com/free-icons/storage" title="storage icons">{"Storage icons created by Freepik - Flaticon"}</a>

        </>}
    }
}
