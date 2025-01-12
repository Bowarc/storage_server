use yew::html;

pub struct Home;

#[derive(Clone, PartialEq, yew::Properties)]
pub struct Props {
    pub on_clicked: yew::Callback<crate::Scene>,
}

impl yew::Component for Home {
    type Message = ();

    type Properties = Props;

    fn create(_ctx: &yew::prelude::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        use crate::Scene;

        let onclick = ctx.props().on_clicked.reform(move |_| Scene::Upload);
        html! {<>
            <div class="home">
                <p class="home_main_title">{
                    "Bowarc's storage server"
                }</p>

                <section class="home_section">
                    <h2 class="home_section_title">{
                        "Welcome"
                    }</h2>
                    <p class="home_section_text">
                        { format!("This file sharing platform doesn't require any login, you can upload files up to {:.0} mb.", crate::scene::upload::SIZE_LIMIT_BYTES as f64 / (1024. * 1024.)) }
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
                                { "Web interface" }
                                <br/>
                                { "Visit " }
                                <button onclick={ onclick }>{ "the upload page" }</button>
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
                                { "Using curl, make a simple file upload request to storage.bowarc.ovh/: " }
                                <pre class="home_section_code_example">
                                    <code>{ "curl --upload-file /path/to/yourfile.ext https://storage.bowarc.ovh/" }</code>
                                </pre>
                                { "This will return a " }
                                <a href="https://en.wikipedia.org/wiki/Universally_unique_identifier">{"uuid"}</a>
                                { " that you can use as url path to acces your file !"}
                                <pre class="home_section_code_example">
                                    <code>{ "curl https://storage.bowarc.ovh/<your uuid> -o myfile.txt" }</code>
                                </pre>
                                { " You can use the -O tag if you add the file name to the path like this: " }
                                <pre class="home_section_code_example">
                                    <code>{ "curl storage.bowarc.ovh/<your uuid>/yourfile.ext -O" }</code>
                                </pre>
                                { "If you try to download a file with a name that doesn't exist, you might see a response like this:" }
                                <pre class="home_section_code_example">
                                    <code>{ "Incorrect file name, did you mean 'yourfile.ext'?" }</code>
                                </pre>
                            </p>
                        </section>

                    </p>
                </section>
            </div>
        </>}
    }
}
