#![recursion_limit = "256"]

use dioxus::prelude::*;

use components::Navbar;
use views::{Blog, Download, Downloads, GetInfo, Home, Search};

mod components;
mod database;
mod server;
mod views;

// Add the common module to the root
pub mod common;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/download")]
    Download {},
    #[route("/downloads")]
    Downloads {},
    #[route("/getinfo")]
    GetInfo {},
    #[route("/search")]
    Search {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[cfg(feature = "server")]
use database::init_database;

fn main() {
    // Print feature flags for debugging
    println!("Starting application with features:");
    println!("  server: {}", cfg!(feature = "server"));
    println!("  desktop: {}", cfg!(feature = "desktop"));
    println!("  web: {}", cfg!(feature = "web"));

    // Initialize database if server feature is enabled
    #[cfg(feature = "server")]
    {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            if let Err(e) = init_database().await {
                eprintln!("Database initialization error: {}", e);
                println!("Will continue with in-memory database");
            }
        });
    }

    // Launch the app based on target platform
    #[cfg(feature = "desktop")]
    {
        LaunchBuilder::desktop().launch(App);
    }

    #[cfg(feature = "web")]
    {
        LaunchBuilder::web().launch(App);
    }

    #[cfg(not(any(feature = "desktop", feature = "web")))]
    {
        LaunchBuilder::new().launch(App);
    }
}

#[component]
fn App() -> Element {
    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}
