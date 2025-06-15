use gloo::utils::window;
use yew::{function_component, html, use_effect_with, use_node_ref, use_state, Callback, Html};

#[function_component]
pub fn LightSwitch() -> Html {
    // let (_i18n, switch_cb) = use_translation();
    let theme = use_state(|| "light".to_string());

    let toggle = use_node_ref();

    let theme_clone = theme.clone();
    let toggle_clone = toggle.clone();
    let set_theme = Callback::from(move |v: String| {
        let w = window();

        let Some(d_e) = w.document().and_then(|d| d.document_element()) else {
            error!("Failed to set dark theme due to: Could not get Document Element");
            return;
        };

        if v != "light" && v != "dark" {
            error!(format!("Unknown theme: {v}"));
            return;
        }

        if let Err(e) = d_e.set_attribute("data-theme", &v) {
            error!(format!("Failed to set {v} theme due to: {e:?}"))
        }

        let input = toggle_clone.cast::<web_sys::HtmlInputElement>().unwrap();
        // No confusion, set toggle state based on theme
        match v.as_str() {
            "light" => {
                input.set_checked(true);
            }
            "dark" => {
                input.set_checked(false);
            }
            _ => unreachable!(),
        }

        'storage: {
            let Some(s) = w.local_storage().ok().flatten() else {
                break 'storage;
            };

            if let Err(e) = s.set_item("theme", v.as_str()) {
                error!(format!(
                    "Failed to write theme to local storage due to: {e:?}"
                ));
            }
        }

        theme_clone.set(v);
    });

    let set_theme_clone = set_theme.clone();
    use_effect_with((), move |_| 'theme_detection: {
        let w = window(); // This is dumb, but that function seems to not be trivial, so I don't know if calling it a lot can impact perfs

        // Load saved theme
        if let Some(saved_theme) = w
            .local_storage()
            .ok()
            .flatten()
            .and_then(|s| s.get_item("theme").ok().flatten())
        {
            set_theme_clone.emit(saved_theme);
            break 'theme_detection;
        }

        // Set theme based on prefered-color-scheme value (often set by browsers)
        if w.match_media("(prefers-color-scheme: dark)")
            .ok()
            .flatten()
            .map(|query| query.matches())
            .unwrap_or(false)
        {
            set_theme_clone.emit(String::from("dark"));
            break 'theme_detection;
        }else{
            set_theme_clone.emit(String::from("light"));
            break 'theme_detection;
        }
    });

    html! {<div class="header-item" id="theme-switch">
        <input type="checkbox" id="theme-switch-checkbox" ref={toggle.clone()} />
        <div class="display">
          <label for="toggle" onclick={
            Callback::from(move |_|{
              match theme.as_str(){
                  "light" => {
                      set_theme.emit(String::from("dark"))
                  }
                  // dark or any (if bug, default to light)
                  _ => {
                      set_theme.emit(String::from("light"))
                  }
              }
            })
          }>
            <div class="circle">
              <svg class="sun" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M12 2.25a.75.75 0 0 1 .75.75v2.25a.75.75 0 0 1-1.5 0V3a.75.75 0 0 1 .75-.75ZM7.5 12a4.5 4.5 0 1 1 9 0 4.5 4.5 0 0 1-9 0ZM18.894 6.166a.75.75 0 0 0-1.06-1.06l-1.591 1.59a.75.75 0 1 0 1.06 1.061l1.591-1.59ZM21.75 12a.75.75 0 0 1-.75.75h-2.25a.75.75 0 0 1 0-1.5H21a.75.75 0 0 1 .75.75ZM17.834 18.894a.75.75 0 0 0 1.06-1.06l-1.59-1.591a.75.75 0 1 0-1.061 1.06l1.59 1.591ZM12 18a.75.75 0 0 1 .75.75V21a.75.75 0 0 1-1.5 0v-2.25A.75.75 0 0 1 12 18ZM7.758 17.303a.75.75 0 0 0-1.061-1.06l-1.591 1.59a.75.75 0 0 0 1.06 1.061l1.591-1.59ZM6 12a.75.75 0 0 1-.75.75H3a.75.75 0 0 1 0-1.5h2.25A.75.75 0 0 1 6 12ZM6.697 7.757a.75.75 0 0 0 1.06-1.06l-1.59-1.591a.75.75 0 0 0-1.061 1.06l1.59 1.591Z" />
              </svg>
              <svg class="moon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
                <path fill-rule="evenodd"
                  d="M9.528 1.718a.75.75 0 0 1 .162.819A8.97 8.97 0 0 0 9 6a9 9 0 0 0 9 9 8.97 8.97 0 0 0 3.463-.69.75.75 0 0 1 .981.98 10.503 10.503 0 0 1-9.694 6.46c-5.799 0-10.5-4.7-10.5-10.5 0-4.368 2.667-8.112 6.46-9.694a.75.75 0 0 1 .818.162Z"
                  clip-rule="evenodd" />
              </svg>
            </div>
          </label>
        </div>
    </div>}
}
