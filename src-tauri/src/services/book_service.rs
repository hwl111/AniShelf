use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use quick_xml::Reader;
use quick_xml::events::Event;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use zip::ZipArchive;

use crate::db::AppPaths;
use crate::error::AppError;
use crate::models::book::{
    BOOK_FORMATS, BOOK_SORT_OPTIONS, Book, BookFileStatus, BookFilter, BookImportResult,
    BookInsert, BookRelocation, BookRow, BookUpdateInput,
};
use crate::models::tag::Tag;
use crate::repositories::book_repository as repository;
use crate::utils::now_ms;

const MAX_FILE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_COVER_BYTES: u64 = 10 * 1024 * 1024;
const MAX_GENERATED_COVER_BYTES: usize = 2 * 1024 * 1024;
const MAX_GENERATED_COVER_DATA_URL_CHARS: usize = 4 * 1024 * 1024;
const GENERATED_COVER_PREFIX: &str = "data:image/png;base64,";
const MAX_XML_BYTES: u64 = 5 * 1024 * 1024;
const TEXT_PREVIEW_BYTES: usize = 64 * 1024;

const MAX_TITLE_CHARS: usize = 300;
const MAX_AUTHOR_CHARS: usize = 200;
const MAX_DESCRIPTION_CHARS: usize = 20_000;
const MAX_CATEGORY_CHARS: usize = 100;
const MAX_TAGS: usize = 20;
const MAX_TAG_CHARS: usize = 50;

#[derive(Debug)]
struct ExtractedMetadata {
    title: String,
    author: String,
    description: String,
    cover: Option<CoverData>,
}

#[derive(Debug)]
struct CoverData {
    bytes: Vec<u8>,
    extension: String,
}

#[derive(Debug)]
struct AnalyzedFile {
    format: String,
    title: String,
    author: String,
    description: String,
    canonical_path: String,
    normalized_path: String,
    file_size: i64,
    file_mtime: Option<i64>,
    file_hash: String,
    cover: Option<CoverData>,
}

#[derive(Debug)]
struct MaterializedFile {
    created_files: Vec<PathBuf>,
}

pub async fn import_book(
    pool: &SqlitePool,
    paths: &AppPaths,
    file_path: String,
    copy_to_library: bool,
) -> Result<BookImportResult, AppError> {
    let input = file_path.trim().to_string();
    if input.is_empty() {
        return Err(AppError::Validation("请选择要导入的文件".to_string()));
    }

    let canonical = tokio::task::spawn_blocking(move || canonicalize_input(&input))
        .await
        .map_err(|error| AppError::Internal(format!("读取文件失败：{error}")))??;
    let normalized = normalize_path(&display_path(&canonical));

    if let Some(existing_id) = repository::find_by_normalized_path(pool, &normalized).await? {
        return duplicate_result(pool, paths, existing_id).await;
    }

    let analyzed = tokio::task::spawn_blocking(move || analyze_file(&canonical))
        .await
        .map_err(|error| AppError::Internal(format!("解析文件失败：{error}")))??;

    if let Some(existing_id) = repository::find_by_hash(pool, &analyzed.file_hash).await? {
        return duplicate_result(pool, paths, existing_id).await;
    }

    let covers_dir = paths.covers_dir.clone();
    let attachments_dir = paths.attachments_dir.clone();
    let (insert, materialized) = tokio::task::spawn_blocking(move || {
        materialize(analyzed, &covers_dir, &attachments_dir, copy_to_library)
    })
    .await
    .map_err(|error| AppError::Internal(format!("保存文件失败：{error}")))??;

    let mut transaction = pool.begin().await?;
    let id = match repository::insert(&mut transaction, &insert).await {
        Ok(id) => id,
        Err(error) => {
            cleanup_files(&materialized.created_files);
            return Err(AppError::Database(error));
        }
    };
    transaction.commit().await?;

    Ok(BookImportResult {
        book: get(pool, paths, id).await?,
        duplicate: false,
    })
}

pub async fn list(
    pool: &SqlitePool,
    paths: &AppPaths,
    filter: &BookFilter,
) -> Result<Vec<Book>, AppError> {
    validate_filter(filter)?;

    let rows = repository::list(pool, filter).await?;
    let all_tags = repository::all_book_tags(pool).await?;
    let favorite_ids: HashSet<i64> = repository::favorite_ids(pool).await?.into_iter().collect();

    let mut tags_by_book: HashMap<i64, Vec<Tag>> = HashMap::new();
    for (book_id, tag) in all_tags {
        tags_by_book.entry(book_id).or_default().push(tag);
    }

    Ok(rows
        .into_iter()
        .map(|row| {
            let tags = tags_by_book.remove(&row.id).unwrap_or_default();
            let is_favorite = favorite_ids.contains(&row.id);
            build_book(row, tags, is_favorite, paths)
        })
        .collect())
}

pub async fn get(pool: &SqlitePool, paths: &AppPaths, id: i64) -> Result<Book, AppError> {
    let mut connection = pool.acquire().await?;
    let row = repository::get_row(&mut connection, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("书籍 {id} 不存在")))?;
    drop(connection);

    let tags = repository::tags_for(pool, id).await?;
    let is_favorite = repository::is_favorite(pool, id).await?;

    Ok(build_book(row, tags, is_favorite, paths))
}

