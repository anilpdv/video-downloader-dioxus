use std::env;
use std::path::Path;

fn main() {
    // Tell cargo to re-run this script if any binary in resources changes
    println!("cargo:rerun-if-changed=resources/");

    // Print the current directory
    println!(
        "cargo:warning=Building from directory: {}",
        env::current_dir().unwrap().display()
    );

    // Check if resources directory exists
    let resources_dir = Path::new("resources");
    if resources_dir.exists() && resources_dir.is_dir() {
        println!("cargo:warning=Resources directory found!");

        // List all files in resources directory
        if let Ok(entries) = std::fs::read_dir(resources_dir) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() {
                    println!("cargo:warning=Including resource: {}", file_path.display());
                }
            }
        }
    } else {
        println!("cargo:warning=Resources directory not found!");
    }
}
