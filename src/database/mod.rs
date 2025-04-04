#[cfg(feature = "server")]
pub mod models;
#[cfg(feature = "server")]
pub mod schema;

#[cfg(feature = "server")]
pub use models::*;
#[cfg(feature = "server")]
pub use schema::*;

#[cfg(feature = "server")]
use sqlx::{sqlite::SqlitePool, Executor, Pool, Sqlite};
#[cfg(feature = "server")]
use std::path::PathBuf;
#[cfg(feature = "server")]
use std::sync::OnceLock;

/// The global database connection pool
#[cfg(feature = "server")]
static DB_POOL: OnceLock<Pool<Sqlite>> = OnceLock::new();

/// Initialize the database
#[cfg(feature = "server")]
pub async fn init_database() -> Result<Pool<Sqlite>, sqlx::Error> {
    // Check for existing pool
    if let Some(pool) = DB_POOL.get() {
        return Ok(pool.clone());
    }

    // IMPORTANT: For desktop apps, we need a reliable database that works across sessions
    #[cfg(feature = "desktop")]
    {
        let db_path = get_desktop_database_path();

        // Use memory database if we can't create a file database
        if db_path == PathBuf::from(":memory:") {
            println!(
                "WARNING: Using in-memory database - history will not persist between sessions"
            );
            return get_memory_database().await;
        }

        println!("Using database at: {}", db_path.display());
        let db_url = format!("sqlite:{}", db_path.display());

        match SqlitePool::connect(&db_url).await {
            Ok(pool) => {
                // Run migrations
                if let Err(e) = run_migrations(&pool).await {
                    println!("Migration error: {}", e);
                    return get_memory_database().await;
                }

                // Store in global static
                let _ = DB_POOL.set(pool.clone());
                return Ok(pool);
            }
            Err(e) => {
                println!(
                    "Database connection error: {} for path {}",
                    e,
                    db_path.display()
                );
                return get_memory_database().await;
            }
        }
    }

    // For non-desktop builds, just use in-memory database
    #[cfg(not(feature = "desktop"))]
    {
        return get_memory_database().await;
    }
}

/// Get a connection to the database
#[cfg(feature = "server")]
pub async fn get_database() -> Result<Pool<Sqlite>, sqlx::Error> {
    if let Some(pool) = DB_POOL.get() {
        Ok(pool.clone())
    } else {
        init_database().await
    }
}

/// Run database migrations
#[cfg(feature = "server")]
async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    // Create tables if they don't exist
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS downloads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            title TEXT,
            filename TEXT NOT NULL,
            file_path TEXT NOT NULL,
            format_type TEXT NOT NULL,
            quality TEXT NOT NULL,
            file_size INTEGER,
            download_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            thumbnail_url TEXT,
            video_id TEXT,
            duration INTEGER
        );
        "#,
    )
    .await?;

    Ok(())
}

/// Get the path to the database file
#[cfg(feature = "server")]
fn get_desktop_database_path() -> PathBuf {
    // Always use ~/Documents/youtube_downloader folder for desktop app
    if let Some(home_dir) = dirs::home_dir() {
        let app_dir = home_dir.join("Documents").join("youtube_downloader");

        // Create directory with proper permissions
        if !app_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&app_dir) {
                println!("ERROR: Could not create app directory: {}", e);
                return PathBuf::from(":memory:");
            }

            // Set directory permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Err(e) =
                    std::fs::set_permissions(&app_dir, std::fs::Permissions::from_mode(0o755))
                {
                    println!("ERROR: Could not set directory permissions: {}", e);
                    return PathBuf::from(":memory:");
                }
            }
        }

        // Create and test the database file
        let db_path = app_dir.join("downloads.db");

        // If file doesn't exist, try to create it
        if !db_path.exists() {
            // Try to create an empty file first
            match std::fs::File::create(&db_path) {
                Ok(file) => {
                    // Close the file handle
                    drop(file);

                    // Set file permissions
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Err(e) = std::fs::set_permissions(
                            &db_path,
                            std::fs::Permissions::from_mode(0o644),
                        ) {
                            println!("ERROR: Could not set file permissions: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("ERROR: Could not create database file: {}", e);
                    return PathBuf::from(":memory:");
                }
            }
        }

        // Final check - make sure the file is writable
        match std::fs::OpenOptions::new().write(true).open(&db_path) {
            Ok(_) => {
                println!("Database file is writable: {}", db_path.display());
                return db_path;
            }
            Err(e) => {
                println!(
                    "ERROR: Database file is not writable: {} - {}",
                    db_path.display(),
                    e
                );
                return PathBuf::from(":memory:");
            }
        }
    }

    // Fallback to in-memory database
    println!("WARNING: Could not determine home directory, using in-memory database");
    PathBuf::from(":memory:")
}

/// Get an in-memory database connection - useful when file permissions are an issue
#[cfg(feature = "server")]
pub async fn get_memory_database() -> Result<Pool<Sqlite>, sqlx::Error> {
    println!("Creating in-memory SQLite database");
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Run migrations
    if let Err(e) = run_migrations(&pool).await {
        println!("Warning: Migration error on in-memory database: {}", e);
    }

    Ok(pool)
}