pub async fn save_generated_cover(
    pool: &SqlitePool,
    paths: &AppPaths,
    id: i64,
    data_url: &str,
) -> Result<Book, AppError> {
    let mut connection = pool.acquire().await?;
    let row = repository::get_row(&mut connection, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("书籍 {id} 不存在")))?;
    drop(connection);

    if !matches!(row.format.as_str(), "pdf" | "txt") {
        return Err(AppError::Validation(
            "仅 PDF 或 TXT 书籍支持生成封面".to_string(),
        ));
    }
    if row.cover_path.is_some() {
        return get(pool, paths, id).await;
    }

    let bytes = decode_generated_cover(data_url)?;
    std::fs::create_dir_all(&paths.covers_dir)?;

    let file_name = format!("generated-cover-{id}-{}.png", now_ms());
    let full_path = paths.covers_dir.join(&file_name);
    let relative_path = format!("covers/{file_name}");
    std::fs::write(&full_path, &bytes)?;

    let result = sqlx::query("UPDATE books SET cover_path = ? WHERE id = ? AND cover_path IS NULL")
        .bind(&relative_path)
        .bind(id)
        .execute(pool)
        .await;

    match result {
        Ok(done) if done.rows_affected() == 1 => {
            log::info!("已生成书籍封面：{}", full_path.display());
            get(pool, paths, id).await
        }
        Ok(_) => {
            let _ = std::fs::remove_file(&full_path);
            get(pool, paths, id).await
        }
        Err(error) => {
            let _ = std::fs::remove_file(&full_path);
            Err(AppError::Database(error))
        }
    }
}

fn decode_generated_cover(data_url: &str) -> Result<Vec<u8>, AppError> {
    if data_url.len() > MAX_GENERATED_COVER_DATA_URL_CHARS {
        return Err(AppError::Validation("生成的封面数据过大".to_string()));
    }
    let payload = data_url
        .strip_prefix(GENERATED_COVER_PREFIX)
        .ok_or_else(|| AppError::Validation("封面数据必须是 PNG Data URL".to_string()))?;
    let bytes = BASE64
        .decode(payload)
        .map_err(|_| AppError::Validation("封面数据不是有效的 Base64".to_string()))?;

    if bytes.len() > MAX_GENERATED_COVER_BYTES {
        return Err(AppError::Validation("生成的封面图片超过 2MB".to_string()));
    }
    if bytes.len() < 24 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(AppError::Validation(
            "封面数据不是有效的 PNG 图片".to_string(),
        ));
    }
    if &bytes[12..16] != b"IHDR" {
        return Err(AppError::Validation("封面 PNG 缺少 IHDR 信息".to_string()));
    }

    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if !(200..=2000).contains(&width) || !(200..=3000).contains(&height) {
        return Err(AppError::Validation(
            "封面图片尺寸必须在 200–2000 × 200–3000 之间".to_string(),
        ));
    }

    Ok(bytes)
}

pub async fn update(
    pool: &SqlitePool,
    paths: &AppPaths,
    id: i64,
    input: BookUpdateInput,
) -> Result<Book, AppError> {
    let input = normalize_update_input(input)?;
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let existing = repository::get_row(&mut transaction, id).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!("书籍 {id} 不存在")));
    }

    repository::update(&mut transaction, id, &input, now).await?;
    repository::set_tags(&mut transaction, id, &input.tags, now).await?;
    transaction.commit().await?;

    get(pool, paths, id).await
}

pub async fn delete(pool: &SqlitePool, paths: &AppPaths, id: i64) -> Result<(), AppError> {
    let mut transaction = pool.begin().await?;
    let row = repository::get_row(&mut transaction, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("书籍 {id} 不存在")))?;

    repository::cleanup_polymorphic(&mut transaction, id).await?;
    repository::delete(&mut transaction, id).await?;
    transaction.commit().await?;

    remove_app_files(paths, &row);
    Ok(())
}

pub async fn set_favorite(
    pool: &SqlitePool,
    paths: &AppPaths,
    id: i64,
    favorite: bool,
) -> Result<Book, AppError> {
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let existing = repository::get_row(&mut transaction, id).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!("书籍 {id} 不存在")));
    }

    repository::set_favorite(&mut transaction, id, favorite, now).await?;
    transaction.commit().await?;

    get(pool, paths, id).await
}

pub async fn check_file(pool: &SqlitePool, id: i64) -> Result<BookFileStatus, AppError> {
    let mut connection = pool.acquire().await?;
    let row = repository::get_row(&mut connection, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("书籍 {id} 不存在")))?;

    let exists = Path::new(&row.file_path).is_file();
    Ok(BookFileStatus {
        exists,
        path: row.file_path,
        message: if exists {
            "文件可用".to_string()
        } else {
            "文件不存在或不可访问，请重新定位".to_string()
        },
    })
}

pub async fn relocate(
    pool: &SqlitePool,
    paths: &AppPaths,
    id: i64,
    new_path: String,
) -> Result<Book, AppError> {
    let new_path = new_path.trim().to_string();
    if new_path.is_empty() {
        return Err(AppError::Validation("请选择新的文件位置".to_string()));
    }

    let mut transaction = pool.begin().await?;
    let row = repository::get_row(&mut transaction, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("书籍 {id} 不存在")))?;

    if row.source_path.is_some() {
        return Err(AppError::Validation(
            "该书籍是应用内副本且文件已丢失，请删除后重新导入".to_string(),
        ));
    }

    let expected_format = row.format.clone();
    let target =
        tokio::task::spawn_blocking(move || analyze_relocation(&new_path, &expected_format))
            .await
            .map_err(|error| AppError::Internal(format!("读取文件失败：{error}")))??;

    if let Some(other_id) =
        repository::find_by_normalized_path(pool, &target.normalized_path).await?
        && other_id != id
    {
        return Err(AppError::Validation("该文件已属于另一本书".to_string()));
    }
    if let Some(other_id) = repository::find_by_hash(pool, &target.file_hash).await?
        && other_id != id
    {
        return Err(AppError::Validation("该文件内容已属于另一本书".to_string()));
    }

    repository::relocate(&mut transaction, id, &target, now_ms()).await?;
    transaction.commit().await?;

    get(pool, paths, id).await
}

