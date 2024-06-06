use yew::html;

pub struct Download;

impl yew::Component for Download {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &yew::prelude::Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        html! {<>
            <p>
            { "Download page" } 
            </p>
        </>}
    }
}
