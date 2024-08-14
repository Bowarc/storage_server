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
            </div>

            <div style="font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px;">
                <h1>{ "Welcome to Your Storage Server" }</h1>
                <p>
                    { "Our platform provides a secure and efficient way to upload and download files. You can choose between using the command line with `curl` or our intuitive web interface. Here's how you can get started:" }
                </p>

                <h2>{ "Uploading Files" }</h2>
                <p>
                    { "You can upload files in two ways: using the command line with `curl` or through our web interface." }
                </p>

                <h3>{ "Using the Command Line" }</h3>
                <pre style="background-color: #f4f4f4; padding: 10px; border-radius: 5px;">
                    <code>{ "curl --upload-file /path/to/yourfile.ext http://yourserver.com/" }</code>
                </pre>
                <p>
                    { "Replace `/path/to/yourfile.ext` with the path to the file you wish to upload, and `<filename>` with the desired name for your file on the server." }
                </p>

                <h3>{ "Using the Web Interface" }</h3>
                <p>
                    { "Navigate to the " }<a href="#" onclick={onclick}>{ "Upload Page" }</a>
                    { " and select your file. Click 'Upload' to upload your file." }
                </p>

                <h2>{ "Downloading Files" }</h2>
                <p>
                    { "After uploading a file, you'll receive a unique UUID that identifies your file. Use this UUID to download your file." }
                </p>

                <h3>{ "Using the Command Line" }</h3>
                <pre style="background-color: #f4f4f4; padding: 10px; border-radius: 5px;">
                    <code>{ "curl http://yourserver.com/<uuid> -o yourfile.ext" }</code>
                </pre>
                <p>
                    { "Replace `<uuid>` with the UUID provided after upload, and `yourfile.ext` with the desired name for the file on your local machine." }
                </p>

                <h3>{ "Direct Download" }</h3>
                <p>
                    { "Enter the download link directly into your browser's address bar to download the file." }
                </p>

                <h2>{ "Getting Started" }</h2>
                <ol>
                    <li>{ "Upload your file using your preferred method." }</li>
                    <li>{ "Receive a UUID for your uploaded file." }</li>
                    <li>{ "Share the UUID with anyone you wish to have access to the file." }</li>
                    <li>{ "Download the file using the UUID." }</li>
                </ol>

                <h2>{ "Security and Privacy" }</h2>
                <p>
                    { "Your data's security and privacy are our top priorities. Each file is uniquely identified and accessible only through the UUID." }
                </p>

                <h2>{ "Support" }</h2>
                <p>
                    { "If you have any questions or encounter any issues, please contact our support team at " }
                    <a href="mailto:support@yourserver.com">{ "support@yourserver.com" }</a>
                    { "." }
                </p>
            </div>
        </>}
    }
}
