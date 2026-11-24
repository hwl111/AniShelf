use std::collections::{HashMap, HashSet};

use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::anime::{
    Anime, AnimeFilter, AnimeInput, AnimeRow, SEASONS, SORT_OPTIONS, WATCH_STATUSES,
};
use crate::models::tag::Tag;
use crate::repositories::anime_repository as repository;
use crate::repositories::episode_repository;
use crate::utils::now_ms;

const MAX_TITLE_CHARS: usize = 300;
const MAX_TEXT_CHARS: usize = 20_000;
const MAX_ALIASES: usize = 20;
const MAX_ALIAS_CHARS: usize = 100;
const MAX_TAGS: usize = 20;
const MAX_TAG_CHARS: usize = 50;

pub async fn list(pool: &SqlitePool, filter: &AnimeFilter) -> Result<Vec<Anime>, AppError> {
    validate_filter(filter)?;

    let rows = repository::list(pool, filter).await?;
    let all_tags = repository::all_anime_tags(pool).await?;
    let favorite_ids: HashSet<i64> = repository::favorite_ids(pool).await?.into_iter().collect();

    let mut tags_by_anime: HashMap<i64, Vec<Tag>> = HashMap::new();
    for (anime_id, tag) in all_tags {
        tags_by_anime.entry(anime_id).or_default().push(tag);
    }

    Ok(rows
        .into_iter()
        .map(|row| {
            let tags = tags_by_anime.remove(&row.id).unwrap_or_default();
            let is_favorite = favorite_ids.contains(&row.id);
            build_anime(row, tags, is_favorite)
        })
        .collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Anime, AppError> {
    let mut connection = pool.acquire().await?;
    let row = repository::get_row(&mut connection, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("番剧 {id} 不存在")))?;
    drop(connection);

    let tags = repository::tags_for(pool, id).await?;
    let is_favorite = repository::is_favorite(pool, id).await?;

    Ok(build_anime(row, tags, is_favorite))
}

pub async fn create(pool: &SqlitePool, input: AnimeInput) -> Result<Anime, AppError> {
    let input = normalize_input(input)?;
    let aliases_json = aliases_json(&input.aliases)?;
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let id = repository::insert(&mut transaction, &input, &aliases_json, now).await?;
    repository::set_tags(&mut transaction, id, &input.tags, now).await?;
    episode_repository::ensure_episodes(&mut transaction, id, input.total_episodes, now).await?;
    transaction.commit().await?;

    get(pool, id).await
}

pub async fn update(pool: &SqlitePool, id: i64, input: AnimeInput) -> Result<Anime, AppError> {
    let input = normalize_input(input)?;
    let aliases_json = aliases_json(&input.aliases)?;
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let existing = repository::get_row(&mut transaction, id).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!("番剧 {id} 不存在")));
    }

    repository::update(&mut transaction, id, &input, &aliases_json, now).await?;
    repository::set_tags(&mut transaction, id, &input.tags, now).await?;
    episode_repository::ensure_episodes(&mut transaction, id, input.total_episodes, now).await?;
    transaction.commit().await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let mut transaction = pool.begin().await?;
    let existing = repository::get_row(&mut transaction, id).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!("番剧 {id} 不存在")));
    }

    repository::cleanup_polymorphic(&mut transaction, id).await?;
    repository::delete(&mut transaction, id).await?;
    transaction.commit().await?;

    Ok(())
}

pub async fn set_favorite(pool: &SqlitePool, id: i64, favorite: bool) -> Result<Anime, AppError> {
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let existing = repository::get_row(&mut transaction, id).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!("番剧 {id} 不存在")));
    }

    repository::set_favorite(&mut transaction, id, favorite, now).await?;
    transaction.commit().await?;

    get(pool, id).await
}

pub async fn list_tags(pool: &SqlitePool) -> Result<Vec<Tag>, AppError> {
    Ok(repository::list_tags(pool).await?)
}

fn build_anime(row: AnimeRow, tags: Vec<Tag>, is_favorite: bool) -> Anime {
    let aliases = serde_json::from_str::<Vec<String>>(&row.aliases_json).unwrap_or_default();

    Anime {
        id: row.id,
        bangumi_subject_id: row.bangumi_subject_id,
        title: row.title,
        original_title: row.original_title,
        aliases,
        summary: row.summary,
        cover_path: row.cover_path,
        cover_source_url: row.cover_source_url,
        year: row.year,
        season: row.season,
        total_episodes: row.total_episodes,
        external_updated_at: row.external_updated_at,
        watch_status: row.watch_status,
        rating: row.rating,
        review: row.review,
        created_at: row.created_at,
        updated_at: row.updated_at,
        watched_episodes: row.watched_episodes,
        tags,
        is_favorite,
    }
}