async fn duplicate_result(
    pool: &SqlitePool,
    paths: &AppPaths,
    existing_id: i64,
) -> Result<BookImportResult, AppError> {
    Ok(BookImportResult {
        book: get(pool, paths, existing_id).await?,
        duplicate: true,
    })
}

fn build_book(row: BookRow, tags: Vec<Tag>, is_favorite: bool, paths: &AppPaths) -> Book {
    let cover_url = row
        .cover_path
        .as_ref()
        .map(|relative| paths.data_dir.join(relative).to_string_lossy().into_owned());
    let file_exists = Path::new(&row.file_path).is_file();

    Book {
        id: row.id,
        title: row.title,
        author: row.author,
        format: row.format,
        file_path: row.file_path,
        source_path: row.source_path,
        file_size: row.file_size,
        file_mtime: row.file_mtime,
        cover_path: row.cover_path,
        cover_url,
        description: row.description,
        category: row.category,
        imported_at: row.imported_at,
        last_opened_at: row.last_opened_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
        tags,
        is_favorite,
        file_exists,
    }
}

fn normalize_update_input(mut input: BookUpdateInput) -> Result<BookUpdateInput, AppError> {
    input.title = input.title.trim().to_string();
    if input.title.is_empty() {
        return Err(AppError::Validation("书名不能为空".to_string()));
    }
    if input.title.chars().count() > MAX_TITLE_CHARS {
        return Err(AppError::Validation(format!(
            "书名不能超过 {MAX_TITLE_CHARS} 个字符"
        )));
    }

    input.author = input.author.trim().to_string();
    if input.author.chars().count() > MAX_AUTHOR_CHARS {
        return Err(AppError::Validation(format!(
            "作者不能超过 {MAX_AUTHOR_CHARS} 个字符"
        )));
    }

    input.description = input.description.trim().to_string();
    if input.description.chars().count() > MAX_DESCRIPTION_CHARS {
        return Err(AppError::Validation(format!(
            "简介不能超过 {MAX_DESCRIPTION_CHARS} 个字符"
        )));
    }

    input.category = input.category.trim().to_string();
    if input.category.chars().count() > MAX_CATEGORY_CHARS {
        return Err(AppError::Validation(format!(
            "分类不能超过 {MAX_CATEGORY_CHARS} 个字符"
        )));
    }

    input.tags = normalize_tags(input.tags)?;
    Ok(input)
}

fn normalize_tags(values: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().count() > MAX_TAG_CHARS {
            return Err(AppError::Validation(format!(
                "标签单个不能超过 {MAX_TAG_CHARS} 个字符"
            )));
        }

        let key = trimmed.to_lowercase();
        if seen.insert(key) {
            result.push(trimmed.to_string());
        }
    }

    if result.len() > MAX_TAGS {
        return Err(AppError::Validation(format!("标签最多 {MAX_TAGS} 个")));
    }

    Ok(result)
}

fn validate_filter(filter: &BookFilter) -> Result<(), AppError> {
    if let Some(format) = filter.format.as_deref()
        && !BOOK_FORMATS.contains(&format)
    {
        return Err(AppError::Validation("书籍格式筛选无效".to_string()));
    }

    if let Some(sort) = filter.sort.as_deref()
        && !BOOK_SORT_OPTIONS.contains(&sort)
    {
        return Err(AppError::Validation("排序方式无效".to_string()));
    }

    Ok(())
}

fn canonicalize_input(input: &str) -> Result<PathBuf, AppError> {
    let path = PathBuf::from(input);
    let canonical = std::fs::canonicalize(&path).map_err(map_io_error)?;
    let metadata = std::fs::metadata(&canonical).map_err(map_io_error)?;

    if !metadata.is_file() {
        return Err(AppError::Validation("请选择一个文件".to_string()));
    }
    if metadata.len() == 0 {
        return Err(AppError::Validation("文件为空".to_string()));
    }
    if metadata.len() > MAX_FILE_BYTES {
        return Err(AppError::Validation("文件超过 1GB 上限".to_string()));
    }

    let extension = extension_lower(&canonical);
    if format_for_extension(&extension).is_none() {
        return Err(AppError::Validation(
            "仅支持 EPUB、PDF、TXT、Markdown 文件".to_string(),
        ));
    }

    Ok(canonical)
}

fn analyze_file(path: &Path) -> Result<AnalyzedFile, AppError> {
    let metadata = std::fs::metadata(path).map_err(map_io_error)?;
    let extension = extension_lower(path);
    let format = format_for_extension(&extension)
        .ok_or_else(|| AppError::Validation("不支持的书籍格式".to_string()))?
        .to_string();

    let file_hash = sha256_file(path)?;
    let extracted = extract_metadata(&format, path)?;

    let fallback_title = file_stem(path);
    let title = if extracted.title.trim().is_empty() {
        fallback_title
    } else {
        truncate_chars(extracted.title.trim(), MAX_TITLE_CHARS)
    };
    let author = truncate_chars(extracted.author.trim(), MAX_AUTHOR_CHARS);
    let description = truncate_chars(extracted.description.trim(), MAX_DESCRIPTION_CHARS);

    let canonical_path = display_path(path);
    let normalized_path = normalize_path(&canonical_path);

    Ok(AnalyzedFile {
        format,
        title,
        author,
        description,
        canonical_path,
        normalized_path,
        file_size: metadata.len() as i64,
        file_mtime: file_mtime(&metadata),
        file_hash,
        cover: extracted.cover,
    })
}

