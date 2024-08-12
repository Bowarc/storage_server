use gloo::console::log;

static CURRENT_LOCAL_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
const SIZE_LIMIT_BYTES: usize = 1024 /*kb*/ * 1024 /*mb*/ * 50;

fn new_local_id() -> u32 {
    use std::sync::atomic::Ordering;

    CURRENT_LOCAL_ID.fetch_add(1, Ordering::AcqRel)
}

#[derive(PartialEq)]
enum FileState {
    Loading,
    Local,
    Uploading,
    Uploaded(uuid::Uuid),
    UploadError(String),
}

struct UserFile {
    local_id: u32,
    name: String,
    file_type: String,
    data64: Option<Vec<u8>>, // Encoded to base64
    state: FileState,
}

pub enum Message {
    Void,
    Load(gloo::file::File),
    Loaded {
        local_id: u32,
        data: Vec<u8>,
    },
    Upload,
    Uploaded {
        local_id: u32,
        upload_uuid: uuid::Uuid,
    },
    UploadError {
        local_id: u32,
        error: String,
    },
    RemoveLocal {
        local_id: u32,
    },
    Error(String),
}

pub struct Upload {
    readers: std::collections::HashMap<u32, gloo::file::callbacks::FileReader>,
    files: Vec<UserFile>,
}

impl yew::Component for Upload {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            readers: std::collections::HashMap::default(),
            files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        use {crate::component, gloo::file::callbacks};

