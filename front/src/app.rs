use crate::{component::LightSwitch, scene::Scene};
use yew::{function_component, use_state, Callback, Html};
use yew_router::hooks::use_navigator;

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

    let nav_opt = use_navigator();
    let cs = current_scene.clone();
    let set_scene_cb = Callback::from(move |scene: Scene| {
        if let Some(nav) = &nav_opt {
            nav.replace(&scene.route())
        } else {
            error!("Failed to retrieve the navigator");
        }
        cs.set(scene);
    });

    html! {
        <div id="global">
        <div id="header">
            <a class="header-item" href="http://github.com/Bowarc/storage_server">
                <img src="/resources/github.webp" alt="Github icon" class="icon"/>
            </a>
            <LightSwitch />
            <div class="header-item" id="scene_list">{
                scenes.into_iter().map(|scene|{
                    html!{
                        <button class={format!("scene_button{}", if *current_scene == scene {" current"} else {""})} onclick={
                            let sscb = set_scene_cb.clone();
                            Callback::from(move |_| sscb.emit(scene))
                        }>
                            { format!("{scene}") }
                        </button>
                    }
                }).collect::<Vec<VNode>>()
            }</div>
        </div>
        <div id="content">
            {
                current_scene.html(set_scene_cb)
            }
            <NotificationManager />
        </div>
        <footer>
            { format!("Rendered: {}", String::from(Date::new_0().to_string())) }
        </footer>
        </div>
    }
}
