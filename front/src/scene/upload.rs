use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
// use futures::SinkExt;
use gloo::file::File;
use gloo::{console::log, file::callbacks::FileReader};
// use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
// use yew::html::TargetCast;
// use yew::{html, Callback, Component, Context, Html};

struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
}

pub enum Msg {
    Loaded(String, String, Vec<u8>),
    Load(Vec<File>),
    Upload,
    Uploaded{id: String},
    UploadError(String),
}

#[derive(serde::Deserialize)]
struct UploadData{
    id: String,
    result: String,
}

pub struct Upload {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
}

impl yew::Component for Upload {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, file_type, data) => {
                self.readers.remove(&file_name);
                self.files.push(FileDetails {
                    data,
                    file_type,
                    name: file_name,
                });
                true
            }
            Msg::Load(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();

                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(
                                file_name,
                                file_type,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
            Msg::Upload => {
                use gloo::utils::format::JsValueSerdeExt as _;
                for file in self.files.iter() {
                    let data = file.data.clone();
                    let ext = file.extension().unwrap_or_default();
                    let name = file.name().unwrap_or_default();

                    ctx.link().send_future(async move{
                        use wasm_bindgen::JsCast as _;
                        let mut reqinit = {
                            let mut r = web_sys::RequestInit::new();
                            r.method("POST");
                            r.mode(web_sys::RequestMode::Cors);
                            r
                        };

                        let data = STANDARD.encode(data);

                        reqinit.body(Some(&wasm_bindgen::JsValue::from_str(
                            &format!("{{\"metadata\": {{\"username\": \"TestUser\",\"file_name\": \"{name}\", \"file_ext\": \"{ext}\"}},\"file\": \"{data}\"}}"),
                        )));

                        let request = match web_sys::Request::new_with_str_and_init("/upload", &reqinit){
                            Ok(request) => request,
                            Err(e) => return Msg::UploadError(e.as_string().unwrap_or(format!("Unable to retrieve the error: {e:?}"))),
                        };

                        request
                            .headers()
                            .set("Content-Type", "application/json")
                            .unwrap();

                        let window = gloo::utils::window();
                        let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
                        let resp: web_sys::Response = match resp_value.dyn_into() {
                            Ok(response) => response,
                            Err(e) => return Msg::UploadError(e.as_string().unwrap_or(format!("Unable to read the response due to: {e:?}"))),
                        };

                        let resp_json = match  resp.json()  {
                            Ok(resp_json) => resp_json,
                            Err(e) => return Msg::UploadError(e.as_string().unwrap_or(format!("Unable to parse the response due to: {e:?}"))),
                        };

                        let json_data = match  wasm_bindgen_futures::JsFuture::from(resp_json).await{
                            Ok(json_data) => json_data,
                            Err(e) => return Msg::UploadError(e.as_string().unwrap_or(format!("Unable to read response data due to: {e:?}"))),
                        };

                        log!(format!("Response: {json_data:?}"));

                        let data = json_data
                            .into_serde::<UploadData>()
                            .unwrap();

                        let id = data.id.clone();

                        Msg::Uploaded{id}
                    });
                }
                false
            }
            Msg::Uploaded{id} => {
                log!(format!("Succesfully uploaded with id: {id}"));
                true
            }
            Msg::UploadError(e) => {
                log!(format!("Received UploadError message with content: {e}"));
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        use yew::TargetCast as _;
        yew::html! {<div class="upload_view">
            <p>{ "Drop your file(s) in the following box, and press upload" }</p>
            <label
                    style="display: block;border: 2px solid red; width: 99.vw;height: 100px"
                    ondrop={ctx.link().callback(|event: yew::DragEvent| {
                        event.prevent_default();
                        let files = event.data_transfer().unwrap().files();
                        Self::upload_files(files)
                    })}
                    ondragover={yew::Callback::from(|event: yew::DragEvent| {
                        event.prevent_default();
                    })}
                    ondragenter={yew::Callback::from(|event: yew::DragEvent| {
                        event.prevent_default();
                    })}>
                    <input
                        id="file-upload"
                        type="file"
                        accept="image/*,video/*"
                        style="display:none"
                        multiple={true}
                        onchange={ctx.link().callback(move |e: yew::Event| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            Self::upload_files(input.files())
                        })}
                    />
            </label>
            <p>{"Drop your images here or click to select"}</p>
            <button onclick={ctx.link().callback(|_| Msg::Upload)}>
                { "Upload !" }
            </button>
            <div>
                { for self.files.iter().map(Self::view_file) }
            </div>
        </div> }
    }
}

impl Upload {
    fn view_file(file: &FileDetails) -> yew::Html {
        yew::html! {
            <div class="preview-tile">
                <p class="preview-name">{ format!("{}", file.name) }</p>
                <div class="preview-media">
                    if file.file_type.contains("image") {
                        <img src={format!("data:{};base64,{}", file.file_type, STANDARD.encode(&file.data))} />
                    } else if file.file_type.contains("video") {
                        <video controls={true}>
                            <source src={format!("data:{};base64,{}", file.file_type, STANDARD.encode(&file.data))} type={file.file_type.clone()}/>
                        </video>
                    }
                </div>
            </div>
        }
    }

    fn upload_files(files: Option<web_sys::FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Load(result)
    }
}

impl FileDetails {
    fn extension(&self) -> Option<String> {
        let name = &self.name;

        if !name.contains(".") {
            return None;
        }

        let dot_index = name.rfind(".").unwrap();

        Some(String::from(&name[(dot_index + 1)..name.len()]))
    }

    fn name(&self) -> Option<String> {
        let name = &self.name;

        if !name.contains(".") {
            return None;
        }

        let dot_index = name.rfind(".").unwrap();

        Some(String::from(&name[0..dot_index]))
    }
}
