use {
    crate::component,
    base64::{engine::general_purpose::STANDARD, Engine},
    gloo::{
        console::log,
        file::{callbacks::FileReader, File as GlooFile},
    },
    std::{collections::HashMap, sync::Mutex},
};

static CURRENT_ID: Mutex<u32> = Mutex::new(0);
const SIZE_LIMIT_BYTES: usize = 1024 /*kb*/ * 1024 /*mb*/ * 50;

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
    UploadError(String),
}

struct UserFile {
    id: u32,
    name: String,
    file_type: String,
    data64: Option<String>, // Encoded to base64
    state: FileState,
}

pub enum Message {
    Load(Vec<GlooFile>),
    Loaded { id: u32, data: Vec<u8> },
    Upload,
    Uploaded { id: u32, upload_id: String },
    UploadError { id: u32, error: String },

    RemoveLocal { id: u32 },
}

#[derive(serde::Deserialize)]
struct UploadData {
    id: String,
    result: String,
}

pub struct Upload {
    readers: HashMap<u32, FileReader>,
    files: Vec<UserFile>,
}

impl yew::Component for Upload {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Load(files) => {
                for file in files.into_iter() {
                    let id = new_id();

                    self.files.push(UserFile {
                        id,
                        data64: None,
                        name: file.name(),
                        file_type: file.raw_mime_type(),
                        state: FileState::Loading,
                    });

                    let link = ctx.link().clone();

                    let task = gloo::file::callbacks::read_as_bytes(&file, move |res| {
                        link.send_message(Message::Loaded {
                            id,
                            data: res.expect("failed to read file"),
                        })
                    });

                    self.readers.insert(id, task);
                }
                true
            }
            Message::Loaded { id, data } => {
                self.readers.remove(&id);

                let Some(file) = self.files.iter_mut().find(|f| f.id == id) else {
                    log!(format!("[Error] Could not get file ({id}) from list"));
                    crate::component::push_notification(crate::component::Notification::error(
                        "Internal error",
                        vec![&format!(
                            "Could not find file with id {id} in local file list."
                        )],
                        5.,
                    ));
                    return true;
                };

                let data_size = data.len();

                if data_size > SIZE_LIMIT_BYTES {
                    crate::component::push_notification(crate::component::Notification::error(
                        "File too large",
                        vec![
                            &format!("File: {}", file.name),
                            &format!("File size: {:.0} MB", data_size as f64 / (1024. * 1024.)),
                            &format!(
                                "Max size: {:.0} MB",
                                SIZE_LIMIT_BYTES as f64 / (1024. * 1024.)
                            ),
                        ],
                        5.,
                    ));
                    self.files.retain(|f| f.id != id);
                    return true;
                }

                file.data64 = Some(STANDARD.encode(data));
                file.state = FileState::Local;

                {
                    component::push_notification(component::Notification::info(
                        "Loaded file",
                        vec![
                            &format!("File name: {}", file.name),
                            &format!("File size: {:.1}mb", data_size as f64 / (1024. * 1024.)),
                        ],
                        5.,
                    ));
                }

                true
            }
            Message::Upload => {
                use gloo::utils::format::JsValueSerdeExt as _;

                let mut count = 0;

                for file in self.files.iter_mut() {
                    let id = file.id;
                    if file.state != FileState::Local {
                        log!(format!("File ({id})'s state is not local"));
                        continue;
                    }
                    let Some(data64) = file.data64.clone() else {
                        // File not yet loaded
                        continue;
                    };
                    let ext = file.extension().unwrap_or_default();
                    let name = file.name().unwrap_or_default();

                    file.state = FileState::Uploading;
                    count += 1;

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
                            Err(e) => return Message::UploadError{id, error: e.as_string().unwrap_or(format!("Unable to retrieve the error: {e:?}"))},
                        };

                        request
                            .headers()
                            .set("Content-Type", "application/json")
                            .unwrap();

                        let window = gloo::utils::window();
                        let resp_value = match wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await{
                            Ok(resp_value) => resp_value,
                            Err(e) =>  return Message::UploadError{id, error: e.as_string().unwrap_or(format!("Unable to receive the response due to: {e:?}"))},
                        };
                        let resp: web_sys::Response = match resp_value.dyn_into() {
                            Ok(response) => response,
                            Err(e) => return Message::UploadError{id, error: e.as_string().unwrap_or(format!("Unable to read the response due to: {e:?}"))},
                        };

                        let resp_json = match  resp.json()  {
                            Ok(resp_json) => resp_json,
                            Err(e) => return Message::UploadError{id, error: e.as_string().unwrap_or(format!("Unable to parse the response due to: {e:?}"))},
                        };

                        let json_data = match  wasm_bindgen_futures::JsFuture::from(resp_json).await{
                            Ok(json_data) => json_data,
                            Err(e) => return Message::UploadError{id, error: e.as_string().unwrap_or(format!("Unable to read response data due to: {e:?}"))},
                        };

                        log!(format!("Response: {json_data:?}"));

                        let data = json_data
                            .into_serde::<UploadData>()
                            .unwrap();

                        let upload_id = data.id.clone();

                        Message::Uploaded{id, upload_id}
                    });
                }

                crate::component::push_notification(crate::component::Notification::info(
                    "Info",
                    vec![&format!("Uploading {count} files")],
                    10.,
                ));

                true
            }
            Message::Uploaded { id, upload_id } => {
                log!(format!("Succesfully uploaded with id: {id}"));

                let Some(file) = self.files.iter_mut().find(|f| f.id == id) else {
                    log!(format!("[Error] Could not get file ({id}) from list"));
                    crate::component::push_notification(crate::component::Notification::error(
                        "Internal error",
                        vec![&format!(
                            "Could not find file with id {id} in local file list."
                        )],
                        5.,
                    ));
                    return true;
                };

                component::push_notification(component::Notification::info(
                    "Uploaded file",
                    vec![
                        &format!("File name: {}", file.name),
                        &format!("Upload Id: {upload_id}"),
                    ],
                    5.,
                ));

                file.state = FileState::Uploaded(upload_id);

                true
            }
            Message::UploadError { id, error } => {
                log!(format!("File ({id}) failled to upload due to: {error}"));

                let Some(file) = self.files.iter_mut().find(|f| f.id == id) else {
                    log!(format!("[Error] Could not get file ({id}) from list"));
                    component::push_notification(component::Notification::error(
                        "Upload error",
                        vec![
                            &format!("File name: [ERROR] Unknown file"),
                            &format!("{error}"),
                        ],
                        5.,
                    ));
                    return true;
                };

                component::push_notification(component::Notification::error(
                    "Upload error",
                    vec![&format!("File name: {}", file.name), &format!("{error}")],
                    10.,
                ));

                file.state = FileState::UploadError(error);

                true
            }
            Message::RemoveLocal { id } => {
                let Some(file) = self.files.iter().find(|f| f.id == id) else{
                    crate::component::push_notification(
                        crate::component::Notification::info(
                            "Error",
                            vec![
                                "Could not find given file",
                            ],
                            5.
                            )
                    );
                    return true;
                };

                crate::component::push_notification(
                    crate::component::Notification::info(
                        "File removed",
                        vec![
                            &format!("File name: {}", file.name),
                        ],
                        5.
                        )
                );

                self.files.retain(|f| f.id != id);
                true
            }
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        use yew::TargetCast as _;

        yew::html! {<div class="upload_view">
            <button class="upload_button" onclick={ctx.link().callback(|_| Message::Upload)}>
                { "Upload !" }
            </button>
            <label
                class = "upload_dragdrop"
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
                <img  src="/resources/upload.png" />
                <p>{ "Drop your file(s) here or click to select" }</p>
                <p class="upload_dragdrop_info">{ format!("{:.0}mb maximum", SIZE_LIMIT_BYTES as f64 / (1024. * 1024.)) }</p>
            </label>
            <div>{
                // .rev() Does fix the video issue see #13
                for self.files.iter().map(|file: &UserFile|{
                    let id = file.id;

                    yew::html! {<div class="upload_file_preview">
                        <div class="upload_file_preview_img_bg"><img src="/resources/upload.png" /></div>
                        <div class="upload_file_preview_info">
                            <p class="upload_file_preview_name">{ &file.name }</p>
                            {
                                match &file.state{
                                    FileState::Loading => yew::html!{ <p class="preview-state">{ "Loading . . ." }</p>},
                                    FileState::Local => yew::html!{ <p class="preview-state">{ "Not yet uploaded" }</p>},
                                    FileState::Uploading => yew::html!{ <p class="preview-state">{ "Uploading . . ." }</p>},
                                    FileState::Uploaded(id) => yew::html!{ <p class="preview-state">{ format!("Uploaded with id: {id}") }</p>},
                                    FileState::UploadError(_error) => yew::html!{ <p class="preview-state">{ format!("Upload error") }</p>},
                                }
                            }
                        </div>

                        if matches!(file.state, FileState::Local) {
                            <div class="upload_file_preview_delete_button_wrapper" onclick={ctx.link().callback(move |_| Message::RemoveLocal { id })}>
                                <button class="upload_file_preview_delete_button">
                                    <img src="/resources/delete.png" />
                                </button>
                            </div>
                        }

                    </div>}
                })
            }</div>
        </div>}
    }
}

