use std::path::PathBuf;

use sqlx::SqlitePool;
use url::Url;

use crate::error::AppError;
use crate::models::reader::{
    Bookmark, BookmarkInput, ProgressInput, ReaderSettings, ReadingProgress,
};
use crate::utils::now_ms;

const MAX_EPUB_BYTES: u64 = 200 * 1024 * 1024;
const MAX_LOCATION_JSON_CHARS: usize = 10_000;
const MAX_SETTINGS_JSON_CHARS: usize = 20_000;
const MAX_LABEL_CHARS: usize = 200;
const READER_SCOPES: [&str; 5] = ["global", "epub", "pdf", "txt", "markdown"];

pub async fn read_epub_bytes(pool: &SqlitePool, book_id: i64) -> Result<Vec<u8>, AppError> {
    let mut connection = pool.acquire().await?;
    let row = crate::repositories::book_repository::get_row(&mut connection, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("书籍 {book_id} 不存在")))?;
    drop(connection);

    if row.format != "epub" {
        return Err(AppError::Validation(
            "该书籍不是 EPUB，当前阅读器尚未接入此格式".to_string(),
        ));
    }

    let path = PathBuf::from(&row.file_path);
    let metadata = std::fs::metadata(&path).map_err(map_io_error)?;
    if !metadata.is_file() {
        return Err(AppError::Validation(
            "书籍文件不存在或不可访问，请重新定位".to_string(),
        ));
    }
    if metadata.len() > MAX_EPUB_BYTES {
        return Err(AppError::Validation("EPUB 文件超过 200MB 上限".to_string()));
    }

    tokio::task::spawn_blocking(move || std::fs::read(&path))
        .await
        .map_err(|error| AppError::Internal(format!("读取 EPUB 失败：{error}")))?
        .map_err(map_io_error)
}

