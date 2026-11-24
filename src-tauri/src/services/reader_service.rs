use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use encoding_rs::Encoding;
use sqlx::SqlitePool;
use url::Url;

use crate::error::AppError;
use crate::models::reader::{
    Bookmark, BookmarkInput, ProgressInput, ReaderSettings, ReadingProgress, TextBookInfo,
    TextChunk,
};
use crate::utils::now_ms;

const MAX_EPUB_BYTES: u64 = 200 * 1024 * 1024;
const MAX_PDF_BYTES: u64 = 300 * 1024 * 1024;
const TEXT_SAMPLE_BYTES: usize = 64 * 1024;
const TEXT_CHUNK_EXTRA_BYTES: usize = 8 * 1024;
const MAX_TEXT_CHUNK_BYTES: i64 = 1024 * 1024;
const MAX_LOCATION_JSON_CHARS: usize = 10_000;
const MAX_SETTINGS_JSON_CHARS: usize = 20_000;
const MAX_LABEL_CHARS: usize = 200;
const READER_SCOPES: [&str; 5] = ["global", "epub", "pdf", "txt", "markdown"];

pub async fn read_book_bytes(pool: &SqlitePool, book_id: i64) -> Result<Vec<u8>, AppError> {
    let mut connection = pool.acquire().await?;
    let row = crate::repositories::book_repository::get_row(&mut connection, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("书籍 {book_id} 不存在")))?;
    drop(connection);

    let max_bytes = match row.format.as_str() {
        "epub" => MAX_EPUB_BYTES,
        "pdf" => MAX_PDF_BYTES,
        _ => {
            return Err(AppError::Validation("该格式阅读器尚未接入".to_string()));
        }
    };

    let path = PathBuf::from(&row.file_path);
    let metadata = std::fs::metadata(&path).map_err(map_io_error)?;
    if !metadata.is_file() {
        return Err(AppError::Validation(
            "书籍文件不存在或不可访问，请重新定位".to_string(),
        ));
    }
    if metadata.len() > max_bytes {
        return Err(AppError::Validation(format!(
            "文件超过 {}MB 上限",
            max_bytes / (1024 * 1024)
        )));
    }

    tokio::task::spawn_blocking(move || std::fs::read(&path))
        .await
        .map_err(|error| AppError::Internal(format!("读取书籍文件失败：{error}")))?
        .map_err(map_io_error)
}

pub async fn get_text_book_info(pool: &SqlitePool, book_id: i64) -> Result<TextBookInfo, AppError> {
    let mut connection = pool.acquire().await?;
    let row = crate::repositories::book_repository::get_row(&mut connection, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("书籍 {book_id} 不存在")))?;
    drop(connection);

    if row.format != "txt" {
        return Err(AppError::Validation("该书籍不是 TXT".to_string()));
    }

    let path = PathBuf::from(&row.file_path);
    let metadata = std::fs::metadata(&path).map_err(map_io_error)?;
    if !metadata.is_file() {
        return Err(AppError::Validation(
            "书籍文件不存在或不可访问，请重新定位".to_string(),
        ));
    }

    let sample = read_text_sample(&path)?;
    let (encoding, has_bom, warning) = detect_text_encoding(&sample)?;

    Ok(TextBookInfo {
        book_id,
        total_bytes: metadata.len() as i64,
        encoding,
        has_bom,
        warning,
    })
}

pub async fn read_text_chunk(
    pool: &SqlitePool,
    book_id: i64,
    offset: i64,
    max_bytes: i64,
    encoding_label: &str,
) -> Result<TextChunk, AppError> {
    let mut connection = pool.acquire().await?;
    let row = crate::repositories::book_repository::get_row(&mut connection, book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("书籍 {book_id} 不存在")))?;
    drop(connection);

    if row.format != "txt" {
        return Err(AppError::Validation("该书籍不是 TXT".to_string()));
    }
    if offset < 0 {
        return Err(AppError::Validation("文本偏移无效".to_string()));
    }

    let max_bytes = max_bytes.clamp(4096, MAX_TEXT_CHUNK_BYTES) as usize;
    let encoding = Encoding::for_label(encoding_label.as_bytes())
        .ok_or_else(|| AppError::Validation("不支持的文本编码".to_string()))?;
    if encoding.name().starts_with("UTF-16") {
        return Err(AppError::Validation(
            "暂不支持 UTF-16 编码的 TXT 文件".to_string(),
        ));
    }

    let path = PathBuf::from(&row.file_path);
    let metadata = std::fs::metadata(&path).map_err(map_io_error)?;
    if !metadata.is_file() {
        return Err(AppError::Validation(
            "书籍文件不存在或不可访问，请重新定位".to_string(),
        ));
    }
    let total_bytes = metadata.len() as i64;
    if offset > total_bytes {
        return Err(AppError::Validation("文本偏移超出文件范围".to_string()));
    }

    tokio::task::spawn_blocking(move || {
        read_text_chunk_blocking(&path, offset, max_bytes, encoding, total_bytes)
    })
    .await
    .map_err(|error| AppError::Internal(format!("读取文本失败：{error}")))?
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
    if !matches!(
        location_type,
        "epub_cfi" | "pdf_page" | "txt_offset" | "md_scroll"
    ) {
        return Err(AppError::Validation("不支持的阅读位置类型".to_string()));
    }

    validate_json_object(location_json, "阅读位置")
}

