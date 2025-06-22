use gloo::utils::window;
use yew::{function_component, html, Callback, Html};

use super::Scene;

#[derive(PartialEq, yew::Properties)]
pub struct Props {
    pub set_scene_cb: Callback<Scene>,
}

#[function_component]
pub fn Home(props: &Props) -> Html {
    let url = {
        let location = window().location();
        format!(
            "{}//{}",
            location.protocol().unwrap_or_else(|_| {
                warn!("Cannot get current protocol, falling back to default");
                String::from("https:")
            }),
            location.host().unwrap_or_else(|_| {
                warn!("Cannot get current url, falling back to default");
                String::from("storage.bowarc.ovh/")
            })
        )
    };
    html! {<>
        <div class="home">
            <p class="home_main_title">{
                "Bowarc's storage server, share files easily"
            }</p>

            <section class="home_section">
                <h2 class="home_section_title">{
                    "Welcome"
                }</h2>
                <p class="home_section_text">
                    { format!("This is the web interface for my file storage server, you can upload files up to {}.", mem::format(crate::scene::upload::SIZE_LIMIT_BYTES, &mem::Prefix::Binary)) }
                    <br/>
                    { "Once uploaded, your will receive a shareable download link for you to send to anyone you want."}
                </p>
            </section>

            <section class="home_section">
                <h2 class="home_section_title">{
                    "How to use"
                }</h2>

                <p class="home_section_text">
                    <section class="home_section">
                        <h3 class="home_section_title">{
                            "Web interface"
                        }</h3>

                        <p class="home_section_text">
                            { "Visit " }
                            <button onclick={
                                let sscb = props.set_scene_cb.clone();

                                Callback::from(move |_| sscb.emit(Scene::Upload))
                            }>{ "the upload page" }</button>
                            { ", select a file, hit upload."}
                            <br/>
                            {"This will give you a link for you to share to anyone !"}
                        </p>
                    </section>

                    <section class="home_section">
                        <h3 class="home_section_title">{
                            "Command line"
                        }</h3>
                        <p class="home_section_text">
                            { format!("Using curl, make a simple file upload request to {url}: ") }
                            {{
                                if url.contains("storage.bowarc.ovh"){
                                // if true {
                                    html!{<>
                                        <br />
                                        <br />
                                        {"Note: On my instance (storage.bowarc.ovh/), you'll need to use credentials to upload."}
                                        <br />
                                        { "You can do that by adding "}
                                        <mark>{ " -u username:password" }</mark>
                                        { " to your curl command."}
                                        <br />
                                        { "You can deploy your own by visiting " }
                                        <a href="https://github.com/bowarc/storage_server">{ "the project on github" }</a>
                                        <br />

                                        </>}
                                }else{
                                    html!{}
                                }
                            }}
                            <br />
                            <pre class="home_section_code_example">
                                <code>{ format!("curl --upload-file /path/to/yourfile.ext {url}/") }</code>
                            </pre>
                            { "This will return a " }
                            <a href="https://en.wikipedia.org/wiki/Universally_unique_identifier">{"uuid"}</a>
                            { "(v4) that you can use to acces your file !"}
                            <pre class="home_section_code_example">
                                <code>{ format!("curl {url}/<your uuid> -o myfile.txt") }</code>
                            </pre>
                            { "You can use the -O tag if you add the file name to the path like this: " }
                            <pre class="home_section_code_example">
                                <code>{ format!("curl {url}/<your uuid>/yourfile.ext -O") }</code>
                            </pre>
                            { "If you try to download a file with a name that doesn't exist, you might see a response like this:" }
                            <pre class="home_section_code_example">
                                <code>{ "Incorrect file name, did you mean 'yourfile.ext'?" }</code>
                            </pre>
                            { "To delete a file, just send a DELETE request like so: " }
                            <pre class="home_section_code_example">
                                <code>{ format!("curl {url}/<your uuid> -X DELETE") }</code>
                            </pre>
                        </p>
                    </section>

                </p>
            </section>
        </div>
    </>}
}
