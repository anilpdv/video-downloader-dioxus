#[cfg(feature = "server")]
use crate::database::models::Download;
use sqlx::Row;
use sqlx::{Pool, Sqlite};
use std::path::Path;

/// Save a download record to the database
pub async fn save_download(pool: &Pool<Sqlite>, download: &Download) -> Result<i64, sqlx::Error> {
    let query = sqlx::query(
        r#"
        INSERT INTO downloads (
            url, title, filename, file_path, format_type, quality, file_size, 
            download_date, thumbnail_url, video_id, duration
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(&download.url)
    .bind(&download.title)
    .bind(&download.filename)
    .bind(&download.file_path)
    .bind(&download.format_type)
    .bind(&download.quality)
    .bind(download.file_size)
    .bind(download.download_date.map(|dt| dt.unix_timestamp()))
    .bind(&download.thumbnail_url)
    .bind(&download.video_id)
    .bind(download.duration);

    let id = query.fetch_one(pool).await?.get(0);
    Ok(id)
}

/// Get all downloads from the database
pub async fn get_all_downloads(pool: &Pool<Sqlite>) -> Result<Vec<Download>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, url, title, filename, file_path, format_type, quality, file_size,
            download_date, thumbnail_url, video_id, duration
        FROM downloads
        ORDER BY download_date DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut downloads = Vec::with_capacity(rows.len());
    for row in rows {
        let timestamp: Option<i64> = row.get("download_date");
        let download = Download {
            id: row.get("id"),
            url: row.get("url"),
            title: row.get("title"),
            filename: row.get("filename"),
            file_path: row.get("file_path"),
            format_type: row.get("format_type"),
            quality: row.get("quality"),
            file_size: row.get("file_size"),
            download_date: timestamp
                .and_then(|ts| time::OffsetDateTime::from_unix_timestamp(ts).ok()),
            thumbnail_url: row.get("thumbnail_url"),
            video_id: row.get("video_id"),
            duration: row.get("duration"),
        };
        downloads.push(download);
    }

    Ok(downloads)
}

/// Get download by ID
pub async fn get_download_by_id(
    pool: &Pool<Sqlite>,
    id: i64,
) -> Result<Option<Download>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
            id, url, title, filename, file_path, format_type, quality, file_size,
            download_date, thumbnail_url, video_id, duration
        FROM downloads
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let timestamp: Option<i64> = row.get("download_date");
            let download = Download {
                id: row.get("id"),
                url: row.get("url"),
                title: row.get("title"),
                filename: row.get("filename"),
                file_path: row.get("file_path"),
                format_type: row.get("format_type"),
                quality: row.get("quality"),
                file_size: row.get("file_size"),
                download_date: timestamp
                    .and_then(|ts| time::OffsetDateTime::from_unix_timestamp(ts).ok()),
                thumbnail_url: row.get("thumbnail_url"),
                video_id: row.get("video_id"),
                duration: row.get("duration"),
            };
            Ok(Some(download))
        }
        None => Ok(None),
    }
}

/// Delete a download record from the database
pub async fn delete_download(pool: &Pool<Sqlite>, id: i64) -> Result<bool, sqlx::Error> {
    // First get the download to check if the file exists
    let download = get_download_by_id(pool, id).await?;
    if let Some(download) = download {
        // Try to delete the file from disk
        if Path::new(&download.file_path).exists() {
            if let Err(e) = std::fs::remove_file(&download.file_path) {
                tracing::warn!("Failed to delete file {}: {}", download.file_path, e);
                // We continue even if file deletion failed
            }
        }

        // Delete the database record
        let result = sqlx::query("DELETE FROM downloads WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    } else {
        Ok(false)
    }
}

/// Search downloads by title or filename
pub async fn search_downloads(
    pool: &Pool<Sqlite>,
    query: &str,
) -> Result<Vec<Download>, sqlx::Error> {
    // Add wildcards for SQL LIKE
    let search_term = format!("%{}%", query);

    let rows = sqlx::query(
        r#"
        SELECT
            id, url, title, filename, file_path, format_type, quality, file_size,
            download_date, thumbnail_url, video_id, duration
        FROM downloads
        WHERE 
            title LIKE ? OR 
            filename LIKE ? OR
            url LIKE ?
        ORDER BY download_date DESC
        "#,
    )
    .bind(&search_term)
    .bind(&search_term)
    .bind(&search_term)
    .fetch_all(pool)
    .await?;

    let mut downloads = Vec::with_capacity(rows.len());
    for row in rows {
        let timestamp: Option<i64> = row.get("download_date");
        let download = Download {
            id: row.get("id"),
            url: row.get("url"),
            title: row.get("title"),
            filename: row.get("filename"),
            file_path: row.get("file_path"),
            format_type: row.get("format_type"),
            quality: row.get("quality"),
            file_size: row.get("file_size"),
            download_date: timestamp
                .and_then(|ts| time::OffsetDateTime::from_unix_timestamp(ts).ok()),
            thumbnail_url: row.get("thumbnail_url"),
            video_id: row.get("video_id"),
            duration: row.get("duration"),
        };
        downloads.push(download);
    }

    Ok(downloads)
}

/// Get downloads filtered by format type (video or audio)
pub async fn get_downloads_by_format(
    pool: &Pool<Sqlite>,
    format_type: &str,
) -> Result<Vec<Download>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, url, title, filename, file_path, format_type, quality, file_size,
            download_date, thumbnail_url, video_id, duration
        FROM downloads
        WHERE format_type = ?
        ORDER BY download_date DESC
        "#,
    )
    .bind(format_type)
    .fetch_all(pool)
    .await?;

    let mut downloads = Vec::with_capacity(rows.len());
    for row in rows {
        let timestamp: Option<i64> = row.get("download_date");
        let download = Download {
            id: row.get("id"),
            url: row.get("url"),
            title: row.get("title"),
            filename: row.get("filename"),
            file_path: row.get("file_path"),
            format_type: row.get("format_type"),
            quality: row.get("quality"),
            file_size: row.get("file_size"),
            download_date: timestamp
                .and_then(|ts| time::OffsetDateTime::from_unix_timestamp(ts).ok()),
            thumbnail_url: row.get("thumbnail_url"),
            video_id: row.get("video_id"),
            duration: row.get("duration"),
        };
        downloads.push(download);
    }

    Ok(downloads)
}

/// Update file_exists status for all downloads
pub async fn update_file_exists_status(pool: &Pool<Sqlite>) -> Result<Vec<i64>, sqlx::Error> {
    // Get all downloads
    let downloads = get_all_downloads(pool).await?;
    let mut deleted_ids = Vec::new();

    // Check each file and mark ones that don't exist
    for download in downloads {
        if !std::path::Path::new(&download.file_path).exists() {
            tracing::warn!("File not found: {}", download.file_path);
            // Optionally, remove entries with missing files
            if let Ok(deleted) = delete_download(pool, download.id.unwrap()).await {
                if deleted {
                    deleted_ids.push(download.id.unwrap());
                }
            }
        }
    }

    Ok(deleted_ids)
}
