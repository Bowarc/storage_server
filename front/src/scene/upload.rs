use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use futures::SinkExt;
use gloo::{console::log, file::callbacks::FileReader};
use gloo::file::File;
use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use yew::html::TargetCast;
use yew::{html, Callback, Component, Context, Html};

struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
}

pub enum Msg {
    Loaded(String, String, Vec<u8>),
    Files(Vec<File>),
    Upload,
    Uploaded
}

pub struct Upload {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
}

impl yew::Component for Upload {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, file_type, data) => {
                self.files.push(FileDetails {
                    data,
                    file_type,
                    name: file_name.clone(),
                });
                self.readers.remove(&file_name);
                true
            }
            Msg::Files(files) => {
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
                for file in self.files.iter(){
                    let data = file.data.clone();
                    let ext = std::path::Path::new(&file.name)
                        .extension()
                        .and_then(std::ffi::OsStr::to_str).unwrap_or("").to_string();
                    ctx.link().send_future(async move{
                        use wasm_bindgen::JsCast as _;
                        let mut init = web_sys::RequestInit::new();
                        init.method("POST");
                        init.mode(web_sys::RequestMode::Cors);

                        let data = STANDARD.encode(data);

                        init.body(Some(&wasm_bindgen::JsValue::from_str(
                            &format!("{{\"metadata\": {{\"username\": \"TestUser\",\"file_ext\": \"{ext}\"}},\"file\": \"{data}\"}}"),
                        )));
                    
                        let request = web_sys::Request::new_with_str_and_init("/upload", &init).unwrap();
                    
                        request
                            .headers()
                            .set("Content-Type", "application/json")
                            .unwrap();
                    
                        let window = gloo::utils::window();
                        let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
                        let resp: web_sys::Response = resp_value.dyn_into().unwrap();
                    
                        let json_data = wasm_bindgen_futures::JsFuture::from(resp.json().unwrap()).await.unwrap();
                    
                        log!(json_data);
                    
                        Msg::Uploaded
                    });
                }
                false
            },
            Msg::Uploaded => {
                log!("Uploaded !");
                true
            }

        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <p>{ "Drop your file(s) in the following box, and press upload" }</p>
                <label id="file-upload">
                    <div
                        ondrop={ctx.link().callback(|event: DragEvent| {
                            event.prevent_default();
                            let files = event.data_transfer().unwrap().files();
                            Self::upload_files(files)
                        })}
                        ondragover={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                        ondragenter={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                    >
                        <i style="display: block;border: 2px solid red; width: 99.vw;height: 100px"></i>
                        <p>{"Drop your images here or click to select"}</p>
                        <button onclick={ctx.link().callback(|_| Msg::Upload)}>
                            { "Upload !" }
                        </button>

                    </div>
                </label>
                // <input
                //     id="file-upload"
                //     type="file"
                //     accept="image/*,video/*"
                //     multiple={true}
                //     onchange={ctx.link().callback(move |e: Event| {
                //         let input: HtmlInputElement = e.target_unchecked_into();
                //         Self::upload_files(input.files())
                //     })}
                // />
                <div>
                    { for self.files.iter().map(Self::view_file) }
                </div>
            </div>
        }
    }
}

impl Upload {
    fn view_file(file: &FileDetails) -> Html {
        html! {
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

    fn upload_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Files(result)
    }
}
