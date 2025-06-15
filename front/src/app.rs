use crate::component::LightSwitch;
use yew::{function_component, use_state, Callback, Html};

#[derive(Debug, PartialEq, yew::Properties)]
pub struct Props {
    pub scenes: Vec<crate::scene::Scene>,
    pub default_scene_index: usize,
}

#[function_component]
pub fn App(props: &Props) -> Html {
    use {
        crate::component::NotificationManager,
        js_sys::Date,
        yew::{html, virtual_dom::VNode},
    };

    let scenes = props.scenes.clone();

    let current_scene_default = {
        scenes
            .get(props.default_scene_index)
            .or_else(|| scenes.first())
            .cloned()
            .unwrap()
    };

    let current_scene = use_state(|| current_scene_default);


    html! {
        <div id="global">
        <div id="header">
            <a class="header-item" href="http://github.com/Bowarc/wasm_portfolio">
                <img src="/resources/github.webp" alt="Github icon" class="icon"/>
            </a>
            <LightSwitch />
            <div class="header-item" id="scene_list">{
                scenes.into_iter().map(|scene|{
                    html!{
                        <button class={format!("scene_button{}", if *current_scene == scene {" current"} else {""})} onclick={
                            let current_scene_clone = current_scene.clone();
                            Callback::from(move |_| current_scene_clone.set(scene))
                        }>
                            { format!("{scene}") }
                        </button>
                    }
                }).collect::<Vec<VNode>>()
            }</div>
        </div>
        <div id="content">
            {
                current_scene.html(current_scene.clone())
            }
            <NotificationManager />
        </div>
        <footer>
            { format!("Rendered: {}", String::from(Date::new_0().to_string())) }
        </footer>
        </div>
    }
}

// pub enum Message {
//     InitWorms,
//     SwitchScene(crate::scene::Scene), // sao <3
//     ChangeLocale(bi18n::Locale),      // Request the change of the current locale to the given one
//     LocaleChanged(bi18n::Locale),     // A locale has changed
//     LoadLocale(bi18n::Locale),        // Request to fetch the given locale to server
//     LocaleLoadError(DataFetchError),
// }

// #[derive(Debug)]
// pub enum DataFetchError {
//     RequestCreation,
//     Fetch,
//     Parsing,
//     Status(u16),
// }

// pub struct App {
//     current_scene: crate::scene::Scene,
//     canvas_node_ref: yew::NodeRef,
// }

// impl yew::Component for App {
//     type Message = Message;
//     type Properties = Props;

//     fn create(ctx: &yew::Context<Self>) -> Self {
//         let scenes = &ctx.props().scenes;
//         let current_scene = *scenes
//             .get(ctx.props().default_scene_index)
//             .or_else(|| scenes.first())
//             .unwrap();

//         // Setup default i18n
//         // bi18n::set(bi18n::I18n::new(
//         //     bi18n::Locale::from("en"),
//         //     serde_json::from_str(include_str!("../resources/i18n/en.json")).unwrap(),
//         // ));

//         Self {
//             current_scene,
//             canvas_node_ref: yew::NodeRef::default(),
//         }
//     }

//     fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
//         use {
//             crate::component::start_wormgrid,
//             wasm_bindgen::JsCast as _,
//             web_sys::{window, HtmlCanvasElement, WebGlRenderingContext},
//         };

//         match msg {
//             Message::InitWorms => {
//                 let canvas = self.canvas_node_ref.cast::<HtmlCanvasElement>().unwrap();
//                 let w = window().unwrap();
//                 canvas.set_width(w.inner_width().unwrap().as_f64().unwrap() as u32);
//                 canvas.set_height(w.inner_height().unwrap().as_f64().unwrap() as u32);

//                 let glctx = canvas
//                     .get_context("webgl")
//                     .unwrap()
//                     .unwrap()
//                     .dyn_into::<WebGlRenderingContext>()
//                     .unwrap();

//                 start_wormgrid(glctx);
//                 true
//             }
//             Message::SwitchScene(scene) => {
//                 self.current_scene = scene;
//                 true
//             }
//             Message::ChangeLocale(locale) => {
//                 if bi18n::set_language(locale.clone()).is_ok() {
//                     ctx.link().send_message(Message::LocaleChanged(locale));
//                     return true;
//                 }

//                 ctx.link().send_message(Message::LoadLocale(locale));
//                 false
//             }
//             Message::LocaleChanged(locale) => {
//                 debug!(format!("Locale changed to: {locale}"));
//                 true
//             }
//             Message::LoadLocale(locale) => {
//                 use {
//                     gloo::console::log,
//                     wasm_bindgen::JsCast as _,
//                     wasm_bindgen_futures::JsFuture,
//                     web_sys::{window, Request, Response},
//                 };

//                 ctx.link().send_future(async move {
//                     let request =
//                         match Request::new_with_str(&format!("/resources/i18n/{locale}.json")) {
//                             Ok(request) => request,
//                             Err(e) => {
//                                 error!(format!(
//                                     "[ERROR] Failed to create locale data request due to: {e:?}"
//                                 ));
//                                 return Message::LocaleLoadError(DataFetchError::RequestCreation);
//                             }
//                         };

//                     let Some(window) = window() else {
//                         panic!("Failed to get the window");
//                     };

//                     let res = match JsFuture::from(window.fetch_with_request(&request)).await {
//                         Ok(res) => res,
//                         Err(e) => {
//                             error!(format!("[ERROR] Fetch (locale) failed due to: {e:?}"));
//                             return Message::LocaleLoadError(DataFetchError::Fetch);
//                         }
//                     };

