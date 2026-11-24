use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::episode::{Episode, EpisodeInput, WatchRecord, WatchRecordInput};
use crate::repositories::episode_repository as repository;
use crate::utils::now_ms;

const MAX_EPISODE_NUMBER: i64 = 10_000;
const MAX_TITLE_CHARS: usize = 200;
const MAX_COMMENT_CHARS: usize = 2_000;

pub async fn list_episodes(pool: &SqlitePool, anime_id: i64) -> Result<Vec<Episode>, AppError> {
    ensure_anime(pool, anime_id).await?;
    Ok(repository::list(pool, anime_id).await?)
}

pub async fn create_episode(
    pool: &SqlitePool,
    anime_id: i64,
    input: EpisodeInput,
) -> Result<Episode, AppError> {
    let input = normalize_episode_input(input)?;
    ensure_anime(pool, anime_id).await?;
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let episode_id = repository::insert(&mut transaction, anime_id, &input, now)
        .await
        .map_err(map_episode_error)?;
    transaction.commit().await?;

    get_episode(pool, episode_id).await
}

pub async fn update_episode(
    pool: &SqlitePool,
    episode_id: i64,
    input: EpisodeInput,
) -> Result<Episode, AppError> {
    let input = normalize_episode_input(input)?;
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let existing = repository::get(&mut transaction, episode_id).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!("单集 {episode_id} 不存在")));
    }

    repository::update(&mut transaction, episode_id, &input, now)
        .await
        .map_err(map_episode_error)?;
    transaction.commit().await?;

    get_episode(pool, episode_id).await
}

pub async fn delete_episode(pool: &SqlitePool, episode_id: i64) -> Result<(), AppError> {
    let mut transaction = pool.begin().await?;
    let existing = repository::get(&mut transaction, episode_id).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!("单集 {episode_id} 不存在")));
    }

    repository::cleanup_note_relations(&mut transaction, episode_id).await?;
    repository::delete(&mut transaction, episode_id).await?;
    transaction.commit().await?;

    Ok(())
}

pub async fn set_episode_watched(
    pool: &SqlitePool,
    episode_id: i64,
    watched: bool,
    watched_at: Option<i64>,
    comment: Option<String>,
) -> Result<Episode, AppError> {
    let watched_at = normalize_watched_at(watched_at)?;
    let comment = normalize_comment(comment.unwrap_or_default())?;
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let episode = repository::get(&mut transaction, episode_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("单集 {episode_id} 不存在")))?;

    if watched {
        repository::insert_watch_record(
            &mut transaction,
            episode.anime_id,
            Some(episode.id),
            episode.episode_number,
            watched_at.unwrap_or(now),
            &comment,
            now,
        )
        .await?;
    } else {
        repository::delete_watch_records_for_episode(&mut transaction, episode_id).await?;
    }

    transaction.commit().await?;

    get_episode(pool, episode_id).await
}

pub async fn list_watch_records(
    pool: &SqlitePool,
    anime_id: i64,
) -> Result<Vec<WatchRecord>, AppError> {
    ensure_anime(pool, anime_id).await?;
    Ok(repository::list_watch_records(pool, anime_id).await?)
}

pub async fn add_watch_record(
    pool: &SqlitePool,
    anime_id: i64,
    input: WatchRecordInput,
) -> Result<WatchRecord, AppError> {
    let input = normalize_watch_input(input)?;
    ensure_anime(pool, anime_id).await?;
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let episode_id =
        repository::find_episode_id(&mut transaction, anime_id, input.episode_number).await?;
    let record_id = repository::insert_watch_record(
        &mut transaction,
        anime_id,
        episode_id,
        input.episode_number,
        input.watched_at.unwrap_or(now),
        &input.comment,
        now,
    )
    .await?;
    transaction.commit().await?;

    get_watch_record(pool, record_id).await
}

