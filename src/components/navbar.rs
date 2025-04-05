use crate::Route;
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{
        bs_icons::{BsHouseDoorFill, BsInfoCircleFill, BsNewspaper, BsSearch},
        fa_solid_icons::{FaDownload, FaMusic, FaVideo},
    },
    Icon,
};

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    let mut show_labels = use_signal(|| true);
    let nav = navigator();

    // Get current path to highlight active link
    let route = use_route::<Route>();

    // Determine if routes are active based on the current route
    let is_home = matches!(route, Route::Home {});
    let is_blog = matches!(route, Route::Blog { .. });
    let is_download = matches!(route, Route::Download {});
    let is_downloads = matches!(route, Route::Downloads {});
    let is_getinfo = matches!(route, Route::GetInfo {});
    let is_search = matches!(route, Route::Search {});

    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        // Main layout container - sidebar + content
        div { class: "flex h-screen w-full overflow-hidden bg-background",
            // Sidebar
            div {
                class: "sidebar bg-background-sidebar text-text-primary transition-all duration-300 border-r border-border-dark",
                class: if show_labels() { "w-60" } else { "w-20" },
                // App title/logo
                div {
                    class: "flex items-center p-4 border-b border-border-dark",
                    class: if show_labels() { "justify-between" } else { "justify-center" },
                    // Logo and title
                    div { class: "flex items-center",
                        span { class: "text-primary-400 text-2xl mr-2", "▶" }
                        if show_labels() {
                            span { class: "font-bold text-lg text-text-primary", "Youtube DL" }
                        }
                    }
                    // Toggle sidebar width button
                    button {
                        class: "text-text-muted hover:text-text-primary p-1 rounded-full transition-colors duration-200",
                        onclick: move |_| show_labels.set(!show_labels()),
                        span {
                            class: "block transition-transform",
                            class: if show_labels() { "" } else { "rotate-180" },
                            "←"
                        }
                    }
                }
                // Navigation links
                nav { class: "mt-6 px-2",
                    // Home link
                    div {
                        class: "flex items-center py-3 px-3 mb-2 rounded-lg transition-all duration-200",
                        class: if !show_labels() { "justify-center" } else { "" },
                        class: if is_home { "bg-primary-600 text-text-primary shadow-glow" } else { "text-text-muted hover:bg-background-hover hover:text-text-primary" },
                        onclick: move |_| {
                            nav.replace(Route::Home {});
                        },
                        div { class: if show_labels() { "mr-3" } else { "" },
                            Icon {
                                icon: BsHouseDoorFill,
                                width: 20,
                                height: 20,
                            }
                        }
                        if show_labels() {
                            span { "Home" }
                        }
                    }
                    // Blog link
                    div {
                        class: "flex items-center py-3 px-3 mb-2 rounded-lg transition-all duration-200",
                        class: if !show_labels() { "justify-center" } else { "" },
                        class: if is_blog { "bg-primary-600 text-text-primary shadow-glow" } else { "text-text-muted hover:bg-background-hover hover:text-text-primary" },
                        onclick: move |_| {
                            nav.replace(Route::Blog { id: 1 });
                        },
                        div { class: if show_labels() { "mr-3" } else { "" },
                            Icon { icon: BsNewspaper, width: 20, height: 20 }
                        }
                        if show_labels() {
                            span { "Blog" }
                        }
                    }
                    // Download link
                    div {
                        class: "flex items-center py-3 px-3 mb-2 rounded-lg transition-all duration-200",
                        class: if !show_labels() { "justify-center" } else { "" },
                        class: if is_download { "bg-primary-600 text-text-primary shadow-glow" } else { "text-text-muted hover:bg-background-hover hover:text-text-primary" },
                        onclick: move |_| {
                            nav.replace(Route::Download {});
                        },
                        div { class: if show_labels() { "mr-3" } else { "" },
                            Icon { icon: FaDownload, width: 20, height: 20 }
                        }
                        if show_labels() {
                            span { "Download" }
                        }
                    }
                    // Search link (new)
                    div {
                        class: "flex items-center py-3 px-3 mb-2 rounded-lg transition-all duration-200",
                        class: if !show_labels() { "justify-center" } else { "" },
                        class: if is_search { "bg-primary-600 text-text-primary shadow-glow" } else { "text-text-muted hover:bg-background-hover hover:text-text-primary" },
                        onclick: move |_| {
                            nav.replace(Route::Search {});
                        },
                        div { class: if show_labels() { "mr-3" } else { "" },
                            Icon { icon: BsSearch, width: 20, height: 20 }
                        }
                        if show_labels() {
                            span { "Search" }
                        }
                    }
                    // My Downloads link
                    div {
                        class: "flex items-center py-3 px-3 mb-2 rounded-lg transition-all duration-200",
                        class: if !show_labels() { "justify-center" } else { "" },
                        class: if is_downloads { "bg-primary-600 text-text-primary shadow-glow" } else { "text-text-muted hover:bg-background-hover hover:text-text-primary" },
                        onclick: move |_| {
                            nav.replace(Route::Downloads {});
                        },
                        div { class: if show_labels() { "mr-3" } else { "" },
                            Icon { icon: FaMusic, width: 20, height: 20 }
                        }
                        if show_labels() {
                            span { "My Downloads" }
                        }
                    }
                    // Get Info link
                    div {
                        class: "flex items-center py-3 px-3 mb-2 rounded-lg transition-all duration-200",
                        class: if !show_labels() { "justify-center" } else { "" },
                        class: if is_getinfo { "bg-primary-600 text-text-primary shadow-glow" } else { "text-text-muted hover:bg-background-hover hover:text-text-primary" },
                        onclick: move |_| {
                            nav.replace(Route::GetInfo {});
                        },
                        div { class: if show_labels() { "mr-3" } else { "" },
                            Icon {
                                icon: BsInfoCircleFill,
                                width: 20,
                                height: 20,
                            }
                        }
                        if show_labels() {
                            span { "Get Info" }
                        }
                    }
                }
            }
            // Main content area
            div { class: "flex-1 overflow-auto bg-background p-6", Outlet::<Route> {} }
        }
    }
}
