use crate::server::download::handlers::info::get_video_info;
use dioxus::prelude::*;

#[component]
pub fn GetInfo() -> Element {
    let result = use_signal(|| String::from("Click to load info"));

    let get_info = move |_| {
        let mut result = result.clone();
        async move {
            result.set(String::from("Loading..."));

            // Call the server function
            match get_video_info("wallnut".to_string()).await {
                Ok(response) => result.set(format!("Success: {}", response)),
                Err(e) => result.set(format!("Error: {:?}", e)),
            }
        }
    };

    rsx! {
        div { class: "p-4",
            h1 { class: "text-2xl mb-4", "Video Info" }
            button {
                class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
                onclick: get_info,
                "Get Info"
            }
            p { class: "mt-4", "{result}" }
        }
    }
}