pub async fn update_watch_record(
    pool: &SqlitePool,
    record_id: i64,
    watched_at: Option<i64>,
    comment: String,
) -> Result<WatchRecord, AppError> {
    let watched_at = normalize_watched_at(watched_at)?;
    let comment = normalize_comment(comment)?;

    let mut transaction = pool.begin().await?;
    let existing = repository::get_watch_record(&mut transaction, record_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("观看记录 {record_id} 不存在")))?;

    repository::update_watch_record(
        &mut transaction,
        record_id,
        watched_at.unwrap_or(existing.watched_at),
        &comment,
    )
    .await?;
    transaction.commit().await?;

    get_watch_record(pool, record_id).await
}

pub async fn delete_watch_record(pool: &SqlitePool, record_id: i64) -> Result<(), AppError> {
    let mut transaction = pool.begin().await?;
    let existing = repository::get_watch_record(&mut transaction, record_id).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!("观看记录 {record_id} 不存在")));
    }

    repository::delete_watch_record(&mut transaction, record_id).await?;
    transaction.commit().await?;

    Ok(())
}

async fn get_episode(pool: &SqlitePool, episode_id: i64) -> Result<Episode, AppError> {
    let mut connection = pool.acquire().await?;
    repository::get(&mut connection, episode_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("单集 {episode_id} 不存在")))
}

async fn get_watch_record(pool: &SqlitePool, record_id: i64) -> Result<WatchRecord, AppError> {
    let mut connection = pool.acquire().await?;
    repository::get_watch_record(&mut connection, record_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("观看记录 {record_id} 不存在")))
}

async fn ensure_anime(pool: &SqlitePool, anime_id: i64) -> Result<(), AppError> {
    if repository::anime_exists(pool, anime_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound(format!("番剧 {anime_id} 不存在")))
    }
}

fn normalize_episode_input(mut input: EpisodeInput) -> Result<EpisodeInput, AppError> {
    if !(1..=MAX_EPISODE_NUMBER).contains(&input.episode_number) {
        return Err(AppError::Validation(format!(
            "集数必须在 1 到 {MAX_EPISODE_NUMBER} 之间"
        )));
    }

    input.title = input.title.trim().to_string();
    if input.title.chars().count() > MAX_TITLE_CHARS {
        return Err(AppError::Validation(format!(
            "单集标题不能超过 {MAX_TITLE_CHARS} 个字符"
        )));
    }

    if let Some(aired_at) = input.aired_at
        && aired_at <= 0
    {
        return Err(AppError::Validation("播出时间无效".to_string()));
    }

    Ok(input)
}

fn normalize_watch_input(mut input: WatchRecordInput) -> Result<WatchRecordInput, AppError> {
    if !(1..=MAX_EPISODE_NUMBER).contains(&input.episode_number) {
        return Err(AppError::Validation(format!(
            "集数必须在 1 到 {MAX_EPISODE_NUMBER} 之间"
        )));
    }

    let watched_at = normalize_watched_at(input.watched_at)?;
    input.watched_at = watched_at;
    input.comment = normalize_comment(input.comment)?;

    Ok(input)
}

fn normalize_watched_at(watched_at: Option<i64>) -> Result<Option<i64>, AppError> {
    if let Some(value) = watched_at
        && value <= 0
    {
        return Err(AppError::Validation("观看时间无效".to_string()));
    }
    Ok(watched_at)
}

fn normalize_comment(comment: String) -> Result<String, AppError> {
    let comment = comment.trim().to_string();
    if comment.chars().count() > MAX_COMMENT_CHARS {
        return Err(AppError::Validation(format!(
            "感想不能超过 {MAX_COMMENT_CHARS} 个字符"
        )));
    }
    Ok(comment)
}