fn read_text_sample(path: &std::path::Path) -> Result<Vec<u8>, AppError> {
    let mut file = std::fs::File::open(path).map_err(map_io_error)?;
    let mut buffer = vec![0u8; TEXT_SAMPLE_BYTES];
    let read = file.read(&mut buffer).map_err(map_io_error)?;
    buffer.truncate(read);
    Ok(buffer)
}

fn detect_text_encoding(sample: &[u8]) -> Result<(String, bool, Option<String>), AppError> {
    if sample.is_empty() {
        return Ok(("utf-8".to_string(), false, None));
    }

    if sample.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok(("utf-8".to_string(), true, None));
    }

    if sample.starts_with(&[0xFF, 0xFE]) || sample.starts_with(&[0xFE, 0xFF]) {
        return Err(AppError::Validation(
            "暂不支持 UTF-16 编码的 TXT 文件".to_string(),
        ));
    }

    match std::str::from_utf8(sample) {
        Ok(_) => return Ok(("utf-8".to_string(), false, None)),
        // A sample can end in the middle of a multi-byte character.
        Err(error) if error.error_len().is_none() => {
            return Ok(("utf-8".to_string(), false, None));
        }
        Err(_) => {}
    }

    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    detector.feed(sample, true);
    let detected = detector.guess(None, chardetng::Utf8Detection::Allow);

    if detected.name().starts_with("UTF-16") {
        return Err(AppError::Validation(
            "暂不支持 UTF-16 编码的 TXT 文件".to_string(),
        ));
    }

    let encoding = if detected.name() == "GBK" {
        encoding_rs::GB18030
    } else {
        detected
    };

    let (_, _, had_errors) = encoding.decode(sample);
    let warning = had_errors.then(|| {
        "部分内容无法按检测到的编码解码，可能显示为替换字符；如内容乱码，请确认文件实际编码。"
            .to_string()
    });

    Ok((encoding.name().to_lowercase(), false, warning))
}