fn materialize(
    analyzed: AnalyzedFile,
    covers_dir: &Path,
    attachments_dir: &Path,
    copy_to_library: bool,
) -> Result<(BookInsert, MaterializedFile), AppError> {
    let mut created_files = Vec::new();

    let result: Result<(Option<String>, String, Option<String>), AppError> = (|| {
        let cover_path = match &analyzed.cover {
            Some(cover) => {
                std::fs::create_dir_all(covers_dir)?;
                let prefix = &analyzed.file_hash[..16.min(analyzed.file_hash.len())];
                let file_name = format!("cover-{prefix}.{}", cover.extension);
                let full_path = covers_dir.join(&file_name);
                std::fs::write(&full_path, &cover.bytes)?;
                created_files.push(full_path);
                Some(format!("covers/{file_name}"))
            }
            None => None,
        };

        let (file_path, source_path) = if copy_to_library {
            let books_dir = attachments_dir.join("books");
            std::fs::create_dir_all(&books_dir)?;
            let extension = extension_lower(Path::new(&analyzed.canonical_path));
            let file_name = format!("{}.{}", analyzed.file_hash, extension);
            let destination = books_dir.join(file_name);
            if !destination.exists() {
                std::fs::copy(&analyzed.canonical_path, &destination)?;
                created_files.push(destination.clone());
            }
            (
                display_path(&destination),
                Some(analyzed.canonical_path.clone()),
            )
        } else {
            (analyzed.canonical_path.clone(), None)
        };

        Ok((cover_path, file_path, source_path))
    })();

    match result {
        Ok((cover_path, file_path, source_path)) => {
            let insert = BookInsert {
                title: analyzed.title,
                author: analyzed.author,
                format: analyzed.format,
                file_path: file_path.clone(),
                source_path: source_path.clone(),
                normalized_path: analyzed.normalized_path,
                file_size: analyzed.file_size,
                file_mtime: analyzed.file_mtime,
                file_hash: analyzed.file_hash,
                cover_path: cover_path.clone(),
                description: analyzed.description,
                imported_at: now_ms(),
            };

            Ok((insert, MaterializedFile { created_files }))
        }
        Err(error) => {
            cleanup_files(&created_files);
            Err(error)
        }
    }
}

fn analyze_relocation(path: &str, expected_format: &str) -> Result<BookRelocation, AppError> {
    let canonical = canonicalize_input(path)?;
    let extension = extension_lower(&canonical);
    let format = format_for_extension(&extension)
        .ok_or_else(|| AppError::Validation("不支持的书籍格式".to_string()))?;

    if format != expected_format {
        return Err(AppError::Validation(format!(
            "文件类型不匹配，需要 {expected_format} 文件"
        )));
    }

    let metadata = std::fs::metadata(&canonical).map_err(map_io_error)?;
    let file_path = display_path(&canonical);
    let normalized_path = normalize_path(&file_path);
    let file_hash = sha256_file(&canonical)?;

    Ok(BookRelocation {
        file_path,
        normalized_path,
        file_size: metadata.len() as i64,
        file_mtime: file_mtime(&metadata),
        file_hash,
    })
}

fn extract_metadata(format: &str, path: &Path) -> Result<ExtractedMetadata, AppError> {
    match format {
        "epub" => extract_epub(path),
        "pdf" => {
            validate_pdf(path)?;
            Ok(empty_metadata())
        }
        "txt" => {
            validate_text(path)?;
            Ok(empty_metadata())
        }
        "markdown" => {
            let title = markdown_title(path)?;
            Ok(ExtractedMetadata {
                title,
                ..empty_metadata()
            })
        }
        _ => Err(AppError::Validation("不支持的书籍格式".to_string())),
    }
}

fn empty_metadata() -> ExtractedMetadata {
    ExtractedMetadata {
        title: String::new(),
        author: String::new(),
        description: String::new(),
        cover: None,
    }
}

fn validate_pdf(path: &Path) -> Result<(), AppError> {
    let mut file = File::open(path).map_err(map_io_error)?;
    let mut header = [0u8; 5];
    file.read_exact(&mut header).map_err(map_io_error)?;

    if &header != b"%PDF-" {
        return Err(AppError::Validation(
            "文件不是有效的 PDF（缺少 %PDF- 头）".to_string(),
        ));
    }

    Ok(())
}

fn validate_text(path: &Path) -> Result<Vec<u8>, AppError> {
    let mut file = File::open(path).map_err(map_io_error)?;
    let mut buffer = vec![0u8; TEXT_PREVIEW_BYTES];
    let read = file.read(&mut buffer).map_err(map_io_error)?;
    buffer.truncate(read);

    if buffer.contains(&0) {
        return Err(AppError::Validation(
            "文件包含二进制内容，不是有效的文本文件".to_string(),
        ));
    }

    Ok(buffer)
}

fn markdown_title(path: &Path) -> Result<String, AppError> {
    let bytes = validate_text(path)?;
    let text = String::from_utf8_lossy(&bytes);

    for line in text.lines().take(50) {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let heading = rest.trim_start_matches('#').trim();
            if !heading.is_empty() {
                return Ok(truncate_chars(heading, MAX_TITLE_CHARS));
            }
        }
    }

    Ok(String::new())
}

fn extract_epub(path: &Path) -> Result<ExtractedMetadata, AppError> {
    let file = File::open(path).map_err(map_io_error)?;
    let mut archive = ZipArchive::new(file).map_err(|error| {
        log::error!("[AniShelf] EPUB ZIP 解析失败：{error}");
        AppError::Validation("EPUB 文件损坏或不是有效的 ZIP".to_string())
    })?;

    let container = read_zip_entry(&mut archive, "META-INF/container.xml", MAX_XML_BYTES)?
        .ok_or_else(|| AppError::Validation("EPUB 缺少 META-INF/container.xml".to_string()))?;
    let opf_path = parse_container_rootfile(&container)?;

    let opf = read_zip_entry(&mut archive, &opf_path, MAX_XML_BYTES)?
        .ok_or_else(|| AppError::Validation("EPUB 缺少内容清单文件".to_string()))?;
    let parsed = parse_opf(&opf)?;

    let cover = parsed.cover_href.as_deref().and_then(|href| {
        let resolved = resolve_archive_path(&opf_path, href)?;
        let extension = extension_lower(Path::new(&resolved));
        if !is_image_extension(&extension) {
            return None;
        }
        match read_zip_entry(&mut archive, &resolved, MAX_COVER_BYTES) {
            Ok(Some(bytes)) => Some(CoverData {
                bytes,
                extension: normalize_image_extension(&extension),
            }),
            Ok(None) => None,
            Err(error) => {
                log::error!("[AniShelf] EPUB 封面读取失败：{error}");
                None
            }
        }
    });

    Ok(ExtractedMetadata {
        title: parsed.title,
        author: parsed.author,
        description: parsed.description,
        cover,
    })
}

