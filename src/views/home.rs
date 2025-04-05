use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{fa_brands_icons::FaGithub, fa_solid_icons::FaDownload},
    Icon,
};
use dioxus_router::prelude::Link;

#[component]
pub fn Home() -> Element {
    const heroImage: Asset = asset!("/assets/hero-image.png");
    rsx! {
        div { class: "min-h-screen flex flex-col",
            // Hero section
            div { class: "container mx-auto px-4 py-20 flex flex-col md:flex-row items-center justify-between",
                // Text content
                div { class: "md:w-1/2 mb-10 md:mb-0 text-center md:text-left",
                    h1 { class: "text-4xl md:text-5xl font-bold mb-6 text-text-primary",
                        "Download Videos & Audio"
                    }
                    p { class: "text-xl text-text-secondary mb-8 leading-relaxed",
                        "A simple, elegant desktop application to download videos and audio from popular platforms."
                    }
                    div { class: "flex flex-col sm:flex-row gap-4 justify-center md:justify-start",
                        Link {
                            to: "/download",
                            class: "inline-flex items-center justify-center bg-accent-teal text-text-invert px-6 py-3 rounded-lg shadow-sm hover:bg-opacity-80 transition-colors",
                            Icon {
                                icon: FaDownload,
                                width: 16,
                                height: 16,
                                class: "mr-2",
                            }
                            "Start Downloading"
                        }
                        a {
                            href: "https://github.com/yourusername/video-downloader",
                            target: "_blank",
                            class: "inline-flex items-center justify-center bg-background-medium text-text-primary px-6 py-3 rounded-lg shadow-sm hover:bg-background-hover border border-border transition-colors",
                            Icon {
                                icon: FaGithub,
                                width: 16,
                                height: 16,
                                class: "mr-2",
                            }
                            "View on GitHub"
                        }
                    }
                }

                // Illustration/preview
                div { class: "md:w-1/2 max-w-lg",
                    img {
                        src: heroImage,
                        alt: "Video Downloader Preview",
                        class: "w-full drop-shadow-lg",
                    }
                }
            }

            // Features section
            div { class: "bg-background-card border-t border-b border-border py-16",
                div { class: "container mx-auto px-4",
                    h2 { class: "text-3xl font-bold text-center mb-12 text-text-primary",
                        "Key Features"
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-8",
                        // Feature 1
                        div { class: "p-6 rounded-xl border border-border bg-background-medium shadow-sm",
                            div { class: "w-12 h-12 bg-accent-teal bg-opacity-20 rounded-full flex items-center justify-center mb-4",
                                span { class: "text-accent-teal text-2xl", "🎬" }
                            }
                            h3 { class: "text-xl font-semibold mb-2 text-text-primary",
                                "Multiple Formats"
                            }
                            p { class: "text-text-secondary",
                                "Download videos in MP4 format or extract audio in MP3 format with various quality options."
                            }
                        }

                        // Feature 2
                        div { class: "p-6 rounded-xl border border-border bg-background-medium shadow-sm",
                            div { class: "w-12 h-12 bg-accent-amber bg-opacity-20 rounded-full flex items-center justify-center mb-4",
                                span { class: "text-accent-amber text-2xl", "⚡" }
                            }
                            h3 { class: "text-xl font-semibold mb-2 text-text-primary",
                                "Fast & Efficient"
                            }
                            p { class: "text-text-secondary",
                                "Optimized download engine ensures fast downloads with minimal system resource usage."
                            }
                        }

                        // Feature 3
                        div { class: "p-6 rounded-xl border border-border bg-background-medium shadow-sm",
                            div { class: "w-12 h-12 bg-accent-rose bg-opacity-20 rounded-full flex items-center justify-center mb-4",
                                span { class: "text-accent-rose text-2xl", "🔒" }
                            }
                            h3 { class: "text-xl font-semibold mb-2 text-text-primary",
                                "Privacy Focused"
                            }
                            p { class: "text-text-secondary",
                                "No tracking, no accounts, and no personal data collection. Your downloads stay private."
                            }
                        }
                    }
                }
            }

            // How it works section
            div { class: "container mx-auto px-4 py-16",
                h2 { class: "text-3xl font-bold text-center mb-12 text-text-primary",
                    "How It Works"
                }
                div { class: "max-w-3xl mx-auto",
                    // Step 1
                    div { class: "flex flex-col md:flex-row items-center mb-12",
                        div { class: "md:w-16 w-12 h-12 md:h-16 rounded-full bg-accent-teal bg-opacity-20 flex items-center justify-center flex-shrink-0 mb-4 md:mb-0 md:mr-6",
                            span { class: "text-accent-teal text-2xl font-bold", "1" }
                        }
                        div { class: "md:flex-1 text-center md:text-left",
                            h3 { class: "text-xl font-semibold mb-2 text-text-primary",
                                "Paste the Video URL"
                            }
                            p { class: "text-text-secondary",
                                "Copy the URL of the video you want to download and paste it into the application."
                            }
                        }
                    }

                    // Step 2
                    div { class: "flex flex-col md:flex-row items-center mb-12",
                        div { class: "md:w-16 w-12 h-12 md:h-16 rounded-full bg-accent-teal bg-opacity-20 flex items-center justify-center flex-shrink-0 mb-4 md:mb-0 md:mr-6",
                            span { class: "text-accent-teal text-2xl font-bold", "2" }
                        }
                        div { class: "md:flex-1 text-center md:text-left",
                            h3 { class: "text-xl font-semibold mb-2 text-text-primary",
                                "Choose Format & Quality"
                            }
                            p { class: "text-text-secondary",
                                "Select whether you want to download the video (MP4) or just the audio (MP3) and choose your preferred quality."
                            }
                        }
                    }

                    // Step 3
                    div { class: "flex flex-col md:flex-row items-center",
                        div { class: "md:w-16 w-12 h-12 md:h-16 rounded-full bg-accent-teal bg-opacity-20 flex items-center justify-center flex-shrink-0 mb-4 md:mb-0 md:mr-6",
                            span { class: "text-accent-teal text-2xl font-bold", "3" }
                        }
                        div { class: "md:flex-1 text-center md:text-left",
                            h3 { class: "text-xl font-semibold mb-2 text-text-primary",
                                "Download & Enjoy"
                            }
                            p { class: "text-text-secondary",
                                "Click the download button and wait for the process to complete. Your file will be saved to your downloads folder."
                            }
                        }
                    }
                }
            }

            // CTA section
            div { class: "bg-background-card border-t border-border py-16",
                div { class: "container mx-auto px-4 text-center",
                    h2 { class: "text-3xl font-bold mb-6 text-text-primary", "Ready to Download?" }
                    p { class: "text-xl text-text-secondary mb-8 max-w-2xl mx-auto",
                        "Start downloading your favorite videos and music in just a few clicks."
                    }
                    Link {
                        to: "/download",
                        class: "inline-flex items-center justify-center bg-accent-teal text-text-invert px-8 py-4 rounded-lg shadow-sm hover:bg-opacity-80 transition-colors text-lg",
                        "Get Started Now"
                    }
                }
            }
        }
    }
}
