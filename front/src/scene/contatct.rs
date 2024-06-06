use yew::html;

pub struct Contact;

impl yew::Component for Contact {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &yew::prelude::Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        html! {<>
            <div class="contact">
                <h1>{"Contact Me"}</h1>
                <p>{"Vous pouvez me contacter sur ces adresses mail:"}</p>
                <ul>
                    // <li>{"GitHub: "}
                        // <a href="https://github.com/Bowarc" target="_blank">{"Bowarc"}</a>{" & "}
                        // <a href="https://github.com/HugoLz" target="_blank">{"HugoLz"}</a>
                    // </li>
                    <li>{"Email: "}
                        <a href="mailto:***REMOVED***">{"***REMOVED***"}</a>{" & "}
                        <a href="mailto:***REMOVED***">{"***REMOVED***"}</a>
                    </li>
                </ul>
            </div>
        </>}
    }
}
