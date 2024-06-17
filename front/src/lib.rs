use gloo::console::log;
use js_sys::Date;
use yew::{html, Component, Context, Html};

mod component;
mod scene;
mod utils;

pub enum Message {
    SwitchScene(Scene), // sao <3
}

#[derive(Clone, Copy, PartialEq)]
pub enum Scene {
    Home,
    Upload,
    Download,
    Dashboard,
    Contact,
    Void,
}

pub struct App {
    current_scene: Scene,
    canvas_node_ref: yew::NodeRef,
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            current_scene: Scene::Home,
            canvas_node_ref: yew::NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::SwitchScene(scene) => {
                self.current_scene = scene;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="global">
            <div id="header">
                <a class="header_item" href="http://github.com/Bowarc/wasm_portfolio">
                    <img src="resources/github.webp" alt="Github icon" class="icon"/>
                </a>
                <div id="scene_list" class="header_item">{
                    [ Scene::Home, Scene::Upload, Scene::Download, Scene::Dashboard, Scene::Contact, Scene::Void ].iter().map(|scene|{
                        html!{
                            <button class={format!("scene_button{}", if  &self.current_scene == scene {" current"}else{""})} onclick={ctx.link().callback(|_| Message::SwitchScene(*scene))}>
                                { format!("{scene}") }
                            </button>
                        }
                    }).collect::<Vec<yew::virtual_dom::VNode>>()
                }</div>
            </div>
            <div id="content">
                {
                    self.current_scene.html()
                }
            </div>
            <footer>
                { format!("Rendered: {}", String::from(Date::new_0().to_string())) }
            </footer>
            </div>
        }
    }
}

impl Scene {
    fn html(&self) -> yew::virtual_dom::VNode {
        match self {
            Scene::Home => html! {<><scene::Home /></>},
            Scene::Upload => html! {<><scene::Upload /></>},
            Scene::Download => html! {<><scene::Download /></>},
            Scene::Dashboard => html! {<><scene::Dashboard /></>},
            Scene::Contact => html! {<><scene::Contact /></>},
            Scene::Void => html! {<></>},
        }
    }
}

impl std::fmt::Display for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scene::Home => write!(f, "Home"),
            Scene::Upload => write!(f, "Upload"),
            Scene::Download => write!(f, "Download"),
            Scene::Dashboard => write!(f, "Dashboard"),
            Scene::Contact => write!(f, "Contact"),
            Scene::Void => write!(f, "Void"),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn start() {
    yew::Renderer::<App>::new().render();
}