fn read_text_chunk_blocking(
    path: &std::path::Path,
    offset: i64,
    max_bytes: usize,
    encoding: &'static Encoding,
    total_bytes: i64,
) -> Result<TextChunk, AppError> {
    let mut file = std::fs::File::open(path).map_err(map_io_error)?;
    file.seek(SeekFrom::Start(offset as u64))
        .map_err(map_io_error)?;

    let mut buffer = Vec::with_capacity(max_bytes + TEXT_CHUNK_EXTRA_BYTES);
    let mut limited = (&mut file).take((max_bytes + TEXT_CHUNK_EXTRA_BYTES) as u64);
    limited.read_to_end(&mut buffer).map_err(map_io_error)?;

    if buffer.is_empty() {
        return Ok(TextChunk {
            text: String::new(),
            start_offset: offset,
            next_offset: offset,
            total_bytes,
            eof: true,
            had_errors: false,
        });
    }

    // Progress and bookmarks may store a byte offset estimated from the
    // scroll position. Move forward to the next line boundary so decoding
    // never starts in the middle of a multi-byte character or a line.
    let mut skip = 0usize;
    if offset > 0 {
        let mut previous = [0u8; 1];
        file.seek(SeekFrom::Start(offset as u64 - 1))
            .map_err(map_io_error)?;
        let previous_byte = file.read_exact(&mut previous).ok().map(|()| previous[0]);

        let newline_index = if previous_byte != Some(b'\n') {
            buffer.iter().position(|byte| *byte == b'\n')
        } else {
            None
        };

        if let Some(index) = newline_index {
            skip = (index + 1).min(buffer.len() - 1);
        }
    }

    let start_offset = offset + skip as i64;
    let buffer = &buffer[skip..];
    let read = buffer.len();
    let bom_len = if start_offset == 0 && buffer.starts_with(&[0xEF, 0xBB, 0xBF]) {
        3
    } else {
        0
    };
    let at_eof = start_offset + read as i64 >= total_bytes;
    let search_limit = read.min(max_bytes);

    let cut = if at_eof {
        read
    } else {
        buffer[..search_limit]
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map(|index| index + 1)
            .or_else(|| {
                buffer
                    .iter()
                    .rposition(|byte| *byte == b'\n')
                    .map(|index| index + 1)
            })
            .unwrap_or(search_limit)
    };
    let cut = cut.max(bom_len);

    if cut <= bom_len && !at_eof {
        return Err(AppError::Validation(
            "无法在文本中找到可用的行边界".to_string(),
        ));
    }

    let (decoded, _, had_errors) = encoding.decode(&buffer[bom_len..cut]);
    let next_offset = start_offset + cut as i64;

    Ok(TextChunk {
        text: decoded.into_owned(),
        start_offset: start_offset + bom_len as i64,
        next_offset,
        total_bytes,
        eof: next_offset >= total_bytes,
        had_errors,
    })
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
        insert_book_with_bytes(pool, root, format, b"book-bytes", format).await
    }

    async fn insert_book_with_bytes(
        pool: &SqlitePool,
        root: &Path,
        format: &str,
        bytes: &[u8],
        slug: &str,
    ) -> i64 {
        let path = root.join(format!("book-{slug}.{format}"));
        std::fs::write(&path, bytes).expect("book file");
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
                file_size: bytes.len() as i64,
                file_mtime: None,
                file_hash: format!("hash-{slug}"),
                cover_path: None,
                description: String::new(),
                imported_at: 1,
            },
        )
        .await
        .expect("insert book")
    }

    #[tokio::test]
    async fn reads_epub_and_pdf_bytes_and_rejects_other_formats() {
        let (dir, pool) = setup().await;
        let epub_id = insert_book(&pool, dir.path(), "epub").await;
        let pdf_id = insert_book(&pool, dir.path(), "pdf").await;
        let txt_id = insert_book(&pool, dir.path(), "txt").await;

        assert_eq!(
            read_book_bytes(&pool, epub_id).await.expect("epub"),
            b"book-bytes"
        );
        assert_eq!(
            read_book_bytes(&pool, pdf_id).await.expect("pdf"),
            b"book-bytes"
        );
        assert!(matches!(
            read_book_bytes(&pool, txt_id).await,
            Err(AppError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn pdf_page_locations_are_supported() {
        let (dir, pool) = setup().await;
        let book_id = insert_book(&pool, dir.path(), "pdf").await;

        let saved = save_progress(
            &pool,
            book_id,
            ProgressInput {
                location_type: "pdf_page".to_string(),
                location_json: r#"{"page":2,"mode":"fit-width","scale":1.2}"#.to_string(),
                percent: 0.5,
            },
        )
        .await
        .expect("pdf progress");
        assert_eq!(saved.percent, 0.5);

        let bookmark = create_bookmark(
            &pool,
            book_id,
            BookmarkInput {
                location_type: "pdf_page".to_string(),
                location_json: r#"{"page":2,"mode":"fit-width","scale":1.2}"#.to_string(),
                label: "第 2 页".to_string(),
            },
        )
        .await
        .expect("pdf bookmark");
        assert_eq!(bookmark.label, "第 2 页");
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
                    location_type: "unknown_location".to_string(),
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

    #[tokio::test]
    async fn text_encoding_detection_utf8_bom_and_gb18030() {
        let (dir, pool) = setup().await;
        let content = "第一行中文\n第二行中文\n第三行中文\n";

        let utf8_id =
            insert_book_with_bytes(&pool, dir.path(), "txt", content.as_bytes(), "utf8").await;
        let utf8_info = get_text_book_info(&pool, utf8_id).await.expect("utf8 info");
        assert_eq!(utf8_info.encoding, "utf-8");
        assert!(!utf8_info.has_bom);
        assert!(utf8_info.warning.is_none());

        let utf8_chunk = read_text_chunk(&pool, utf8_id, 0, 4096, &utf8_info.encoding)
            .await
            .expect("utf8 chunk");
        assert_eq!(utf8_chunk.text, content);
        assert!(utf8_chunk.eof);
        assert_eq!(utf8_chunk.start_offset, 0);

        let mut bom_bytes = vec![0xEF, 0xBB, 0xBF];
        bom_bytes.extend_from_slice(content.as_bytes());
        let bom_id = insert_book_with_bytes(&pool, dir.path(), "txt", &bom_bytes, "bom").await;
        let bom_info = get_text_book_info(&pool, bom_id).await.expect("bom info");
        assert_eq!(bom_info.encoding, "utf-8");
        assert!(bom_info.has_bom);
        let bom_chunk = read_text_chunk(&pool, bom_id, 0, 4096, &bom_info.encoding)
            .await
            .expect("bom chunk");
        assert_eq!(bom_chunk.text, content);
        assert_eq!(bom_chunk.start_offset, 3);

        let (encoded, _, _) = encoding_rs::GB18030.encode(content);
        let gb_id =
            insert_book_with_bytes(&pool, dir.path(), "txt", encoded.as_ref(), "gb18030").await;
        let gb_info = get_text_book_info(&pool, gb_id).await.expect("gb info");
        assert!(gb_info.encoding.contains("gb"));
        let gb_chunk = read_text_chunk(&pool, gb_id, 0, 4096, &gb_info.encoding)
            .await
            .expect("gb chunk");
        assert_eq!(gb_chunk.text, content);
    }

    #[tokio::test]
    async fn text_chunks_are_line_aligned_and_lossless() {
        let (dir, pool) = setup().await;
        let content: String = (1..=2000)
            .map(|line| format!("第 {line} 行中文内容，用于验证分块边界。\n"))
            .collect();
        let book_id =
            insert_book_with_bytes(&pool, dir.path(), "txt", content.as_bytes(), "chunks").await;

        let info = get_text_book_info(&pool, book_id).await.expect("info");
        let mut offset = 0;
        let mut combined = String::new();
        let mut iterations = 0;

        loop {
            let chunk = read_text_chunk(&pool, book_id, offset, 4096, &info.encoding)
                .await
                .expect("chunk");
            if !chunk.eof {
                assert!(
                    chunk.text.ends_with('\n'),
                    "non-final chunks must end on a line boundary"
                );
                assert!(chunk.next_offset > offset);
            }
            combined.push_str(&chunk.text);
            offset = chunk.next_offset;
            iterations += 1;

            if chunk.eof {
                break;
            }
            assert!(iterations < 200, "chunk loop did not terminate");
        }

        assert_eq!(combined, content);
    }

    #[tokio::test]
    async fn text_chunk_aligns_arbitrary_offsets_to_line_boundaries() {
        let (dir, pool) = setup().await;
        let first_line = "第一行内容\n";
        let second_line = "第二行中文内容\n";
        let content = format!("{first_line}{second_line}第三行内容\n");
        let book_id =
            insert_book_with_bytes(&pool, dir.path(), "txt", content.as_bytes(), "align").await;

        let info = get_text_book_info(&pool, book_id).await.expect("info");
        let middle_offset = (first_line.len() / 2) as i64;
        let aligned = read_text_chunk(&pool, book_id, middle_offset, 4096, &info.encoding)
            .await
            .expect("aligned chunk");

        assert_eq!(aligned.start_offset, first_line.len() as i64);
        assert_eq!(aligned.text, format!("{second_line}第三行内容\n"));
        assert!(!aligned.text.contains('\u{FFFD}'));
        assert!(aligned.eof);

        let line_start = read_text_chunk(
            &pool,
            book_id,
            first_line.len() as i64,
            4096,
            &info.encoding,
        )
        .await
        .expect("line start chunk");
        assert_eq!(line_start.start_offset, first_line.len() as i64);
        assert_eq!(line_start.text, format!("{second_line}第三行内容\n"));
    }

    #[tokio::test]
    async fn text_info_rejects_utf16_and_non_text_books() {
        let (dir, pool) = setup().await;
        let utf16_bytes = [0xFF, 0xFE, 0x41, 0x00, 0x42, 0x00];
        let utf16_id =
            insert_book_with_bytes(&pool, dir.path(), "txt", &utf16_bytes, "utf16").await;
        assert!(matches!(
            get_text_book_info(&pool, utf16_id).await,
            Err(AppError::Validation(_))
        ));

        let epub_id = insert_book(&pool, dir.path(), "epub").await;
        assert!(matches!(
            get_text_book_info(&pool, epub_id).await,
            Err(AppError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn txt_offset_progress_and_bookmarks_are_supported() {
        let (dir, pool) = setup().await;
        let book_id =
            insert_book_with_bytes(&pool, dir.path(), "txt", b"line one\nline two\n", "offset")
                .await;

        let saved = save_progress(
            &pool,
            book_id,
            ProgressInput {
                location_type: "txt_offset".to_string(),
                location_json: r#"{"offset":9}"#.to_string(),
                percent: 0.5,
            },
        )
        .await
        .expect("txt progress");
        assert_eq!(saved.percent, 0.5);

        let bookmark = create_bookmark(
            &pool,
            book_id,
            BookmarkInput {
                location_type: "txt_offset".to_string(),
                location_json: r#"{"offset":9}"#.to_string(),
                label: "第二行".to_string(),
            },
        )
        .await
        .expect("txt bookmark");
        assert_eq!(bookmark.label, "第二行");
    }
}
