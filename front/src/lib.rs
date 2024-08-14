use {
    js_sys::Date,
    yew::{html, Component, Context, Html},
};

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
    // Download,
    Contact,
}

pub struct App {
    current_scene: Scene,
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            current_scene: Scene::Home,
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
                <a class="header_item" href="http://github.com/Bowarc/storage_server">
                    <img src="resources/github.webp" alt="Github icon" class="icon"/>
                </a>
                <div id="scene_list" class="header_item">{
                    [ Scene::Home, Scene::Upload, Scene::Contact ].iter().map(|scene|{
                        html!{
                            <button class={format!("scene_button{}", if &self.current_scene == scene {" current"} else{""})} onclick={ctx.link().callback(|_| Message::SwitchScene(*scene))}>
                                { format!("{scene}") }
                            </button>
                        }
                    }).collect::<Vec<yew::virtual_dom::VNode>>()
                }</div>
            </div>
            <div id="content">
                {
                    self.current_scene.html(ctx)
                }
                <component::NotificationManager />
            </div>
            <footer>
                { format!("Rendered: {}", String::from(Date::new_0().to_string())) }
            </footer>
            </div>
        }
    }
}

impl Scene {
    fn html(&self, ctx: &Context<App>) -> yew::virtual_dom::VNode {
        match self {
            Scene::Home => {
                let on_clicked = ctx.link().callback(Message::SwitchScene);
                html! {<><scene::Home {on_clicked} /></>}
            }
            Scene::Upload => html! {<><scene::Upload /></>},
            // Scene::Download => html! {<><scene::Download /></>},
            Scene::Contact => html! {<><scene::Contact /></>},
        }
    }
}

impl std::fmt::Display for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scene::Home => write!(f, "Home"),
            Scene::Upload => write!(f, "Upload"),
            // Scene::Download => write!(f, "Download"),
            Scene::Contact => write!(f, "Contact"),
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn start() {
    yew::Renderer::<App>::new().render();
}