        match msg {
            Message::Void => false,
            Message::Load(file) => {
                let local_id = new_local_id();

                self.files.push(UserFile {
                    local_id,
                    data64: None,
                    name: file.name(),
                    file_type: file.raw_mime_type(),
                    state: FileState::Loading,
                });

                let link = ctx.link().clone();

                let task = callbacks::read_as_bytes(&file, move |res| {
                    let message = match res {
                        Ok(data) => Message::Loaded { local_id, data },
                        Err(e) => {
                            Message::Error(format!("Could not load given file due to: {e:?}"))
                        }
                    };

                    link.send_message(message)
                });

                self.readers.insert(local_id, task);
                true
            }
            Message::Loaded { local_id, data } => {
                self.readers.remove(&local_id);

                let Some(file) = self.files.iter_mut().find(|f| f.local_id == local_id) else {
                    log!(format!("[Error] Could not get file ({local_id}) from list"));
                    component::push_notification(crate::component::Notification::error(
                        "Internal error",
                        vec![&format!(
                            "Could not find file with id {local_id} in local file list."
                        )],
                        5.,
                    ));
                    return true;
                };

                let data_size = data.len();

                if data_size > SIZE_LIMIT_BYTES {
                    component::push_notification(component::Notification::error(
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
                    self.files.retain(|f| f.local_id != local_id);
                    return true;
                }

                file.data64 = Some(data);
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
                use std::str::FromStr as _;

                let mut count = 0;

                for file in self.files.iter_mut() {
                    let local_id = file.local_id;
                    if file.state != FileState::Local {
                        log!(format!("File ({local_id})'s state is not local"));
                        continue;
                    }
                    let Some(data) = file.data64.clone() else {
                        // File not yet loaded
                        continue;
                    };
                    let name = file.name().unwrap_or_default();
                    let ext = file.extension().unwrap_or_default();

                    file.state = FileState::Uploading;
                    count += 1;

                    ctx.link().send_future(async move {
                        use wasm_bindgen::JsCast as _;
                        let mut reqinit = {
                            let mut r = web_sys::RequestInit::new();
                            r.method("PUT");
                            r.mode(web_sys::RequestMode::Cors);
                            r
                        };

                        //
                        //  Safety:
                        //      The original data won't be accessed before that object is destroyed
                        //
                        let data_uint8_array = unsafe { js_sys::Uint8Array::view(&data) };

                        reqinit.body(Some(&data_uint8_array));

                        let request = match web_sys::Request::new_with_str_and_init(
                            &format!("/{name}.{ext}"),
                            &reqinit,
                        ) {
                            Ok(request) => request,
                            Err(e) => {
                                return Message::UploadError {
                                    local_id,
                                    error: e
                                        .as_string()
                                        .unwrap_or(format!("Unable to retrieve the error: {e:?}")),
                                }
                            }
                        };

                        let window = gloo::utils::window();
                        let resp_value = match wasm_bindgen_futures::JsFuture::from(
                            window.fetch_with_request(&request),
                        )
                        .await
                        {
                            Ok(resp_value) => resp_value,
                            Err(e) => {
                                return Message::UploadError {
                                    local_id,
                                    error: e.as_string().unwrap_or(format!(
                                        "Unable to receive the response due to: {e:?}"
                                    )),
                                }
                            }
                        };
                        let resp: web_sys::Response = match resp_value.dyn_into() {
                            Ok(response) => response,
                            Err(e) => {
                                return Message::UploadError {
                                    local_id,
                                    error: e.as_string().unwrap_or(format!(
                                        "Unable to read the response due to: {e:?}"
                                    )),
                                }
                            }
                        };

                        if !resp.ok() {
                            return Message::UploadError {
                                local_id,
                                error: format!(
                                    "Response error with status: {:?}",
                                    resp.status_text()
                                ),
                            };
                        }

                        let Ok(resp_text_promise) = resp.text() else {
                            return Message::UploadError {
                                local_id,
                                error: format!("Could not get text"),
                            };
                        };

                        let resp_text_value =
                            match wasm_bindgen_futures::JsFuture::from(resp_text_promise).await {
                                Ok(resp_text_value) => resp_text_value,
                                Err(e) => {
                                    return Message::UploadError {
                                        local_id,
                                        error: e.as_string().unwrap_or_else(|| {
                                            format!("Failed to read response body: {:?}", e)
                                        }),
                                    };
                                }
                            };

                        let Some(resp_text) = resp_text_value.as_string() else {
                            return Message::UploadError {
                                local_id,
                                error: format!("Could not parse received id"),
                            };
                        };

                        let Ok(uuid) = uuid::Uuid::from_str(&resp_text) else {
                            return Message::UploadError {
                                local_id,
                                error: format!("Could not parse received id into a uuid"),
                            };
                        };

                        log!(format!("Response: {uuid:?}"));

                        Message::Uploaded {
                            local_id,
                            upload_uuid: uuid,
                        }
                    });
                }

                crate::component::push_notification(crate::component::Notification::info(
                    "Info",
                    vec![&format!("Uploading {count} files")],
                    10.,
                ));

                true
            }
            Message::Uploaded {
                local_id,
                upload_uuid,
            } => {
                log!(format!("Succesfully uploaded with id: {local_id}"));

                let Some(file) = self.files.iter_mut().find(|f| f.local_id == local_id) else {
                    log!(format!("[Error] Could not get file ({local_id}) from list"));
                    crate::component::push_notification(crate::component::Notification::error(
                        "Internal error",
                        vec![&format!(
                            "Could not find file with id {local_id} in local file list."
                        )],
                        5.,
                    ));
                    return true;
                };

                component::push_notification(component::Notification::info(
                    "Uploaded file",
                    vec![
                        &format!("File name: {}", file.name),
                        &format!("Upload Uuid: {upload_uuid}"),
                    ],
                    5.,
                ));

                file.state = FileState::Uploaded(upload_uuid);

                true
            }
            Message::UploadError { local_id, error } => {
                log!(format!(
                    "File ({local_id}) failled to upload due to: {error}"
                ));

                let Some(file) = self.files.iter_mut().find(|f| f.local_id == local_id) else {
                    log!(format!("[Error] Could not get file ({local_id}) from list"));
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
            Message::RemoveLocal { local_id } => {
                let Some(file) = self.files.iter().find(|f| f.local_id == local_id) else {
                    crate::component::push_notification(crate::component::Notification::info(
                        "Error",
                        vec!["Could not find given file"],
                        5.,
                    ));
                    return true;
                };

                crate::component::push_notification(crate::component::Notification::info(
                    "File removed",
                    vec![&format!("File name: {}", file.name)],
                    5.,
                ));

                self.files.retain(|f| f.local_id != local_id);
                true
            }
            Message::Error(e) => {
                crate::component::push_notification(crate::component::Notification::info(
                    "An error occured",
                    vec![&format!("{e}")],
                    5.,
                ));
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
                ondrop={ctx.link().batch_callback(|event: yew::DragEvent| {
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
                    onchange={ctx.link().batch_callback(move |e: yew::Event| {
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
                    let local_id = file.local_id;

                    yew::html! {<div class="upload_file_preview">
                        <div class="upload_file_preview_img_bg"><img src="/resources/upload.png" /></div>
                        <div class="upload_file_preview_info">
                            <p class="upload_file_preview_name">{ &file.name }</p>
                            {
                                match &file.state{
                                    FileState::Loading => yew::html!{ <p class="preview-state">{ "Loading . . ." }</p>},
                                    FileState::Local => yew::html!{ <p class="preview-state">{ "Not yet uploaded" }</p>},
                                    FileState::Uploading => yew::html!{ <p class="preview-state">{ "Uploading . . ." }</p>},
                                    FileState::Uploaded(local_id) => yew::html!{ <p class="preview-state">{ format!("Uploaded with id: {local_id}") }</p>},
                                    FileState::UploadError(_error) => yew::html!{ <p class="preview-state">{ format!("Upload error") }</p>},
                                }
                            }
                        </div>

                        if matches!(file.state, FileState::Local) {
                            <div class="upload_file_preview_delete_button_wrapper" onclick={ctx.link().callback(move |_| Message::RemoveLocal { local_id })}>
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

    fn load_files(input_files: Option<web_sys::FileList>) -> Vec<Message> {
        let Some(files) = input_files else {
            return vec![Message::Void];
        };

        let Ok(Some(fileiter)) = js_sys::try_iter(&files) else {
            return vec![Message::Error(
                "Unable to create an iterator over given files".to_string(),
            )];
        };

        fileiter
            .map(|v| match v {
                Ok(f) => Message::Load(gloo::file::File::from(web_sys::File::from(f))),
                Err(e) => Message::Error(format!("{e:?}")),
            })
            .collect::<Vec<_>>()
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