fn normalize_input(mut input: AnimeInput) -> Result<AnimeInput, AppError> {
    input.title = input.title.trim().to_string();
    if input.title.is_empty() {
        return Err(AppError::Validation("标题不能为空".to_string()));
    }
    if input.title.chars().count() > MAX_TITLE_CHARS {
        return Err(AppError::Validation(format!(
            "标题不能超过 {MAX_TITLE_CHARS} 个字符"
        )));
    }

    input.original_title = input
        .original_title
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    input.summary = input.summary.trim().to_string();
    input.review = input.review.trim().to_string();
    if input.summary.chars().count() > MAX_TEXT_CHARS {
        return Err(AppError::Validation(format!(
            "简介不能超过 {MAX_TEXT_CHARS} 个字符"
        )));
    }
    if input.review.chars().count() > MAX_TEXT_CHARS {
        return Err(AppError::Validation(format!(
            "评论不能超过 {MAX_TEXT_CHARS} 个字符"
        )));
    }

    input.watch_status = input.watch_status.trim().to_lowercase();
    if !WATCH_STATUSES.contains(&input.watch_status.as_str()) {
        return Err(AppError::Validation("观看状态无效".to_string()));
    }

    if let Some(rating) = input.rating
        && !(1..=10).contains(&rating)
    {
        return Err(AppError::Validation("评分必须在 1 到 10 之间".to_string()));
    }

    if let Some(year) = input.year
        && !(1900..=2200).contains(&year)
    {
        return Err(AppError::Validation(
            "年份必须在 1900 到 2200 之间".to_string(),
        ));
    }

    input.season = input
        .season
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());
    if let Some(season) = input.season.as_deref()
        && !SEASONS.contains(&season)
    {
        return Err(AppError::Validation("季度无效".to_string()));
    }

    if let Some(total_episodes) = input.total_episodes
        && !(0..=10_000).contains(&total_episodes)
    {
        return Err(AppError::Validation(
            "总集数必须在 0 到 10000 之间".to_string(),
        ));
    }

    input.aliases = normalize_list(input.aliases, MAX_ALIASES, MAX_ALIAS_CHARS, "别名", false)?;
    input.tags = normalize_list(input.tags, MAX_TAGS, MAX_TAG_CHARS, "标签", true)?;

    Ok(input)
}

fn validate_filter(filter: &AnimeFilter) -> Result<(), AppError> {
    if let Some(status) = filter.status.as_deref()
        && !WATCH_STATUSES.contains(&status)
    {
        return Err(AppError::Validation("观看状态筛选无效".to_string()));
    }

    if let Some(sort) = filter.sort.as_deref()
        && !SORT_OPTIONS.contains(&sort)
    {
        return Err(AppError::Validation("排序方式无效".to_string()));
    }

    if let Some(year) = filter.year
        && !(1900..=2200).contains(&year)
    {
        return Err(AppError::Validation("年份筛选无效".to_string()));
    }

    Ok(())
}

fn normalize_list(
    values: Vec<String>,
    max_items: usize,
    max_chars: usize,
    label: &str,
    case_insensitive: bool,
) -> Result<Vec<String>, AppError> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().count() > max_chars {
            return Err(AppError::Validation(format!(
                "{label}单个不能超过 {max_chars} 个字符"
            )));
        }

        let key = if case_insensitive {
            trimmed.to_lowercase()
        } else {
            trimmed.to_string()
        };

        if seen.insert(key) {
            result.push(trimmed.to_string());
        }
    }

    if result.len() > max_items {
        return Err(AppError::Validation(format!("{label}最多 {max_items} 个")));
    }

    Ok(result)
}