fn map_episode_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database_error) = &error
        && database_error.is_unique_violation()
    {
        return AppError::Validation("该集数已存在".to_string());
    }
    AppError::Database(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect, migrate};
    use crate::models::anime::AnimeInput;
    use crate::services::anime_service;

    use sqlx::SqlitePool;
    use tempfile::{TempDir, tempdir};

    async fn migrated_pool() -> (TempDir, SqlitePool) {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("episode-test.sqlite");
        let backups_dir = dir.path().join("backups");
        let pool = connect(&database_file).await.expect("connect");
        migrate::run(&pool, &backups_dir).await.expect("migrate");
        (dir, pool)
    }

    async fn create_anime(pool: &SqlitePool, total_episodes: Option<i64>) -> i64 {
        let input = AnimeInput {
            title: "测试番剧".to_string(),
            original_title: None,
            aliases: Vec::new(),
            summary: String::new(),
            year: None,
            season: None,
            total_episodes,
            watch_status: "watching".to_string(),
            rating: None,
            review: String::new(),
            tags: Vec::new(),
        };

        anime_service::create(pool, input)
            .await
            .expect("create anime")
            .id
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(pool)
            .await
            .expect("count")
    }

    #[tokio::test]
    async fn total_episodes_generate_episode_rows() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = create_anime(&pool, Some(3)).await;

        let episodes = list_episodes(&pool, anime_id).await.expect("episodes");

        assert_eq!(episodes.len(), 3);
        assert_eq!(
            episodes
                .iter()
                .map(|episode| episode.episode_number)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert!(episodes.iter().all(|episode| !episode.is_watched));
    }

    #[tokio::test]
    async fn toggle_watched_adds_and_removes_records() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = create_anime(&pool, Some(1)).await;
        let episode = list_episodes(&pool, anime_id)
            .await
            .expect("episodes")
            .remove(0);

        let watched = set_episode_watched(
            &pool,
            episode.id,
            true,
            Some(1_700_000_000_000),
            Some("很好看".to_string()),
        )
        .await
        .expect("watch");
        assert!(watched.is_watched);
        assert_eq!(watched.watch_count, 1);
        assert_eq!(watched.last_watched_at, Some(1_700_000_000_000));

        let anime = anime_service::get(&pool, anime_id).await.expect("anime");
        assert_eq!(anime.watched_episodes, 1);

        add_watch_record(
            &pool,
            anime_id,
            WatchRecordInput {
                episode_number: 1,
                watched_at: Some(1_700_000_100_000),
                comment: "二刷".to_string(),
            },
        )
        .await
        .expect("second record");

        let refreshed = list_episodes(&pool, anime_id)
            .await
            .expect("episodes")
            .remove(0);
        assert_eq!(refreshed.watch_count, 2);

        let unwatched = set_episode_watched(&pool, episode.id, false, None, None)
            .await
            .expect("unwatch");
        assert!(!unwatched.is_watched);
        assert_eq!(unwatched.watch_count, 0);
        assert_eq!(count(&pool, "watch_records").await, 0);
    }

    #[tokio::test]
    async fn backfill_without_episode_row_is_kept() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = create_anime(&pool, None).await;

        let record = add_watch_record(
            &pool,
            anime_id,
            WatchRecordInput {
                episode_number: 5,
                watched_at: Some(1_600_000_000_000),
                comment: "补录".to_string(),
            },
        )
        .await
        .expect("add record");

        assert!(record.episode_id.is_none());
        assert_eq!(record.episode_number, 5);

        let anime = anime_service::get(&pool, anime_id).await.expect("anime");
        assert_eq!(anime.watched_episodes, 1);

        let records = list_watch_records(&pool, anime_id).await.expect("records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].comment, "补录");
    }

    #[tokio::test]
    async fn episode_crud_and_duplicate_validation() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = create_anime(&pool, None).await;

        let episode = create_episode(
            &pool,
            anime_id,
            EpisodeInput {
                episode_number: 1,
                title: "第一集".to_string(),
                aired_at: Some(1_700_000_000_000),
            },
        )
        .await
        .expect("create episode");
        assert_eq!(episode.episode_number, 1);
        assert_eq!(episode.title, "第一集");

        let duplicate = create_episode(
            &pool,
            anime_id,
            EpisodeInput {
                episode_number: 1,
                title: "重复".to_string(),
                aired_at: None,
            },
        )
        .await;
        assert!(matches!(duplicate, Err(AppError::Validation(_))));

        let updated = update_episode(
            &pool,
            episode.id,
            EpisodeInput {
                episode_number: 2,
                title: "改名".to_string(),
                aired_at: None,
            },
        )
        .await
        .expect("update episode");
        assert_eq!(updated.episode_number, 2);
        assert_eq!(updated.title, "改名");

        let third = create_episode(
            &pool,
            anime_id,
            EpisodeInput {
                episode_number: 3,
                title: String::new(),
                aired_at: None,
            },
        )
        .await
        .expect("create third");

        delete_episode(&pool, third.id)
            .await
            .expect("delete episode");
        let remaining = list_episodes(&pool, anime_id).await.expect("episodes");
        assert_eq!(remaining.len(), 1);
    }

    #[tokio::test]
    async fn deleting_episode_keeps_history_and_cleans_relations() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = create_anime(&pool, Some(1)).await;
        let episode = list_episodes(&pool, anime_id)
            .await
            .expect("episodes")
            .remove(0);

        let record = add_watch_record(
            &pool,
            anime_id,
            WatchRecordInput {
                episode_number: 1,
                watched_at: Some(1_700_000_000_000),
                comment: String::new(),
            },
        )
        .await
        .expect("record");

        let note = sqlx::query(
            "INSERT INTO notes (title, content_md, created_at, updated_at)
             VALUES ('关联', '', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("note");
        let note_id = note.last_insert_rowid();

        for (target_type, target_id) in [("episode", episode.id), ("watch_record", record.id)] {
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

        delete_episode(&pool, episode.id)
            .await
            .expect("delete episode");

        assert_eq!(count(&pool, "watch_records").await, 1);
        let episode_id: Option<i64> =
            sqlx::query_scalar("SELECT episode_id FROM watch_records LIMIT 1")
                .fetch_one(&pool)
                .await
                .expect("record");
        assert!(episode_id.is_none());
        assert_eq!(count(&pool, "note_relations").await, 1);
        let remaining_type: String =
            sqlx::query_scalar("SELECT target_type FROM note_relations LIMIT 1")
                .fetch_one(&pool)
                .await
                .expect("relation");
        assert_eq!(remaining_type, "watch_record");
    }

    #[tokio::test]
    async fn deleting_watch_record_cleans_relations() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = create_anime(&pool, None).await;

        let record = add_watch_record(
            &pool,
            anime_id,
            WatchRecordInput {
                episode_number: 1,
                watched_at: None,
                comment: String::new(),
            },
        )
        .await
        .expect("record");

        let note = sqlx::query(
            "INSERT INTO notes (title, content_md, created_at, updated_at)
             VALUES ('关联', '', 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("note");
        let note_id = note.last_insert_rowid();

        sqlx::query(
            "INSERT INTO note_relations (note_id, target_type, target_id, created_at)
             VALUES (?, 'watch_record', ?, 1)",
        )
        .bind(note_id)
        .bind(record.id)
        .execute(&pool)
        .await
        .expect("relation");

        delete_watch_record(&pool, record.id)
            .await
            .expect("delete record");

        assert_eq!(count(&pool, "watch_records").await, 0);
        assert_eq!(count(&pool, "note_relations").await, 0);
    }

    #[tokio::test]
    async fn validation_rejects_invalid_episode_values() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = create_anime(&pool, None).await;

        assert!(matches!(
            create_episode(
                &pool,
                anime_id,
                EpisodeInput {
                    episode_number: 0,
                    title: String::new(),
                    aired_at: None,
                },
            )
            .await,
            Err(AppError::Validation(_))
        ));

        let long_title = "长".repeat(MAX_TITLE_CHARS + 1);
        assert!(matches!(
            create_episode(
                &pool,
                anime_id,
                EpisodeInput {
                    episode_number: 1,
                    title: long_title,
                    aired_at: None,
                },
            )
            .await,
            Err(AppError::Validation(_))
        ));

        assert!(matches!(
            add_watch_record(
                &pool,
                anime_id,
                WatchRecordInput {
                    episode_number: 1,
                    watched_at: Some(-1),
                    comment: String::new(),
                },
            )
            .await,
            Err(AppError::Validation(_))
        ));

        let long_comment = "长".repeat(MAX_COMMENT_CHARS + 1);
        assert!(matches!(
            add_watch_record(
                &pool,
                anime_id,
                WatchRecordInput {
                    episode_number: 1,
                    watched_at: None,
                    comment: long_comment,
                },
            )
            .await,
            Err(AppError::Validation(_))
        ));
    }
}
