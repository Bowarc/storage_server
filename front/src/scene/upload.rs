use std::collections::HashMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gloo::file::File as GlooFile;
use gloo::{console::log, file::callbacks::FileReader};
// use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
// use yew::html::TargetCast;
// use yew::{html, Callback, Component, Context, Html};
use std::sync::{Arc, Mutex};

static CURRENT_ID: Mutex<u32> = Mutex::new(0);

fn new_id() -> u32 {
    let mut guard = CURRENT_ID.lock().unwrap();
    *guard += 1;
    *guard - 1
}

#[derive(PartialEq)]
enum FileState {
    Loading,
    Local,
    Uploading,
    Uploaded(String),
}

struct FileDetails {
    id: u32,
    name: String,
    file_type: String,
    data64: Option<String>, // Encoded to base64
    state: FileState,
}

pub enum Msg {
    Loaded { id: u32, data: Vec<u8> },
    Load(Vec<GlooFile>),
    Upload,
    Uploaded { id: u32, upload_id: String },
    UploadError { id: u32, error: String },
}

#[derive(serde::Deserialize)]
struct UploadData {
    id: String,
    result: String,
}

pub struct Upload {
    readers: HashMap<u32, FileReader>,
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
            Msg::Loaded { id, data } => {
                self.readers.remove(&id);

                let mut file_entries = self
                    .files
                    .iter_mut()
                    .filter(|f| f.id == id)
                    .collect::<Vec<_>>();

                if file_entries.len() != 1 {
                    log!("TODO: manage error: multiple file with same id");
                }

                let file_entry = file_entries.get_mut(0).unwrap(); // This should never crash as we checked above
                file_entry.data64 = Some(STANDARD.encode(data));
                file_entry.state = FileState::Local;

                true
            }
            Msg::Load(files) => {
                for file in files.into_iter() {
                    let id = new_id();

                    self.files.push(FileDetails {
                        id,
                        data64: None,
                        name: file.name(),
                        file_type: file.raw_mime_type(),
                        state: FileState::Loading,
                    });

                    let link = ctx.link().clone();

                    let task = gloo::file::callbacks::read_as_bytes(&file, move |res| {
                        link.send_message(Msg::Loaded {
                            id,
                            data: res.expect("failed to read file"),
                        })
                    });

                    self.readers.insert(id, task);
                }
                true
            }
            Msg::Upload => {
                use gloo::utils::format::JsValueSerdeExt as _;
                for file in self.files.iter_mut() {
                    if file.state != FileState::Local {
                        log!(format!("File ({})'s state is not local", file.id));
                        continue;
                    }
                    let id = file.id;
                    let Some(data64) = file.data64.clone() else{
                        // File not yet loaded
                        continue;
                    };
                    let ext = file.extension().unwrap_or_default();
                    let name = file.name().unwrap_or_default();

                    file.state = FileState::Uploading;

                    ctx.link().send_future(async move{
                        use wasm_bindgen::JsCast as _;
                        let mut reqinit = {
                            let mut r = web_sys::RequestInit::new();
                            r.method("POST");
                            r.mode(web_sys::RequestMode::Cors);
                            r
                        };

                        reqinit.body(Some(&wasm_bindgen::JsValue::from_str(
                            &format!("{{\"metadata\": {{\"username\": \"TestUser\",\"file_name\": \"{name}\", \"file_ext\": \"{ext}\"}},\"file\": \"{data64}\"}}"),
                        )));

                        let request = match web_sys::Request::new_with_str_and_init("/upload", &reqinit){
                            Ok(request) => request,
                            Err(e) => return Msg::UploadError{id, error: e.as_string().unwrap_or(format!("Unable to retrieve the error: {e:?}"))},
                        };

                        request
                            .headers()
                            .set("Content-Type", "application/json")
                            .unwrap();

                        let window = gloo::utils::window();
                        let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
                        let resp: web_sys::Response = match resp_value.dyn_into() {
                            Ok(response) => response,
                            Err(e) => return Msg::UploadError{id, error: e.as_string().unwrap_or(format!("Unable to read the response due to: {e:?}"))},
                        };

                        let resp_json = match  resp.json()  {
                            Ok(resp_json) => resp_json,
                            Err(e) => return Msg::UploadError{id, error: e.as_string().unwrap_or(format!("Unable to parse the response due to: {e:?}"))},
                        };

                        let json_data = match  wasm_bindgen_futures::JsFuture::from(resp_json).await{
                            Ok(json_data) => json_data,
                            Err(e) => return Msg::UploadError{id, error: e.as_string().unwrap_or(format!("Unable to read response data due to: {e:?}"))},
                        };

                        log!(format!("Response: {json_data:?}"));

                        let data = json_data
                            .into_serde::<UploadData>()
                            .unwrap();

                        let upload_id = data.id.clone();

                        Msg::Uploaded{id, upload_id}
                    });
                }
                false
            }
            Msg::Uploaded { id, upload_id } => {
                log!(format!("Succesfully uploaded with id: {id}"));

                let mut file = self
                    .files
                    .iter_mut()
                    .filter(|f| f.id == id)
                    .collect::<Vec<_>>();
                if file.len() != 1 {
                    panic!("An error occured while trying to update sorted files with received id")
                }
                let f = file.get_mut(0).unwrap();

                f.state = FileState::Uploaded(upload_id);

                true
            }
            Msg::UploadError { id, error } => {
                log!(format!("File ({id}) failled to upload due to: {error}"));
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        use yew::TargetCast as _;

        yew::html! {<div class="upload_view">
            <p>{ "Drop your file(s) in the following box, and press upload" }</p>
            <label
                class = "upload_dragdrop_label"
                ondrop={ctx.link().callback(|event: yew::DragEvent| {
                    event.prevent_default();
                    let files = event.data_transfer().unwrap().files();
                    Self::load_files(files)
                })}
                ondragover={yew::Callback::from(|event: yew::DragEvent| {
                    event.prevent_default();
                })}
                ondragenter={yew::Callback::from(|event: yew::DragEvent| {
                    event.prevent_default();
                })}>
                <input
                    class = "upload_input"
                    type="file"
                    accept="image/*,video/*"
                    multiple={true}
                    onchange={ctx.link().callback(move |e: yew::Event| {
                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                        Self::load_files(input.files())
                    })}
                />
            </label>
            <p>{ "Drop your images here or click to select" }</p>
            <button class="upload_button" onclick={ctx.link().callback(|_| Msg::Upload)}>
                { "Upload !" }
            </button>
            <div>{
                // .rev() Does fix the video issue see #13
                for self.files.iter().map(Self::view_file)
            }</div>
        </div> }
    }
}

