mod upload;
use upload::Upload;
mod contatct;
pub use contatct::Contact;
mod home;
pub use home::Home;
mod not_found;
pub use not_found::NotFound;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Scene {
    Home,
    Upload,
    Contact,
    NotFound
}

impl Scene {
    pub fn html(&self, current_scene: yew::UseStateHandle<Scene>) -> yew::virtual_dom::VNode {
        use yew::html;

        match self {
            Scene::Home => html! {<Home {current_scene}/>},
            Scene::Upload => html! {<Upload />},
            Scene::Contact => html! {<Contact />},
            Scene::NotFound => html! {<NotFound />},
        }
    }
}

impl std::fmt::Display for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scene::Home => write!(f, "Home"),
            Scene::Upload => write!(f, "Upload"),
            Scene::Contact => write!(f, "Contact"),
            Scene::NotFound => write!(f, "Not found"),
        }
    }
}
