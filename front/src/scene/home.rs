use crate::component;
use yew::html;

pub struct Home;

impl yew::Component for Home {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &yew::prelude::Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        html! { <>
         { "Home page" }
        </>}
    }
}
