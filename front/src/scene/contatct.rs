use yew::{function_component, html, Html};

#[function_component]
pub fn Contact() -> Html {

    html! {<div id="contact">
        <h1>{"Contact Me"}</h1>
        <p>{"You can reach me at: "}</p>
        <table>
            <tr>
                <th>{
                    "Email:"
                }</th>
                <td>
                    <a href="mailto:contact@bowarc.ovh">{"contact@bowarc.ovh"}</a>
                </td>
            </tr>
        </table>
    </div>}
}
