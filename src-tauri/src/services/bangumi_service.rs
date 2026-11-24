use sqlx::SqlitePool;

use crate::error::AppError;
use crate::integrations::bangumi::{BangumiClient, BangumiError, BangumiSubject};
use crate::models::anime::{Anime, AnimeInput, ExternalAnimeFields};
use crate::repositories::anime_repository;
use crate::services::anime_service;
use crate::utils::now_ms;

const TOKEN_KEY: &str = "bangumi.token";
const MAX_TOKEN_CHARS: usize = 500;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BangumiSearchResult {
    pub subject_id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub summary: String,
    pub year: Option<i64>,
    pub season: Option<String>,
    pub total_episodes: Option<i64>,
    pub cover_url: Option<String>,
    pub air_date: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BangumiFieldChange {
    pub field: String,
    pub label: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BangumiRefreshPreview {
    pub subject_id: i64,
    pub title: String,
    pub changes: Vec<BangumiFieldChange>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BangumiTokenStatus {
    pub has_token: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BangumiConnectionStatus {
    pub ok: bool,
    pub message: String,
}

pub async fn search(
    pool: &SqlitePool,
    keyword: &str,
) -> Result<Vec<BangumiSearchResult>, AppError> {
    let keyword = keyword.trim();
    if keyword.is_empty() {
        return Err(AppError::Validation("请输入搜索关键词".to_string()));
    }
    if keyword.chars().count() > 100 {
        return Err(AppError::Validation(
            "搜索关键词不能超过 100 个字符".to_string(),
        ));
    }

    let client = client_from_settings(pool).await?;
    let subjects = client.search(keyword).await.map_err(map_bangumi_error)?;

    Ok(subjects.into_iter().map(search_result).collect())
}

pub async fn import_subject(pool: &SqlitePool, subject_id: i64) -> Result<Anime, AppError> {
    if subject_id <= 0 {
        return Err(AppError::Validation("Bangumi subject ID 无效".to_string()));
    }

    if let Some(existing_id) =
        anime_repository::find_id_by_bangumi_subject_id(pool, subject_id).await?
    {
        return anime_service::get(pool, existing_id).await;
    }

    let client = client_from_settings(pool).await?;
    let subject = client
        .subject(subject_id)
        .await
        .map_err(map_bangumi_error)?;

    import_subject_record(pool, &subject).await
}

pub async fn import_subject_record(
    pool: &SqlitePool,
    subject: &BangumiSubject,
) -> Result<Anime, AppError> {
    if let Some(existing_id) =
        anime_repository::find_id_by_bangumi_subject_id(pool, subject.id).await?
    {
        return anime_service::get(pool, existing_id).await;
    }

    let (input, fields) = import_payload(subject)?;
    let anime = anime_service::create_external(pool, input, fields).await?;

    let fields_updated = [
        "title",
        "originalTitle",
        "aliases",
        "summary",
        "coverSourceUrl",
        "year",
        "season",
        "totalEpisodes",
    ];

    if let Err(error) = insert_sync_log(
        pool,
        Some(anime.id),
        Some(subject.id),
        "success",
        "从 Bangumi 导入",
        &fields_updated,
        now_ms(),
    )
    .await
    {
        eprintln!("[AniShelf] 写入 Bangumi 同步日志失败：{error}");
    }

    Ok(anime)
}

pub async fn preview_refresh(
    pool: &SqlitePool,
    anime_id: i64,
) -> Result<BangumiRefreshPreview, AppError> {
    let anime = anime_service::get(pool, anime_id).await?;
    let subject_id = anime
        .bangumi_subject_id
        .ok_or_else(|| AppError::Validation("该番剧没有 Bangumi ID".to_string()))?;

    let client = client_from_settings(pool).await?;
    let subject = client
        .subject(subject_id)
        .await
        .map_err(map_bangumi_error)?;
    let (_, fields) = import_payload(&subject)?;
    let changes = diff(&anime, &fields);

    Ok(BangumiRefreshPreview {
        subject_id,
        title: anime.title,
        changes,
    })
}

pub async fn apply_refresh(pool: &SqlitePool, anime_id: i64) -> Result<Anime, AppError> {
    let anime = anime_service::get(pool, anime_id).await?;
    let subject_id = anime
        .bangumi_subject_id
        .ok_or_else(|| AppError::Validation("该番剧没有 Bangumi ID".to_string()))?;
    let started_at = now_ms();

    let client = client_from_settings(pool).await?;
    let subject = match client.subject(subject_id).await {
        Ok(subject) => subject,
        Err(error) => {
            let message = error.to_string();
            if let Err(log_error) = insert_sync_log(
                pool,
                Some(anime_id),
                Some(subject_id),
                "error",
                &message,
                &[],
                started_at,
            )
            .await
            {
                eprintln!("[AniShelf] 写入 Bangumi 同步日志失败：{log_error}");
            }
            return Err(map_bangumi_error(error));
        }
    };

    let (_, fields) = import_payload(&subject)?;
    let changes = diff(&anime, &fields);
    let changed_fields: Vec<&str> = changes.iter().map(|change| change.field.as_str()).collect();

    match anime_service::apply_external_fields(pool, anime_id, fields).await {
        Ok(updated) => {
            if let Err(error) = insert_sync_log(
                pool,
                Some(anime_id),
                Some(subject_id),
                "success",
                "从 Bangumi 刷新",
                &changed_fields,
                started_at,
            )
            .await
            {
                eprintln!("[AniShelf] 写入 Bangumi 同步日志失败：{error}");
            }
            Ok(updated)
        }
        Err(error) => {
            if let Err(log_error) = insert_sync_log(
                pool,
                Some(anime_id),
                Some(subject_id),
                "error",
                &error.to_string(),
                &[],
                started_at,
            )
            .await
            {
                eprintln!("[AniShelf] 写入 Bangumi 同步日志失败：{log_error}");
            }
            Err(error)
        }
    }
}

pub async fn token_status(pool: &SqlitePool) -> Result<BangumiTokenStatus, AppError> {
    Ok(BangumiTokenStatus {
        has_token: load_token(pool).await?.is_some(),
    })
}

pub async fn save_token(
    pool: &SqlitePool,
    token: Option<String>,
) -> Result<BangumiTokenStatus, AppError> {
    let token = token
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if let Some(token) = token.as_deref() {
        if token.chars().count() > MAX_TOKEN_CHARS {
            return Err(AppError::Validation(format!(
                "Token 不能超过 {MAX_TOKEN_CHARS} 个字符"
            )));
        }

        let value_json = serde_json::to_string(token)
            .map_err(|error| AppError::Internal(format!("Token 序列化失败：{error}")))?;

        sqlx::query(
            "INSERT INTO app_settings (key, value_json, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE
             SET value_json = excluded.value_json, updated_at = excluded.updated_at",
        )
        .bind(TOKEN_KEY)
        .bind(value_json)
        .bind(now_ms())
        .execute(pool)
        .await?;
    } else {
        sqlx::query("DELETE FROM app_settings WHERE key = ?")
            .bind(TOKEN_KEY)
            .execute(pool)
            .await?;
    }

    token_status(pool).await
}

pub async fn test_connection(pool: &SqlitePool) -> Result<BangumiConnectionStatus, AppError> {
    let client = client_from_settings(pool).await?;

    match client.subject(2).await {
        Ok(subject) => Ok(BangumiConnectionStatus {
            ok: true,
            message: format!("已连接 Bangumi（示例作品：{}）", subject.display_title()),
        }),
        Err(error) => Ok(BangumiConnectionStatus {
            ok: false,
            message: error.to_string(),
        }),
    }
}

async fn client_from_settings(pool: &SqlitePool) -> Result<BangumiClient, AppError> {
    let token = load_token(pool).await?;
    BangumiClient::new(token).map_err(|error| AppError::External(error.to_string()))
}

async fn load_token(pool: &SqlitePool) -> Result<Option<String>, AppError> {
    let raw: Option<String> =
        sqlx::query_scalar("SELECT value_json FROM app_settings WHERE key = ?")
            .bind(TOKEN_KEY)
            .fetch_optional(pool)
            .await?;

    Ok(raw
        .and_then(|value| serde_json::from_str::<String>(&value).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()))
}

fn search_result(subject: BangumiSubject) -> BangumiSearchResult {
    let (year, season) = subject.year_and_season();

    BangumiSearchResult {
        subject_id: subject.id,
        title: truncate_chars(subject.display_title().trim(), 300),
        original_title: subject.original_title(),
        summary: subject.summary.trim().to_string(),
        year,
        season,
        total_episodes: subject.total_episodes(),
        cover_url: subject.cover_url(),
        air_date: subject.date.clone(),
    }
}

fn import_payload(subject: &BangumiSubject) -> Result<(AnimeInput, ExternalAnimeFields), AppError> {
    let title = truncate_chars(subject.display_title().trim(), 300);
    if title.is_empty() {
        return Err(AppError::Validation("Bangumi 作品缺少标题".to_string()));
    }

    let original_title = subject
        .original_title()
        .map(|value| truncate_chars(value.trim(), 300))
        .filter(|value| !value.is_empty());

    let mut aliases: Vec<String> = Vec::new();
    for candidate in [subject.name.trim(), subject.name_cn.trim()] {
        if candidate.is_empty() || candidate == title {
            continue;
        }
        let candidate = truncate_chars(candidate, 100);
        if !aliases
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&candidate))
        {
            aliases.push(candidate);
        }
    }
    aliases.truncate(20);

    let summary = truncate_chars(subject.summary.trim(), 20_000);
    let (year, season) = subject.year_and_season();
    let total_episodes = subject.total_episodes();
    let now = now_ms();

    let input = AnimeInput {
        title: title.clone(),
        original_title: original_title.clone(),
        aliases: aliases.clone(),
        summary: summary.clone(),
        year,
        season: season.clone(),
        total_episodes,
        watch_status: "wish".to_string(),
        rating: None,
        review: String::new(),
        tags: Vec::new(),
    };

    let fields = ExternalAnimeFields {
        bangumi_subject_id: subject.id,
        title,
        original_title,
        aliases,
        summary,
        cover_source_url: subject.cover_url(),
        year,
        season,
        total_episodes,
        updated_at: now,
    };

    Ok((input, fields))
}

fn diff(anime: &Anime, fields: &ExternalAnimeFields) -> Vec<BangumiFieldChange> {
    let mut changes = Vec::new();

    push_change(&mut changes, "title", "标题", &anime.title, &fields.title);
    push_change(
        &mut changes,
        "originalTitle",
        "原名",
        &display_optional(anime.original_title.as_deref()),
        &display_optional(fields.original_title.as_deref()),
    );
    push_change(
        &mut changes,
        "aliases",
        "别名",
        &anime.aliases.join("、"),
        &fields.aliases.join("、"),
    );
    push_change(
        &mut changes,
        "summary",
        "简介",
        &anime.summary,
        &fields.summary,
    );
    push_change(
        &mut changes,
        "coverSourceUrl",
        "封面链接",
        &display_optional(anime.cover_source_url.as_deref()),
        &display_optional(fields.cover_source_url.as_deref()),
    );
    push_change(
        &mut changes,
        "year",
        "年份",
        &display_number(anime.year),
        &display_number(fields.year),
    );
    push_change(
        &mut changes,
        "season",
        "季度",
        &display_season(anime.season.as_deref()),
        &display_season(fields.season.as_deref()),
    );
    push_change(
        &mut changes,
        "totalEpisodes",
        "总集数",
        &display_number(anime.total_episodes),
        &display_number(fields.total_episodes),
    );

    changes
}

fn push_change(
    changes: &mut Vec<BangumiFieldChange>,
    field: &str,
    label: &str,
    old_value: &str,
    new_value: &str,
) {
    if old_value != new_value {
        changes.push(BangumiFieldChange {
            field: field.to_string(),
            label: label.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
        });
    }
}

fn display_optional(value: Option<&str>) -> String {
    match value {
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => "（空）".to_string(),
    }
}

fn display_number(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "（空）".to_string())
}

fn display_season(value: Option<&str>) -> String {
    match value {
        Some("winter") => "冬".to_string(),
        Some("spring") => "春".to_string(),
        Some("summer") => "夏".to_string(),
        Some("fall") => "秋".to_string(),
        Some(other) => other.to_string(),
        None => "（空）".to_string(),
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn map_bangumi_error(error: BangumiError) -> AppError {
    AppError::External(error.to_string())
}

async fn insert_sync_log(
    pool: &SqlitePool,
    anime_id: Option<i64>,
    subject_id: Option<i64>,
    status: &str,
    message: &str,
    fields_updated: &[&str],
    started_at: i64,
) -> Result<(), AppError> {
    let fields_json = serde_json::to_string(fields_updated)
        .map_err(|error| AppError::Internal(format!("同步字段序列化失败：{error}")))?;
    let finished_at = now_ms();

    sqlx::query(
        "INSERT INTO anime_metadata_sync_logs
         (anime_id, bangumi_subject_id, status, message, fields_updated_json, started_at, finished_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(anime_id)
    .bind(subject_id)
    .bind(status)
    .bind(message)
    .bind(fields_json)
    .bind(started_at)
    .bind(finished_at)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect, migrate};
    use crate::models::anime::AnimeInput;
    use crate::services::episode_service;

    use sqlx::SqlitePool;
    use tempfile::{TempDir, tempdir};

    const SUBJECT_JSON: &str = r#"{
        "id": 400602,
        "name": "葬送のフリーレン",
        "name_cn": "葬送的芙莉莲",
        "summary": "魔法使的旅途",
        "date": "2023-09-29",
        "eps": 12,
        "images": {
            "large": "https://example.com/large.jpg"
        }
    }"#;

    fn subject() -> BangumiSubject {
        serde_json::from_str(SUBJECT_JSON).expect("subject")
    }

    fn changed_subject() -> BangumiSubject {
        serde_json::from_str(
            r#"{
                "id": 400602,
                "name": "葬送のフリーレン",
                "name_cn": "葬送的芙莉莲 第二季",
                "summary": "新的简介",
                "date": "2024-04-05",
                "eps": 13,
                "images": {
                    "large": "https://example.com/new.jpg"
                }
            }"#,
        )
        .expect("subject")
    }

    fn anime_input(
        status: &str,
        rating: Option<i64>,
        review: &str,
        tags: Vec<String>,
    ) -> AnimeInput {
        AnimeInput {
            title: "个人标题".to_string(),
            original_title: None,
            aliases: Vec::new(),
            summary: "个人简介".to_string(),
            year: None,
            season: None,
            total_episodes: Some(12),
            watch_status: status.to_string(),
            rating,
            review: review.to_string(),
            tags,
        }
    }

    async fn migrated_pool() -> (TempDir, SqlitePool) {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("bangumi-test.sqlite");
        let backups_dir = dir.path().join("backups");
        let pool = connect(&database_file).await.expect("connect");
        migrate::run(&pool, &backups_dir).await.expect("migrate");
        (dir, pool)
    }

    #[tokio::test]
    async fn imports_subject_with_external_fields_and_defaults() {
        let (_dir, pool) = migrated_pool().await;

        let anime = import_subject_record(&pool, &subject())
            .await
            .expect("import");

        assert_eq!(anime.title, "葬送的芙莉莲");
        assert_eq!(anime.bangumi_subject_id, Some(400602));
        assert_eq!(anime.original_title.as_deref(), Some("葬送のフリーレン"));
        assert_eq!(anime.year, Some(2023));
        assert_eq!(anime.season.as_deref(), Some("summer"));
        assert_eq!(anime.total_episodes, Some(12));
        assert_eq!(anime.watch_status, "wish");
        assert!(anime.rating.is_none());
        assert!(anime.review.is_empty());
        assert!(anime.tags.is_empty());
        assert_eq!(
            anime.cover_source_url.as_deref(),
            Some("https://example.com/large.jpg")
        );

        let episodes = episode_service::list_episodes(&pool, anime.id)
            .await
            .expect("episodes");
        assert_eq!(episodes.len(), 12);

        let again = import_subject_record(&pool, &subject())
            .await
            .expect("re-import");
        assert_eq!(again.id, anime.id);

        let log_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM anime_metadata_sync_logs")
            .fetch_one(&pool)
            .await
            .expect("log count");
        assert_eq!(log_count, 1);
    }

    #[tokio::test]
    async fn refresh_preserves_personal_fields() {
        let (_dir, pool) = migrated_pool().await;

        let anime = import_subject_record(&pool, &subject())
            .await
            .expect("import");

        let updated_personal = anime_service::update(
            &pool,
            anime.id,
            anime_input(
                "completed",
                Some(9),
                "个人评论",
                vec!["个人标签".to_string()],
            ),
        )
        .await
        .expect("update personal");
        assert_eq!(updated_personal.watch_status, "completed");
        assert_eq!(updated_personal.rating, Some(9));

        let (_, fields) = import_payload(&changed_subject()).expect("payload");
        let refreshed = anime_service::apply_external_fields(&pool, anime.id, fields)
            .await
            .expect("refresh");

        assert_eq!(refreshed.title, "葬送的芙莉莲 第二季");
        assert_eq!(refreshed.summary, "新的简介");
        assert_eq!(refreshed.total_episodes, Some(13));
        assert_eq!(refreshed.watch_status, "completed");
        assert_eq!(refreshed.rating, Some(9));
        assert_eq!(refreshed.review, "个人评论");
        assert_eq!(refreshed.tags.len(), 1);
        assert_eq!(refreshed.tags[0].name, "个人标签");

        let episodes = episode_service::list_episodes(&pool, anime.id)
            .await
            .expect("episodes");
        assert_eq!(episodes.len(), 13);
    }

    #[tokio::test]
    async fn diff_only_reports_external_changes() {
        let (_dir, pool) = migrated_pool().await;
        let anime = import_subject_record(&pool, &subject())
            .await
            .expect("import");

        let (_, same_fields) = import_payload(&subject()).expect("payload");
        assert!(diff(&anime, &same_fields).is_empty());

        let (_, changed_fields) = import_payload(&changed_subject()).expect("payload");
        let changes = diff(&anime, &changed_fields);
        let fields: Vec<&str> = changes.iter().map(|change| change.field.as_str()).collect();

        assert!(fields.contains(&"title"));
        assert!(fields.contains(&"summary"));
        assert!(fields.contains(&"totalEpisodes"));
        assert!(fields.contains(&"year"));
        assert!(!fields.contains(&"watchStatus"));
        assert!(!fields.contains(&"rating"));
    }

    #[tokio::test]
    async fn token_roundtrip_never_exposes_value_in_status() {
        let (_dir, pool) = migrated_pool().await;

        let initial = token_status(&pool).await.expect("status");
        assert!(!initial.has_token);

        let saved = save_token(&pool, Some("  secret-token  ".to_string()))
            .await
            .expect("save");
        assert!(saved.has_token);
        assert_eq!(
            load_token(&pool).await.expect("load").as_deref(),
            Some("secret-token")
        );

        let cleared = save_token(&pool, Some("   ".to_string()))
            .await
            .expect("clear");
        assert!(!cleared.has_token);
        assert!(load_token(&pool).await.expect("load").is_none());
    }

    #[test]
    fn import_payload_truncates_and_builds_aliases() {
        let (input, fields) = import_payload(&subject()).expect("payload");

        assert_eq!(input.watch_status, "wish");
        assert!(input.tags.is_empty());
        assert!(input.aliases.contains(&"葬送のフリーレン".to_string()));
        assert_eq!(fields.bangumi_subject_id, 400602);
    }
}
