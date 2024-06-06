use gloo::console::log;
use yew::html;

pub struct Upload;

impl yew::Component for Upload {
    type Message = ();

    type Properties = ();

    fn create(ctx: &yew::prelude::Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        html! {<>
            {"Upload view"}
        </>}
    }
}