struct ParsedOpf {
    title: String,
    author: String,
    description: String,
    cover_href: Option<String>,
}

enum MetadataField {
    Title,
    Creator,
    Description,
}

fn parse_container_rootfile(xml: &[u8]) -> Result<String, AppError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) | Ok(Event::Empty(event)) => {
                if local_name(event.name().as_ref()) == "rootfile" {
                    for attribute in event.attributes().flatten() {
                        if local_name(attribute.key.as_ref()) == "full-path" {
                            let value = attribute_text(&attribute);
                            if !value.trim().is_empty() {
                                return Ok(value);
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => {
                log::error!("[AniShelf] container.xml 解析失败：{error}");
                return Err(AppError::Validation(
                    "EPUB container.xml 解析失败".to_string(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }

    Err(AppError::Validation("EPUB 缺少根文件信息".to_string()))
}

fn parse_opf(xml: &[u8]) -> Result<ParsedOpf, AppError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();

    let mut title = String::new();
    let mut author = String::new();
    let mut description = String::new();
    let mut current: Option<MetadataField> = None;
    let mut cover_id: Option<String> = None;
    let mut cover_href: Option<String> = None;
    let mut manifest: HashMap<String, String> = HashMap::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) | Ok(Event::Empty(event)) => {
                let name = event.name();
                let local = local_name(name.as_ref());
                match local {
                    "title" => current = Some(MetadataField::Title),
                    "creator" => current = Some(MetadataField::Creator),
                    "description" => current = Some(MetadataField::Description),
                    "meta" => {
                        let mut meta_name = None;
                        let mut meta_content = None;
                        for attribute in event.attributes().flatten() {
                            match local_name(attribute.key.as_ref()) {
                                "name" => meta_name = Some(attribute_text(&attribute)),
                                "content" => meta_content = Some(attribute_text(&attribute)),
                                _ => {}
                            }
                        }
                        if meta_name.as_deref() == Some("cover") {
                            cover_id = meta_content;
                        }
                    }
                    "item" => {
                        let mut id = None;
                        let mut href = None;
                        let mut is_cover_property = false;
                        for attribute in event.attributes().flatten() {
                            let key = local_name(attribute.key.as_ref());
                            let value = attribute_text(&attribute);
                            match key {
                                "id" => id = Some(value),
                                "href" => href = Some(value),
                                "properties" => {
                                    is_cover_property = value
                                        .split_whitespace()
                                        .any(|property| property == "cover-image");
                                }
                                _ => {}
                            }
                        }
                        if let (Some(id), Some(href)) = (id, href) {
                            manifest.insert(id, href.clone());
                            if is_cover_property && cover_href.is_none() {
                                cover_href = Some(href);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(event)) => {
                if let Some(field) = &current {
                    let raw = event.as_ref();
                    let text = quick_xml::escape::unescape(raw)
                        .map(|value| value.into_owned())
                        .unwrap_or_else(|_| raw.to_string());
                    match field {
                        MetadataField::Title => title.push_str(&text),
                        MetadataField::Creator => author.push_str(&text),
                        MetadataField::Description => description.push_str(&text),
                    }
                }
            }
            Ok(Event::End(event)) => {
                let name = event.name();
                let local = local_name(name.as_ref());
                if matches!(local, "title" | "creator" | "description") {
                    current = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => {
                log::error!("[AniShelf] OPF 解析失败：{error}");
                return Err(AppError::Validation("EPUB 内容清单解析失败".to_string()));
            }
            _ => {}
        }

        buffer.clear();
    }

    if cover_href.is_none()
        && let Some(id) = cover_id
    {
        cover_href = manifest.get(&id).cloned();
    }

    Ok(ParsedOpf {
        title: title.trim().to_string(),
        author: author.trim().to_string(),
        description: description.trim().to_string(),
        cover_href,
    })
}

fn read_zip_entry(
    archive: &mut ZipArchive<File>,
    name: &str,
    max_bytes: u64,
) -> Result<Option<Vec<u8>>, AppError> {
    let entry = match archive.by_name(name) {
        Ok(entry) => entry,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(error) => {
            log::error!("[AniShelf] EPUB 条目读取失败：{error}");
            return Err(AppError::Validation("EPUB 文件损坏".to_string()));
        }
    };

    if entry.size() > max_bytes {
        return Ok(None);
    }

    let mut bytes = Vec::with_capacity(entry.size() as usize);
    entry
        .take(max_bytes)
        .read_to_end(&mut bytes)
        .map_err(AppError::Io)?;

    Ok(Some(bytes))
}

fn resolve_archive_path(base_file: &str, href: &str) -> Option<String> {
    let href = href.split('#').next()?.replace('\\', "/");
    if href.starts_with('/') || href.contains(':') {
        return None;
    }

    let base_dir = base_file
        .rsplit_once('/')
        .map(|(directory, _)| directory)
        .unwrap_or("");
    let mut parts: Vec<&str> = Vec::new();

    for part in base_dir.split('/').chain(href.split('/')) {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }

    Some(parts.join("/"))
}

fn local_name(name: &str) -> &str {
    match name.rsplit_once(':') {
        Some((_, local)) => local,
        None => name,
    }
}

fn attribute_text(attribute: &quick_xml::events::attributes::Attribute) -> String {
    attribute
        .normalized_value(quick_xml::XmlVersion::Implicit1_0)
        .map(|value| value.into_owned())
        .unwrap_or_else(|_| attribute.value.as_ref().to_string())
}

fn sha256_file(path: &Path) -> Result<String, AppError> {
    let mut file = File::open(path).map_err(map_io_error)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer).map_err(map_io_error)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn format_for_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "epub" => Some("epub"),
        "pdf" => Some("pdf"),
        "txt" => Some("txt"),
        "md" | "markdown" => Some("markdown"),
        _ => None,
    }
}

fn extension_lower(path: &Path) -> String {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(|value| truncate_chars(value.trim(), MAX_TITLE_CHARS))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "未命名书籍".to_string())
}

fn file_mtime(metadata: &std::fs::Metadata) -> Option<i64> {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
}

fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(stripped) => stripped.to_string(),
        None => text.into_owned(),
    }
}

fn normalize_path(path: &str) -> String {
    path.replace('/', "\\").to_lowercase()
}

fn is_image_extension(extension: &str) -> bool {
    matches!(extension, "jpg" | "jpeg" | "png" | "webp" | "gif")
}

fn normalize_image_extension(extension: &str) -> String {
    match extension {
        "jpeg" => "jpg".to_string(),
        other => other.to_string(),
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn cleanup_files(files: &[PathBuf]) {
    for file in files {
        if let Err(error) = std::fs::remove_file(file)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            log::error!("[AniShelf] 清理文件失败 {}：{error}", file.display());
        }
    }
}

fn remove_app_files(paths: &AppPaths, row: &BookRow) {
    if let Some(cover) = &row.cover_path {
        let path = paths.data_dir.join(cover);
        if path.starts_with(&paths.covers_dir)
            && let Err(error) = std::fs::remove_file(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            log::error!("[AniShelf] 删除封面失败 {}：{error}", path.display());
        }
    }

    if row.source_path.is_some() {
        let path = PathBuf::from(&row.file_path);
        if path.starts_with(&paths.attachments_dir)
            && let Err(error) = std::fs::remove_file(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            log::error!("[AniShelf] 删除应用内副本失败 {}：{error}", path.display());
        }
    }
}

fn map_io_error(error: std::io::Error) -> AppError {
    match error.kind() {
        std::io::ErrorKind::NotFound => AppError::Validation("文件不存在".to_string()),
        std::io::ErrorKind::PermissionDenied => {
            AppError::Validation("没有读取该文件的权限".to_string())
        }
        _ => AppError::Io(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect, migrate};

    use sqlx::SqlitePool;
    use std::io::Write;
    use tempfile::{TempDir, tempdir};
    use zip::write::SimpleFileOptions;

    async fn setup() -> (TempDir, SqlitePool, AppPaths) {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("book-test.sqlite");
        let backups_dir = dir.path().join("backups");
        let pool = connect(&database_file).await.expect("connect");
        migrate::run(&pool, &backups_dir).await.expect("migrate");
        let paths = test_paths(dir.path());
        (dir, pool, paths)
    }

    fn test_paths(root: &Path) -> AppPaths {
        let data_dir = root.join("app-data");
        let paths = AppPaths {
            data_dir: data_dir.clone(),
            database_file: data_dir.join("database/anishelf.sqlite"),
            covers_dir: data_dir.join("covers"),
            attachments_dir: data_dir.join("attachments"),
            cache_dir: data_dir.join("cache"),
            backups_dir: data_dir.join("backups"),
            logs_dir: data_dir.join("logs"),
        };

        for directory in [
            &paths.covers_dir,
            &paths.attachments_dir,
            &paths.cache_dir,
            &paths.logs_dir,
        ] {
            std::fs::create_dir_all(directory).expect("create dir");
        }

        paths
    }

    fn write_epub(path: &Path) {
        let file = File::create(path).expect("epub file");
        let mut writer = zip::ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        writer.start_file("mimetype", options).expect("mimetype");
        writer.write_all(b"application/epub+zip").expect("mimetype");

        writer
            .start_file("META-INF/container.xml", options)
            .expect("container");
        writer
            .write_all(
                br#"<?xml version="1.0"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
            )
            .expect("container");

        writer
            .start_file("OEBPS/content.opf", options)
            .expect("opf");
        writer
            .write_all(
                r#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <metadata>
    <dc:title>测试 EPUB</dc:title>
    <dc:creator>测试作者</dc:creator>
    <dc:description>测试简介</dc:description>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="cover-img" href="images/cover.png" media-type="image/png" properties="cover-image"/>
  </manifest>
</package>"#
                    .as_bytes(),
            )
            .expect("opf");

        writer
            .start_file("OEBPS/images/cover.png", options)
            .expect("cover");
        writer.write_all(b"fake-png").expect("cover");
        writer.finish().expect("finish epub");
    }

    async fn import(
        pool: &SqlitePool,
        paths: &AppPaths,
        path: &Path,
        copy_to_library: bool,
    ) -> BookImportResult {
        import_book(
            pool,
            paths,
            path.to_string_lossy().into_owned(),
            copy_to_library,
        )
        .await
        .expect("import book")
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(pool)
            .await
            .expect("count")
    }

    #[tokio::test]
    async fn imports_epub_metadata_cover_and_detects_duplicates() {
        let (dir, pool, paths) = setup().await;
        let epub = dir.path().join("book.epub");
        write_epub(&epub);
        let import_path = epub.to_string_lossy().into_owned();

        let result = import_book(&pool, &paths, import_path.clone(), false)
            .await
            .expect("import");
        assert!(!result.duplicate);

        let book = result.book;
        assert_eq!(book.title, "测试 EPUB");
        assert_eq!(book.author, "测试作者");
        assert_eq!(book.description, "测试简介");
        assert_eq!(book.format, "epub");
        assert!(book.source_path.is_none());
        assert!(book.file_exists);

        let cover = book.cover_url.as_ref().expect("cover url");
        assert!(Path::new(cover).is_file());

        let duplicate = import_book(&pool, &paths, import_path, false)
            .await
            .expect("duplicate import");
        assert!(duplicate.duplicate);
        assert_eq!(duplicate.book.id, book.id);
    }

    #[tokio::test]
    async fn imports_txt_pdf_and_markdown_with_stem_or_heading_titles() {
        let (dir, pool, paths) = setup().await;

        let txt = dir.path().join("我的笔记.txt");
        std::fs::write(&txt, "hello world").expect("write txt");
        let txt_book = import(&pool, &paths, &txt, false).await.book;
        assert_eq!(txt_book.title, "我的笔记");
        assert_eq!(txt_book.format, "txt");

        let pdf = dir.path().join("文档.pdf");
        std::fs::write(&pdf, b"%PDF-1.4\n%%EOF").expect("write pdf");
        let pdf_book = import(&pool, &paths, &pdf, false).await.book;
        assert_eq!(pdf_book.title, "文档");
        assert_eq!(pdf_book.format, "pdf");

        let markdown = dir.path().join("读书笔记.md");
        std::fs::write(&markdown, "# 我的读书笔记\n\n正文").expect("write markdown");
        let markdown_book = import(&pool, &paths, &markdown, false).await.book;
        assert_eq!(markdown_book.title, "我的读书笔记");
        assert_eq!(markdown_book.format, "markdown");

        let all = list(&pool, &paths, &BookFilter::default())
            .await
            .expect("list");
        assert_eq!(all.len(), 3);

        let pdf_only = list(
            &pool,
            &paths,
            &BookFilter {
                format: Some("pdf".to_string()),
                ..BookFilter::default()
            },
        )
        .await
        .expect("filter format");
        assert_eq!(pdf_only.len(), 1);
        assert_eq!(pdf_only[0].id, pdf_book.id);
    }

    #[tokio::test]
    async fn rejects_invalid_corrupt_and_binary_files() {
        let (dir, pool, paths) = setup().await;

        let fake_epub = dir.path().join("fake.epub");
        std::fs::write(&fake_epub, b"not a zip").expect("fake epub");
        let error = import_book(
            &pool,
            &paths,
            fake_epub.to_string_lossy().into_owned(),
            false,
        )
        .await;
        assert!(matches!(error, Err(AppError::Validation(_))));

        let binary = dir.path().join("binary.txt");
        std::fs::write(&binary, b"hello\0world").expect("binary txt");
        let error = import_book(&pool, &paths, binary.to_string_lossy().into_owned(), false).await;
        assert!(matches!(error, Err(AppError::Validation(_))));

        let unsupported = dir.path().join("file.docx");
        std::fs::write(&unsupported, b"content").expect("docx");
        let error = import_book(
            &pool,
            &paths,
            unsupported.to_string_lossy().into_owned(),
            false,
        )
        .await;
        assert!(matches!(error, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn copy_to_library_never_touches_original_on_delete() {
        let (dir, pool, paths) = setup().await;
        let source = dir.path().join("source.txt");
        std::fs::write(&source, "copy me").expect("source");

        let book = import(&pool, &paths, &source, true).await.book;
        assert!(book.source_path.is_some());
        let copied = PathBuf::from(&book.file_path);
        assert!(copied.is_file());
        assert!(source.is_file());

        delete(&pool, &paths, book.id).await.expect("delete");

        assert!(source.is_file());
        assert!(!copied.exists());
    }

    #[tokio::test]
    async fn missing_file_can_be_relocated() {
        let (dir, pool, paths) = setup().await;
        let original = dir.path().join("moved.txt");
        std::fs::write(&original, "first").expect("original");

        let book = import(&pool, &paths, &original, false).await.book;
        std::fs::remove_file(&original).expect("remove original");

        let status = check_file(&pool, book.id).await.expect("check");
        assert!(!status.exists);

        let replacement = dir.path().join("moved-new.txt");
        std::fs::write(&replacement, "second").expect("replacement");
        let relocated = relocate(
            &pool,
            &paths,
            book.id,
            replacement.to_string_lossy().into_owned(),
        )
        .await
        .expect("relocate");

        assert!(relocated.file_exists);
        assert!(relocated.file_path.ends_with("moved-new.txt"));
        let status = check_file(&pool, book.id).await.expect("check");
        assert!(status.exists);
    }

    #[tokio::test]
    async fn tags_favorites_and_filters_work() {
        let (dir, pool, paths) = setup().await;
        let file = dir.path().join("filter.txt");
        std::fs::write(&file, "filter").expect("file");
        let book = import(&pool, &paths, &file, false).await.book;

        let updated = update(
            &pool,
            &paths,
            book.id,
            BookUpdateInput {
                title: "新书名".to_string(),
                author: "作者".to_string(),
                description: "简介".to_string(),
                category: "技术".to_string(),
                tags: vec!["Rust".to_string(), "rust".to_string()],
            },
        )
        .await
        .expect("update");
        assert_eq!(updated.title, "新书名");
        assert_eq!(updated.author, "作者");
        assert_eq!(updated.category, "技术");
        assert_eq!(updated.tags.len(), 1);

        let favorited = set_favorite(&pool, &paths, book.id, true)
            .await
            .expect("favorite");
        assert!(favorited.is_favorite);

        let filtered = list(
            &pool,
            &paths,
            &BookFilter {
                category: Some("技术".to_string()),
                favorite: Some(true),
                tag_id: Some(updated.tags[0].id),
                ..BookFilter::default()
            },
        )
        .await
        .expect("filter");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, book.id);
    }

    #[tokio::test]
    async fn deleting_book_cleans_polymorphic_relations() {
        let (dir, pool, paths) = setup().await;
        let file = dir.path().join("clean.txt");
        std::fs::write(&file, "clean").expect("file");
        let book = import(&pool, &paths, &file, false).await.book;

        let updated = update(
            &pool,
            &paths,
            book.id,
            BookUpdateInput {
                title: book.title.clone(),
                author: String::new(),
                description: String::new(),
                category: String::new(),
                tags: vec!["清理".to_string()],
            },
        )
        .await
        .expect("update");
        set_favorite(&pool, &paths, book.id, true)
            .await
            .expect("favorite");

        let annotation = sqlx::query(
            "INSERT INTO annotations
             (book_id, kind, selected_text, location_type, location_json, created_at, updated_at)
             VALUES (?, 'highlight', '摘录', 'txt_offset', '{}', 1, 1)",
        )
        .bind(book.id)
        .execute(&pool)
        .await
        .expect("annotation");
        let annotation_id = annotation.last_insert_rowid();

        sqlx::query(
            "INSERT INTO entity_tags (entity_type, entity_id, tag_id, created_at)
             VALUES ('annotation', ?, ?, 1)",
        )
        .bind(annotation_id)
        .bind(updated.tags[0].id)
        .execute(&pool)
        .await
        .expect("annotation tag");

        let note = sqlx::query(
            "INSERT INTO notes (title, content_md, created_at, updated_at)
             VALUES ('笔记', '', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("note");
        let note_id = note.last_insert_rowid();

        for (target_type, target_id) in [("book", book.id), ("annotation", annotation_id)] {
            sqlx::query(
                "INSERT INTO note_relations (note_id, target_type, target_id, created_at)
                 VALUES (?, ?, ?, 1)",
            )
            .bind(note_id)
            .bind(target_type)
            .bind(target_id)
            .execute(&pool)
            .await
            .expect("relation");
        }

        delete(&pool, &paths, book.id).await.expect("delete");

        assert_eq!(count(&pool, "books").await, 0);
        assert_eq!(count(&pool, "annotations").await, 0);
        assert_eq!(count(&pool, "note_relations").await, 0);
        assert_eq!(count(&pool, "entity_tags").await, 0);
        assert_eq!(count(&pool, "favorites").await, 0);
        assert_eq!(count(&pool, "notes").await, 1);
    }

    #[tokio::test]
    async fn duplicate_content_at_another_path_is_detected_by_hash() {
        let (dir, pool, paths) = setup().await;
        let first_path = dir.path().join("first.txt");
        std::fs::write(&first_path, "same content").expect("first");
        let second_path = dir.path().join("second.txt");
        std::fs::copy(&first_path, &second_path).expect("copy");

        let first = import(&pool, &paths, &first_path, false).await.book;
        let duplicate = import_book(
            &pool,
            &paths,
            second_path.to_string_lossy().into_owned(),
            false,
        )
        .await
        .expect("duplicate import");

        assert!(duplicate.duplicate);
        assert_eq!(duplicate.book.id, first.id);
    }

    #[test]
    fn archive_paths_cannot_escape_the_root() {
        assert_eq!(
            resolve_archive_path("OEBPS/content.opf", "images/cover.png").as_deref(),
            Some("OEBPS/images/cover.png")
        );
        assert!(resolve_archive_path("OEBPS/content.opf", "../../evil.png").is_none());
        assert!(resolve_archive_path("OEBPS/content.opf", "/absolute.png").is_none());
        assert!(
            resolve_archive_path("OEBPS/content.opf", "https://example.com/cover.png").is_none()
        );
    }

    fn fake_png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 2, 0, 0, 0]);
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes
    }

    fn png_data_url(width: u32, height: u32) -> String {
        format!(
            "data:image/png;base64,{}",
            BASE64.encode(fake_png(width, height))
        )
    }

    #[tokio::test]
    async fn generated_cover_is_validated_stored_and_cleaned_up() {
        let (dir, pool, paths) = setup().await;
        let file = dir.path().join("cover-source.txt");
        std::fs::write(&file, "cover source").expect("file");
        let book = import(&pool, &paths, &file, false).await.book;
        assert!(book.cover_path.is_none());

        let saved = save_generated_cover(&pool, &paths, book.id, &png_data_url(600, 800))
            .await
            .expect("save cover");
        let cover_path = saved.cover_path.clone().expect("cover path");
        assert!(cover_path.starts_with("covers/generated-cover-"));
        let full_path = paths.data_dir.join(&cover_path);
        assert!(full_path.is_file());

        let second = save_generated_cover(&pool, &paths, book.id, &png_data_url(300, 400))
            .await
            .expect("second save");
        assert_eq!(second.cover_path, Some(cover_path.clone()));

        assert!(decode_generated_cover(&png_data_url(10, 10)).is_err());
        assert!(decode_generated_cover("data:image/png;base64,AAAA").is_err());
        assert!(decode_generated_cover("data:image/jpeg;base64,AAAA").is_err());

        delete(&pool, &paths, book.id).await.expect("delete");
        assert!(!full_path.exists());
    }

    #[tokio::test]
    async fn generated_cover_is_rejected_for_epub_books() {
        let (_dir, pool, paths) = setup().await;
        let id = sqlx::query(
            "INSERT INTO books
             (title, format, file_path, normalized_path, file_size, imported_at, created_at, updated_at)
             VALUES ('EPUB 书', 'epub', '/tmp/epub.epub', '/tmp/epub.epub', 1, 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("book")
        .last_insert_rowid();

        let result = save_generated_cover(&pool, &paths, id, &png_data_url(600, 800)).await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