pub async fn get_progress(
    pool: &SqlitePool,
    book_id: i64,
) -> Result<Option<ReadingProgress>, AppError> {
    Ok(sqlx::query_as::<_, ReadingProgress>(
        "SELECT book_id, location_type, location_json, percent, updated_at
         FROM reading_progress WHERE book_id = ?",
    )
    .bind(book_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn save_progress(
    pool: &SqlitePool,
    book_id: i64,
    input: ProgressInput,
) -> Result<ReadingProgress, AppError> {
    ensure_book(pool, book_id).await?;
    validate_location(&input.location_type, &input.location_json)?;

    if !input.percent.is_finite() || !(0.0..=1.0).contains(&input.percent) {
        return Err(AppError::Validation(
            "阅读进度必须在 0 到 1 之间".to_string(),
        ));
    }

    let now = now_ms();
    let mut transaction = pool.begin().await?;

    sqlx::query(
        "INSERT INTO reading_progress (book_id, location_type, location_json, percent, updated_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(book_id) DO UPDATE
         SET location_type = excluded.location_type,
             location_json = excluded.location_json,
             percent = excluded.percent,
             updated_at = excluded.updated_at",
    )
    .bind(book_id)
    .bind(&input.location_type)
    .bind(&input.location_json)
    .bind(input.percent)
    .bind(now)
    .execute(&mut *transaction)
    .await?;

    sqlx::query("UPDATE books SET last_opened_at = ?, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(now)
        .bind(book_id)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    get_progress(pool, book_id)
        .await?
        .ok_or_else(|| AppError::Internal("阅读进度保存失败".to_string()))
}

pub async fn list_bookmarks(pool: &SqlitePool, book_id: i64) -> Result<Vec<Bookmark>, AppError> {
    ensure_book(pool, book_id).await?;

    Ok(sqlx::query_as::<_, Bookmark>(
        "SELECT id, book_id, location_type, location_json, label, created_at
         FROM bookmarks WHERE book_id = ? ORDER BY created_at DESC, id DESC",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create_bookmark(
    pool: &SqlitePool,
    book_id: i64,
    input: BookmarkInput,
) -> Result<Bookmark, AppError> {
    ensure_book(pool, book_id).await?;
    validate_location(&input.location_type, &input.location_json)?;

    let label = input.label.trim().to_string();
    if label.chars().count() > MAX_LABEL_CHARS {
        return Err(AppError::Validation(format!(
            "书签名称不能超过 {MAX_LABEL_CHARS} 个字符"
        )));
    }

    let now = now_ms();
    let result = sqlx::query(
        "INSERT INTO bookmarks (book_id, location_type, location_json, label, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(book_id)
    .bind(&input.location_type)
    .bind(&input.location_json)
    .bind(&label)
    .bind(now)
    .execute(pool)
    .await?;

    get_bookmark(pool, result.last_insert_rowid()).await
}

pub async fn delete_bookmark(pool: &SqlitePool, bookmark_id: i64) -> Result<(), AppError> {
    let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM bookmarks WHERE id = ?")
        .bind(bookmark_id)
        .fetch_optional(pool)
        .await?;

    if existing.is_none() {
        return Err(AppError::NotFound(format!("书签 {bookmark_id} 不存在")));
    }

    sqlx::query("DELETE FROM bookmarks WHERE id = ?")
        .bind(bookmark_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_settings(
    pool: &SqlitePool,
    scope: &str,
) -> Result<Option<ReaderSettings>, AppError> {
    validate_scope(scope)?;

    Ok(sqlx::query_as::<_, ReaderSettings>(
        "SELECT scope, preferences_json, updated_at FROM reader_settings WHERE scope = ?",
    )
    .bind(scope)
    .fetch_optional(pool)
    .await?)
}

pub async fn save_settings(
    pool: &SqlitePool,
    scope: &str,
    preferences_json: &str,
) -> Result<ReaderSettings, AppError> {
    validate_scope(scope)?;
    validate_json_object(preferences_json, "阅读设置")?;

    if preferences_json.chars().count() > MAX_SETTINGS_JSON_CHARS {
        return Err(AppError::Validation("阅读设置内容过大".to_string()));
    }

    let now = now_ms();
    sqlx::query(
        "INSERT INTO reader_settings (scope, preferences_json, updated_at)
         VALUES (?, ?, ?)
         ON CONFLICT(scope) DO UPDATE
         SET preferences_json = excluded.preferences_json,
             updated_at = excluded.updated_at",
    )
    .bind(scope)
    .bind(preferences_json)
    .bind(now)
    .execute(pool)
    .await?;

    get_settings(pool, scope)
        .await?
        .ok_or_else(|| AppError::Internal("阅读设置保存失败".to_string()))
}

pub async fn start_session(pool: &SqlitePool, book_id: i64) -> Result<i64, AppError> {
    ensure_book(pool, book_id).await?;

    let result = sqlx::query("INSERT INTO reading_sessions (book_id, started_at) VALUES (?, ?)")
        .bind(book_id)
        .bind(now_ms())
        .execute(pool)
        .await?;

    Ok(result.last_insert_rowid())
}

pub async fn end_session(
    pool: &SqlitePool,
    session_id: i64,
    end_location_json: Option<String>,
) -> Result<(), AppError> {
    let session: Option<(i64, i64, Option<i64>)> =
        sqlx::query_as("SELECT book_id, started_at, ended_at FROM reading_sessions WHERE id = ?")
            .bind(session_id)
            .fetch_optional(pool)
            .await?;

    let (book_id, started_at, ended_at) =
        session.ok_or_else(|| AppError::NotFound(format!("阅读会话 {session_id} 不存在")))?;

    if ended_at.is_some() {
        return Ok(());
    }

    if let Some(location) = end_location_json.as_deref() {
        validate_json_object(location, "阅读位置")?;
    }

    let now = now_ms();
    let duration_ms = (now - started_at).max(0);

    let mut transaction = pool.begin().await?;
    sqlx::query(
        "UPDATE reading_sessions
         SET ended_at = ?, duration_ms = ?, end_location_json = ?
         WHERE id = ?",
    )
    .bind(now)
    .bind(duration_ms)
    .bind(end_location_json.as_deref())
    .bind(session_id)
    .execute(&mut *transaction)
    .await?;

    sqlx::query("UPDATE books SET last_opened_at = ? WHERE id = ?")
        .bind(now)
        .bind(book_id)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;
    Ok(())
}

pub fn validate_external_url(value: &str) -> Result<Url, AppError> {
    let url =
        Url::parse(value.trim()).map_err(|_| AppError::Validation("外部链接无效".to_string()))?;

    match url.scheme() {
        "http" | "https" => Ok(url),
        _ => Err(AppError::Validation(
            "仅允许打开 http/https 链接".to_string(),
        )),
    }
}

async fn get_bookmark(pool: &SqlitePool, bookmark_id: i64) -> Result<Bookmark, AppError> {
    sqlx::query_as::<_, Bookmark>(
        "SELECT id, book_id, location_type, location_json, label, created_at
         FROM bookmarks WHERE id = ?",
    )
    .bind(bookmark_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("书签 {bookmark_id} 不存在")))
}

async fn ensure_book(pool: &SqlitePool, book_id: i64) -> Result<(), AppError> {
    let exists: i64 = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM books WHERE id = ?)")
        .bind(book_id)
        .fetch_one(pool)
        .await?;

    if exists == 0 {
        return Err(AppError::NotFound(format!("书籍 {book_id} 不存在")));
    }

    Ok(())
}

fn validate_scope(scope: &str) -> Result<(), AppError> {
    if !READER_SCOPES.contains(&scope) {
        return Err(AppError::Validation("阅读设置范围无效".to_string()));
    }
    Ok(())
}

fn validate_location(location_type: &str, location_json: &str) -> Result<(), AppError> {
    if location_type != "epub_cfi" {
        return Err(AppError::Validation(
            "当前仅支持 EPUB CFI 阅读位置".to_string(),
        ));
    }

    validate_json_object(location_json, "阅读位置")
}

fn validate_json_object(value: &str, label: &str) -> Result<(), AppError> {
    if value.chars().count() > MAX_LOCATION_JSON_CHARS {
        return Err(AppError::Validation(format!("{label}内容过大")));
    }

    let parsed: serde_json::Value = serde_json::from_str(value)
        .map_err(|_| AppError::Validation(format!("{label}不是有效的 JSON")))?;

    if !parsed.is_object() {
        return Err(AppError::Validation(format!("{label}必须是 JSON 对象")));
    }

    Ok(())
}

fn map_io_error(error: std::io::Error) -> AppError {
    match error.kind() {
        std::io::ErrorKind::NotFound => {
            AppError::Validation("书籍文件不存在或不可访问，请重新定位".to_string())
        }
        std::io::ErrorKind::PermissionDenied => {
            AppError::Validation("没有读取该书籍文件的权限".to_string())
        }
        _ => AppError::Io(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect, migrate};
    use crate::models::book::BookInsert;
    use crate::repositories::book_repository;

    use std::path::Path;
    use std::time::Duration;
    use tempfile::{TempDir, tempdir};

    async fn setup() -> (TempDir, SqlitePool) {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("reader-test.sqlite");
        let backups_dir = dir.path().join("backups");
        let pool = connect(&database_file).await.expect("connect");
        migrate::run(&pool, &backups_dir).await.expect("migrate");
        (dir, pool)
    }

    async fn insert_book(pool: &SqlitePool, root: &Path, format: &str) -> i64 {
        let path = root.join(format!("book.{format}"));
        std::fs::write(&path, b"book-bytes").expect("book file");
        let path_string = path.to_string_lossy().to_lowercase();
        let mut connection = pool.acquire().await.expect("conn");

        book_repository::insert(
            &mut connection,
            &BookInsert {
                title: "测试书籍".to_string(),
                author: String::new(),
                format: format.to_string(),
                file_path: path.to_string_lossy().into_owned(),
                source_path: None,
                normalized_path: path_string,
                file_size: 10,
                file_mtime: None,
                file_hash: format!("hash-{format}"),
                cover_path: None,
                description: String::new(),
                imported_at: 1,
            },
        )
        .await
        .expect("insert book")
    }

    #[tokio::test]
    async fn reads_epub_bytes_and_rejects_other_formats() {
        let (dir, pool) = setup().await;
        let book_id = insert_book(&pool, dir.path(), "epub").await;

        let bytes = read_epub_bytes(&pool, book_id).await.expect("read");
        assert_eq!(bytes, b"book-bytes");

        let pdf_id = insert_book(&pool, dir.path(), "pdf").await;
        assert!(matches!(
            read_epub_bytes(&pool, pdf_id).await,
            Err(AppError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn progress_roundtrip_updates_last_opened() {
        let (dir, pool) = setup().await;
        let book_id = insert_book(&pool, dir.path(), "epub").await;

        let saved = save_progress(
            &pool,
            book_id,
            ProgressInput {
                location_type: "epub_cfi".to_string(),
                location_json: r#"{"cfi":"epubcfi(/6/2)"}"#.to_string(),
                percent: 0.25,
            },
        )
        .await
        .expect("save progress");

        assert_eq!(saved.percent, 0.25);
        let loaded = get_progress(&pool, book_id)
            .await
            .expect("get")
            .expect("progress");
        assert_eq!(loaded.location_json, r#"{"cfi":"epubcfi(/6/2)"}"#);

        let last_opened: Option<i64> =
            sqlx::query_scalar("SELECT last_opened_at FROM books WHERE id = ?")
                .bind(book_id)
                .fetch_one(&pool)
                .await
                .expect("last opened");
        assert!(last_opened.is_some());

        assert!(matches!(
            save_progress(
                &pool,
                book_id,
                ProgressInput {
                    location_type: "epub_cfi".to_string(),
                    location_json: "{}".to_string(),
                    percent: 1.5,
                },
            )
            .await,
            Err(AppError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn bookmarks_crud_and_validation() {
        let (dir, pool) = setup().await;
        let book_id = insert_book(&pool, dir.path(), "epub").await;

        let bookmark = create_bookmark(
            &pool,
            book_id,
            BookmarkInput {
                location_type: "epub_cfi".to_string(),
                location_json: r#"{"cfi":"epubcfi(/6/4)"}"#.to_string(),
                label: "第一章".to_string(),
            },
        )
        .await
        .expect("bookmark");

        let bookmarks = list_bookmarks(&pool, book_id).await.expect("list");
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].label, "第一章");

        delete_bookmark(&pool, bookmark.id).await.expect("delete");
        assert!(
            list_bookmarks(&pool, book_id)
                .await
                .expect("list")
                .is_empty()
        );

        assert!(matches!(
            create_bookmark(
                &pool,
                book_id,
                BookmarkInput {
                    location_type: "pdf_page".to_string(),
                    location_json: "{}".to_string(),
                    label: String::new(),
                },
            )
            .await,
            Err(AppError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn reader_settings_roundtrip() {
        let (_dir, pool) = setup().await;

        let saved = save_settings(&pool, "epub", r#"{"fontSize":18}"#)
            .await
            .expect("save");
        assert_eq!(saved.scope, "epub");

        let loaded = get_settings(&pool, "epub")
            .await
            .expect("get")
            .expect("settings");
        assert_eq!(loaded.preferences_json, r#"{"fontSize":18}"#);

        assert!(matches!(
            save_settings(&pool, "unknown", "{}").await,
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            save_settings(&pool, "epub", "not-json").await,
            Err(AppError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn reading_session_records_duration() {
        let (dir, pool) = setup().await;
        let book_id = insert_book(&pool, dir.path(), "epub").await;

        let session_id = start_session(&pool, book_id).await.expect("start");
        tokio::time::sleep(Duration::from_millis(5)).await;
        end_session(
            &pool,
            session_id,
            Some(r#"{"cfi":"epubcfi(/6/8)"}"#.to_string()),
        )
        .await
        .expect("end");

        let (ended_at, duration_ms): (Option<i64>, Option<i64>) =
            sqlx::query_as("SELECT ended_at, duration_ms FROM reading_sessions WHERE id = ?")
                .bind(session_id)
                .fetch_one(&pool)
                .await
                .expect("session");
        assert!(ended_at.is_some());
        assert!(duration_ms.unwrap_or_default() >= 0);

        end_session(&pool, session_id, None)
            .await
            .expect("idempotent end");
    }

    #[test]
    fn external_urls_are_limited_to_http_and_https() {
        assert!(validate_external_url("https://example.com").is_ok());
        assert!(validate_external_url("http://example.com").is_ok());
        assert!(validate_external_url("javascript:alert(1)").is_err());
        assert!(validate_external_url("file:///C:/secret").is_err());
        assert!(validate_external_url("data:text/html,hello").is_err());
    }
}