//                     let Ok(resp) = res.dyn_into::<Response>() else {
//                         return Message::LocaleLoadError(DataFetchError::Parsing);
//                     };

//                     if resp.status() != 200 {
//                         return Message::LocaleLoadError(DataFetchError::Status(resp.status()));
//                     }

//                     let resp_text_promise = match resp.text() {
//                         Ok(json) => json,
//                         Err(e) => {
//                             error!(format!("[ERROR] failed to get response text due to: {e:?}"));
//                             return Message::LocaleLoadError(DataFetchError::Parsing);
//                         }
//                     };

//                     let resp_text_value = match JsFuture::from(resp_text_promise).await {
//                         Ok(json) => json,
//                         Err(e) => {
//                             error!(format!(
//                                 "[ERROR] Failed to convert response text to JsValue {e:?}"
//                             ));
//                             return Message::LocaleLoadError(DataFetchError::Parsing);
//                         }
//                     };

//                     let Some(resp_text) = resp_text_value.as_string() else {
//                         return Message::LocaleLoadError(DataFetchError::Parsing);
//                     };

//                     let map = match serde_json::from_str(&resp_text) {
//                         Ok(v) => v,
//                         Err(e) => {
//                             error!(format!(
//                                 "[ERROR] Failed to parse received group data due to: {e:?}"
//                             ));
//                             return Message::LocaleLoadError(DataFetchError::Parsing);
//                         }
//                     };

//                     log!(format!("Data received: {map:?}"));

//                     bi18n::set(bi18n::I18n::new(locale.clone(), map));

//                     Message::LocaleChanged(locale)
//                 });
//                 false
//             }
//             Message::LocaleLoadError(locales_load_error) => {
//                 error!(format!("{locales_load_error:?}"));
//                 false
//             }
//         }
//     }

//     // fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
//     //     use {crate::scene::Scene, gloo::console::log, js_sys::Date, yew::html};

//     //     log!("Draw");

//     //     let scenes = ctx.props().scenes.clone();

//     //     html! {
//     //         <div id="global">
//     //         <div id="header">
//     //             <a class="header-item" href="http://github.com/Bowarc/wasm_portfolio">
//     //                 <img src="resources/github.webp" alt="Github icon" class="icon"/>
//     //             </a>
//     //             <div id="scene_list" class="header-item">{
//     //                 scenes.into_iter().map(|scene|{
//     //                     let current = if self.current_scene == scene{
//     //                         "current"
//     //                     }else{
//     //                         ""
//     //                     };
//     //                     html!{
//     //                         <button class={format!("scene_button {current}")} onclick={ctx.link().callback(|_| Message::SwitchScene(scene))}>
//     //                             { format!("{scene}") }
//     //                         </button>
//     //                     }
//     //                 }).collect::<Vec<yew::virtual_dom::VNode>>()
//     //             }</div>
//     //         </div>
//     //         <div id="content">
//     //             <canvas id="gridworm-canvas" ref={self.canvas_node_ref.clone()} />
//     //             {
//     //                 self.current_scene.html()
//     //             }
//     //         </div>
//     //         <footer>
//     //             { format!("Rendered: {}", String::from(Date::new_0().to_string()))}
//     //         </footer>
//     //         </div>
//     //     }
//     // }

//     fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
//         use {
//             crate::component::NotificationManager,
//             js_sys::Date,
//             yew::{html, virtual_dom::VNode},
//             std::collections::HashMap,
//             i18nrs::{StorageType, yew::I18nProvider}
//         };

//         let scenes = ctx.props().scenes.clone();
//         html! {
//             <div id="global">
//             <div id="header">
//                 <div id="scene_list" class="header-item">{
//                     scenes.into_iter().map(|scene|{
//                         html!{
//                             <button class={format!("scene_button{}", if self.current_scene == scene {" current"} else {""})} onclick={ctx.link().callback(move |_| Message::SwitchScene(scene))}>
//                                 { format!("{scene}") }
//                             </button>
//                         }
//                     }).collect::<Vec<VNode>>()
//                 }</div>
//                 <div>{
//                         ["en", "fr"].into_iter().map(|locale|{
//                             html!{
//                                 <button onclick={ctx.link().callback(move |_| Message::ChangeLocale(locale.into()))} class="header-item" href="http://github.com/Bowarc/leaguecord">
//                                     <img src={format!("/resources/flag_{locale}.webp")} alt={format!("{locale} flag")} class="icon"/>
//                                 </button>
//                             }
//                         }).collect::<Vec<VNode>>()
//                 }</div>
//                 <a class="header-item" href="http://github.com/Bowarc/leaguecord">
//                     <img src="/resources/github.webp" alt="Github icon" class="icon"/>
//                 </a>
//             </div>
//             <div id="content">
//                 <I18nProvider
//                     translations={HashMap::from([("en",include_str!("../resources/i18n/en.json"))])}
//                     storage_type={StorageType::LocalStorage}
//                     storage_name={"i18nrs".to_string()}
//                     default_language={"en".to_string()}
//                 >
//                     <canvas id="gridworm-canvas" ref={self.canvas_node_ref.clone()} />
//                     {
//                         self.current_scene.html(ctx)
//                     }
//                     <NotificationManager />
//                 </I18nProvider>
//             </div>
//             <footer>
//                 { format!("Rendered: {}", String::from(Date::new_0().to_string())) }
//             </footer>
//             </div>
//         }
//     }
//     fn rendered(&mut self, ctx: &yew::Context<Self>, first_render: bool) {
//         if first_render {
//             ctx.link().send_message(Message::InitWorms);
//         }
//     }
// }
