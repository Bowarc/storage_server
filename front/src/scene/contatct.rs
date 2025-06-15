use yew::{function_component, html, Html};

#[function_component]
pub fn Contact() -> Html {
    html! {<>
        <div class="contact">
            <h1>{"Contact Me"}</h1>
            <p>{"Vous pouvez me contacter sur ces adresses mail:"}</p>
            <ul>
                // <li>{"GitHub: "}
                    // <a href="https://github.com/Bowarc" target="_blank">{"Bowarc"}</a>{" & "}
                    // <a href="https://github.com/HugoLz" target="_blank">{"HugoLz"}</a>
                // </li>
            </ul>
        </div>
    </>}
}