impl Upload {
    // fn view_file(file: &FileDetails) -> yew::Html {
    //     log!(format!(
    //         "Displaying file:\nType: {}\nName: {}\ndata64 size: {}",
    //         file.file_type,
    //         file.name,
    //         file.data64.len()
    //     ));

    //     yew::html! {<>
    //         <p class="preview-name">{ format!("{}", file.name) }</p>
    //         {
    //             match &file.state{
    //                 FileState::Local => yew::html!{ <p class="preview-state">{ "Not yet uploaded" }</p>},
    //                 FileState::Uploading => yew::html!{ <p class="preview-state">{ "Uploading . . ." }</p>},
    //                 FileState::Uploaded(id) => yew::html!{ <p class="preview-state">{ format!("Uploaded with id: {id}") }</p>},
    //             }
    //         }
    //         <div class="preview-media">
    //             if file.file_type.contains("image") {
    //                 <img style="width:30vw; height:auto" src={format!("data:{};base64,{}", file.file_type, file.data64)} />
    //             } else if file.file_type.contains("video") {
    //                 <video style="width:30vw; height:auto" controls={true}>
    //                     <source src={format!("data:{};base64,{}", file.file_type, file.data64)} type={file.file_type.clone()}/>
    //                 </video>
    //             }
    //         </div>
    //     </>}
    // }
    fn view_file(file: &FileDetails) -> yew::Html {
        log!(format!(
            "Displaying file:\nType: {}\nName: {}\ndata64 size: {:?}",
            file.file_type,
            file.name,
            file.data64.as_ref().and_then(|data| Some(data.len()))
        ));

        yew::html! {<div class="upload_file_preview">

            <p class="upload_file_preview_name">{ &file.name }</p>
            {
                match &file.state{
                    FileState::Loading => yew::html!{ <p class="preview-state">{ "Loading . . ." }</p>},
                    FileState::Local => yew::html!{ <p class="preview-state">{ "Not yet uploaded" }</p>},
                    FileState::Uploading => yew::html!{ <p class="preview-state">{ "Uploading . . ." }</p>},
                    FileState::Uploaded(id) => yew::html!{ <p class="preview-state">{ format!("Uploaded with id: {id}") }</p>},
                }
            }
        </div>}
    }

    fn load_files(files: Option<web_sys::FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(GlooFile::from);
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