impl Upload {
    // fn view_file(file: &UserFile) -> yew::Html {
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
    // fn view_file(file: &UserFile) -> yew::Html {
    //     yew::html! {<div class="upload_file_preview">
    //         <div class="upload_file_preview_img_bg"><img  src="/resources/upload.png" /></div>
    //         <div class="upload_file_preview_info">
    //             <p class="upload_file_preview_name">{ &file.name }</p>
    //             {
    //                 match &file.state{
    //                     FileState::Loading => yew::html!{ <p class="preview-state">{ "Loading . . ." }</p>},
    //                     FileState::Local => yew::html!{ <p class="preview-state">{ "Not yet uploaded" }</p>},
    //                     FileState::Uploading => yew::html!{ <p class="preview-state">{ "Uploading . . ." }</p>},
    //                     FileState::Uploaded(id) => yew::html!{ <p class="preview-state">{ format!("Uploaded with id: {id}") }</p>},
    //                     FileState::UploadError(_error) => yew::html!{ <p class="preview-state">{ format!("Upload error") }</p>},
    //                 }
    //             }
    //         </div>

    //             if matches!(file.state, FileState::Local) {
    //                 <button class="upload_file_preview_delete_button">
    //                     <img  src="/resources/delete.png" />
    //                 </button>
    //             }

    //     </div>}
    // }

    fn load_files(files: Option<web_sys::FileList>) -> Message {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(GlooFile::from);
            result.extend(files);
        }
        Message::Load(result)
    }
}

impl UserFile {
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
