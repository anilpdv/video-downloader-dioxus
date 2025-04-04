use crate::server::get_video_info;
use dioxus::prelude::*;
#[component]
pub fn GetInfo() -> Element {
    rsx! {
        div { "Information" }
        button {
            onclick: move |_| async move {
                match get_video_info("wallnut".to_string()).await {
                    Ok(_) => {
                        println!("Downloaded video");
                    }
                    Err(e) => {
                        println!("Error downloading video: {}", e);
                    }
                }
            },
            "Information"
        }
    }
}