fn aliases_json(aliases: &[String]) -> Result<String, AppError> {
    serde_json::to_string(aliases)
        .map_err(|error| AppError::Internal(format!("别名序列化失败：{error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect, migrate};

    use sqlx::SqlitePool;
    use tempfile::{TempDir, tempdir};

    async fn migrated_pool() -> (TempDir, SqlitePool) {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("anime-test.sqlite");
        let backups_dir = dir.path().join("backups");
        let pool = connect(&database_file).await.expect("connect");
        migrate::run(&pool, &backups_dir).await.expect("migrate");
        (dir, pool)
    }

    fn input(title: &str, status: &str) -> AnimeInput {
        AnimeInput {
            title: title.to_string(),
            original_title: None,
            aliases: Vec::new(),
            summary: String::new(),
            year: None,
            season: None,
            total_episodes: None,
            watch_status: status.to_string(),
            rating: None,
            review: String::new(),
            tags: Vec::new(),
        }
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(pool)
            .await
            .expect("count")
    }

    #[tokio::test]
    async fn anime_crud_round_trip() {
        let (_dir, pool) = migrated_pool().await;

        let mut create_input = input("葬送的芙莉莲", "watching");
        create_input.original_title = Some("Sousou no Frieren".to_string());
        create_input.aliases = vec!["芙莉莲".to_string()];
        create_input.summary = "魔法使的旅途".to_string();
        create_input.year = Some(2023);
        create_input.season = Some("fall".to_string());
        create_input.total_episodes = Some(28);
        create_input.rating = Some(9);
        create_input.review = "很喜欢".to_string();
        create_input.tags = vec!["奇幻".to_string(), "旅途".to_string()];

        let created = create(&pool, create_input).await.expect("create");
        assert_eq!(created.title, "葬送的芙莉莲");
        assert_eq!(created.watch_status, "watching");
        assert_eq!(created.rating, Some(9));
        assert_eq!(created.watched_episodes, 0);
        assert_eq!(created.tags.len(), 2);
        assert!(!created.is_favorite);

        let listed = list(&pool, &AnimeFilter::default()).await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);

        let mut update_input = input("葬送的芙莉莲 第二季", "completed");
        update_input.rating = Some(10);
        update_input.tags = vec!["完成".to_string()];

        let updated = update(&pool, created.id, update_input)
            .await
            .expect("update");
        assert_eq!(updated.title, "葬送的芙莉莲 第二季");
        assert_eq!(updated.watch_status, "completed");
        assert_eq!(updated.rating, Some(10));
        assert_eq!(updated.tags.len(), 1);
        assert_eq!(updated.tags[0].name, "完成");

        delete(&pool, created.id).await.expect("delete");
        let missing = get(&pool, created.id).await;
        assert!(matches!(missing, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn filters_and_sorting_work() {
        let (_dir, pool) = migrated_pool().await;

        let mut first = input("B 番剧", "watching");
        first.year = Some(2024);
        first.rating = Some(7);
        first.tags = vec!["标签一".to_string()];
        let first = create(&pool, first).await.expect("create first");

        let mut second = input("A 番剧", "completed");
        second.year = Some(2023);
        second.rating = Some(9);
        second.tags = vec!["标签二".to_string()];
        create(&pool, second).await.expect("create second");

        let watching = list(
            &pool,
            &AnimeFilter {
                status: Some("watching".to_string()),
                ..AnimeFilter::default()
            },
        )
        .await
        .expect("filter status");
        assert_eq!(watching.len(), 1);
        assert_eq!(watching[0].id, first.id);

        let by_year = list(
            &pool,
            &AnimeFilter {
                year: Some(2023),
                ..AnimeFilter::default()
            },
        )
        .await
        .expect("filter year");
        assert_eq!(by_year.len(), 1);

        let tag_id = first.tags[0].id;
        let by_tag = list(
            &pool,
            &AnimeFilter {
                tag_id: Some(tag_id),
                ..AnimeFilter::default()
            },
        )
        .await
        .expect("filter tag");
        assert_eq!(by_tag.len(), 1);
        assert_eq!(by_tag[0].id, first.id);

        let sorted = list(
            &pool,
            &AnimeFilter {
                sort: Some("title_asc".to_string()),
                ..AnimeFilter::default()
            },
        )
        .await
        .expect("sort title");
        assert_eq!(sorted[0].title, "A 番剧");

        let by_rating = list(
            &pool,
            &AnimeFilter {
                sort: Some("rating_desc".to_string()),
                ..AnimeFilter::default()
            },
        )
        .await
        .expect("sort rating");
        assert_eq!(by_rating[0].rating, Some(9));
    }

    #[tokio::test]
    async fn favorite_toggle_round_trip() {
        let (_dir, pool) = migrated_pool().await;
        let anime = create(&pool, input("收藏测试", "wish"))
            .await
            .expect("create");

        let favorited = set_favorite(&pool, anime.id, true).await.expect("favorite");
        assert!(favorited.is_favorite);

        let listed = list(&pool, &AnimeFilter::default()).await.expect("list");
        assert!(listed[0].is_favorite);

        let unfavorited = set_favorite(&pool, anime.id, false)
            .await
            .expect("unfavorite");
        assert!(!unfavorited.is_favorite);
    }

    #[tokio::test]
    async fn delete_cleans_polymorphic_relations_and_cascades() {
        let (_dir, pool) = migrated_pool().await;

        let mut create_input = input("删除测试", "watching");
        create_input.tags = vec!["测试标签".to_string()];
        let anime = create(&pool, create_input).await.expect("create");
        set_favorite(&pool, anime.id, true).await.expect("favorite");

        let episode = sqlx::query(
            "INSERT INTO anime_episodes (anime_id, episode_number, created_at, updated_at)
             VALUES (?, 1, 1, 1)",
        )
        .bind(anime.id)
        .execute(&pool)
        .await
        .expect("insert episode");
        let episode_id = episode.last_insert_rowid();

        let watch_record = sqlx::query(
            "INSERT INTO watch_records (anime_id, episode_id, episode_number, watched_at, created_at)
             VALUES (?, ?, 1, 1, 1)",
        )
        .bind(anime.id)
        .bind(episode_id)
        .execute(&pool)
        .await
        .expect("insert watch record");
        let watch_record_id = watch_record.last_insert_rowid();

        let note = sqlx::query(
            "INSERT INTO notes (title, content_md, created_at, updated_at)
             VALUES ('关联笔记', '', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("insert note");
        let note_id = note.last_insert_rowid();

        for (target_type, target_id) in [
            ("anime", anime.id),
            ("episode", episode_id),
            ("watch_record", watch_record_id),
        ] {
            sqlx::query(
                "INSERT INTO note_relations (note_id, target_type, target_id, created_at)
                 VALUES (?, ?, ?, 1)",
            )
            .bind(note_id)
            .bind(target_type)
            .bind(target_id)
            .execute(&pool)
            .await
            .expect("insert relation");
        }

        delete(&pool, anime.id).await.expect("delete");

        assert_eq!(count(&pool, "anime").await, 0);
        assert_eq!(count(&pool, "anime_episodes").await, 0);
        assert_eq!(count(&pool, "watch_records").await, 0);
        assert_eq!(count(&pool, "entity_tags").await, 0);
        assert_eq!(count(&pool, "favorites").await, 0);
        assert_eq!(count(&pool, "note_relations").await, 0);
        assert_eq!(count(&pool, "notes").await, 1);
    }

    #[tokio::test]
    async fn validation_rejects_invalid_values() {
        let (_dir, pool) = migrated_pool().await;

        let empty_title = create(&pool, input("   ", "wish")).await;
        assert!(matches!(empty_title, Err(AppError::Validation(_))));

        let bad_status = create(&pool, input("测试", "unknown")).await;
        assert!(matches!(bad_status, Err(AppError::Validation(_))));

        let mut bad_rating = input("测试", "wish");
        bad_rating.rating = Some(11);
        assert!(matches!(
            create(&pool, bad_rating).await,
            Err(AppError::Validation(_))
        ));

        let mut bad_year = input("测试", "wish");
        bad_year.year = Some(1800);
        assert!(matches!(
            create(&pool, bad_year).await,
            Err(AppError::Validation(_))
        ));

        let mut bad_season = input("测试", "wish");
        bad_season.season = Some("spring-summer".to_string());
        assert!(matches!(
            create(&pool, bad_season).await,
            Err(AppError::Validation(_))
        ));

        let mut too_many_tags = input("测试", "wish");
        too_many_tags.tags = (0..21).map(|index| format!("标签{index}")).collect();
        assert!(matches!(
            create(&pool, too_many_tags).await,
            Err(AppError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn tag_names_are_case_insensitive_and_deduplicated() {
        let (_dir, pool) = migrated_pool().await;

        let mut create_input = input("标签测试", "wish");
        create_input.tags = vec!["Rust".to_string(), "rust".to_string(), " 动漫 ".to_string()];
        let anime = create(&pool, create_input).await.expect("create");

        assert_eq!(anime.tags.len(), 2);
        let names: Vec<&str> = anime.tags.iter().map(|tag| tag.name.as_str()).collect();
        assert!(names.contains(&"Rust"));
        assert!(names.contains(&"动漫"));

        let tags = list_tags(&pool).await.expect("list tags");
        assert_eq!(tags.len(), 2);
    }

    #[tokio::test]
    async fn watched_episode_count_uses_distinct_episode_numbers() {
        let (_dir, pool) = migrated_pool().await;
        let anime = create(&pool, input("观看进度", "watching"))
            .await
            .expect("create");

        for episode_number in [1, 1, 2] {
            sqlx::query(
                "INSERT INTO watch_records
                 (anime_id, episode_id, episode_number, watched_at, created_at)
                 VALUES (?, NULL, ?, 1, 1)",
            )
            .bind(anime.id)
            .bind(episode_number)
            .execute(&pool)
            .await
            .expect("insert watch record");
        }

        let refreshed = get(&pool, anime.id).await.expect("get");
        assert_eq!(refreshed.watched_episodes, 2);
    }
}
