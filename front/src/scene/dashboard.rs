use gloo::console::log;
use yew::html;

pub struct Dashboard;

impl yew::Component for Dashboard {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &yew::prelude::Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        html! {<>
            {"Dashboard view"}
        </>}
    }
}
