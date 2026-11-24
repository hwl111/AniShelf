use sqlx::query_as;
use sqlx::{SqliteConnection, SqlitePool};

use crate::models::episode::{Episode, EpisodeInput, WatchRecord};

const EPISODE_SELECT: &str = "SELECT e.id, e.anime_id, e.episode_number, e.title, e.aired_at,
    EXISTS(SELECT 1 FROM watch_records w WHERE w.episode_id = e.id) AS is_watched,
    (SELECT MAX(w.watched_at) FROM watch_records w WHERE w.episode_id = e.id) AS last_watched_at,
    (SELECT COUNT(*) FROM watch_records w WHERE w.episode_id = e.id) AS watch_count
    FROM anime_episodes e";

pub async fn list(pool: &SqlitePool, anime_id: i64) -> Result<Vec<Episode>, sqlx::Error> {
    let sql = format!("{EPISODE_SELECT} WHERE e.anime_id = ? ORDER BY e.episode_number");
    query_as::<_, Episode>(sqlx::AssertSqlSafe(sql))
        .bind(anime_id)
        .fetch_all(pool)
        .await
}

pub async fn get(
    connection: &mut SqliteConnection,
    episode_id: i64,
) -> Result<Option<Episode>, sqlx::Error> {
    let sql = format!("{EPISODE_SELECT} WHERE e.id = ?");
    query_as::<_, Episode>(sqlx::AssertSqlSafe(sql))
        .bind(episode_id)
        .fetch_optional(&mut *connection)
        .await
}

pub async fn insert(
    connection: &mut SqliteConnection,
    anime_id: i64,
    input: &EpisodeInput,
    now: i64,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO anime_episodes (anime_id, episode_number, title, aired_at, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(anime_id)
    .bind(input.episode_number)
    .bind(&input.title)
    .bind(input.aired_at)
    .bind(now)
    .bind(now)
    .execute(&mut *connection)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update(
    connection: &mut SqliteConnection,
    episode_id: i64,
    input: &EpisodeInput,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE anime_episodes
         SET episode_number = ?, title = ?, aired_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(input.episode_number)
    .bind(&input.title)
    .bind(input.aired_at)
    .bind(now)
    .bind(episode_id)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn delete(connection: &mut SqliteConnection, episode_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM anime_episodes WHERE id = ?")
        .bind(episode_id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn cleanup_note_relations(
    connection: &mut SqliteConnection,
    episode_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM note_relations WHERE target_type = 'episode' AND target_id = ?")
        .bind(episode_id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn ensure_episodes(
    connection: &mut SqliteConnection,
    anime_id: i64,
    total_episodes: Option<i64>,
    now: i64,
) -> Result<(), sqlx::Error> {
    let Some(total) = total_episodes else {
        return Ok(());
    };
    if total <= 0 {
        return Ok(());
    }

    sqlx::query(
        "WITH RECURSIVE seq(n) AS (
             SELECT 1
             UNION ALL
             SELECT n + 1 FROM seq WHERE n < ?
         )
         INSERT OR IGNORE INTO anime_episodes
             (anime_id, episode_number, created_at, updated_at)
         SELECT ?, n, ?, ? FROM seq",
    )
    .bind(total)
    .bind(anime_id)
    .bind(now)
    .bind(now)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn find_episode_id(
    connection: &mut SqliteConnection,
    anime_id: i64,
    episode_number: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM anime_episodes WHERE anime_id = ? AND episode_number = ?")
        .bind(anime_id)
        .bind(episode_number)
        .fetch_optional(&mut *connection)
        .await
}

pub async fn insert_watch_record(
    connection: &mut SqliteConnection,
    anime_id: i64,
    episode_id: Option<i64>,
    episode_number: i64,
    watched_at: i64,
    comment: &str,
    now: i64,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO watch_records
         (anime_id, episode_id, episode_number, watched_at, comment, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(anime_id)
    .bind(episode_id)
    .bind(episode_number)
    .bind(watched_at)
    .bind(comment)
    .bind(now)
    .execute(&mut *connection)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn delete_watch_records_for_episode(
    connection: &mut SqliteConnection,
    episode_id: i64,
) -> Result<(), sqlx::Error> {
    cleanup_watch_relations_for_episode(connection, episode_id).await?;
    sqlx::query("DELETE FROM watch_records WHERE episode_id = ?")
        .bind(episode_id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

async fn cleanup_watch_relations_for_episode(
    connection: &mut SqliteConnection,
    episode_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM note_relations
         WHERE target_type = 'watch_record'
           AND target_id IN (SELECT id FROM watch_records WHERE episode_id = ?)",
    )
    .bind(episode_id)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn list_watch_records(
    pool: &SqlitePool,
    anime_id: i64,
) -> Result<Vec<WatchRecord>, sqlx::Error> {
    query_as::<_, WatchRecord>(
        "SELECT id, anime_id, episode_id, episode_number, watched_at, comment, created_at
         FROM watch_records
         WHERE anime_id = ?
         ORDER BY watched_at DESC, id DESC",
    )
    .bind(anime_id)
    .fetch_all(pool)
    .await
}

pub async fn get_watch_record(
    connection: &mut SqliteConnection,
    record_id: i64,
) -> Result<Option<WatchRecord>, sqlx::Error> {
    query_as::<_, WatchRecord>(
        "SELECT id, anime_id, episode_id, episode_number, watched_at, comment, created_at
         FROM watch_records WHERE id = ?",
    )
    .bind(record_id)
    .fetch_optional(&mut *connection)
    .await
}

pub async fn update_watch_record(
    connection: &mut SqliteConnection,
    record_id: i64,
    watched_at: i64,
    comment: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE watch_records SET watched_at = ?, comment = ? WHERE id = ?")
        .bind(watched_at)
        .bind(comment)
        .bind(record_id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn delete_watch_record(
    connection: &mut SqliteConnection,
    record_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM note_relations WHERE target_type = 'watch_record' AND target_id = ?")
        .bind(record_id)
        .execute(&mut *connection)
        .await?;
    sqlx::query("DELETE FROM watch_records WHERE id = ?")
        .bind(record_id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn anime_exists(pool: &SqlitePool, anime_id: i64) -> Result<bool, sqlx::Error> {
    let exists: i64 = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM anime WHERE id = ?)")
        .bind(anime_id)
        .fetch_one(pool)
        .await?;
    Ok(exists != 0)
}
