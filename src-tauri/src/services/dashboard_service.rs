use sqlx::{FromRow, SqlitePool};

use crate::db::AppPaths;
use crate::error::AppError;
use crate::models::dashboard::{
    ActivityItem, DashboardAnime, DashboardBook, DashboardData, DashboardNote, DashboardStats,
    DashboardWatchRecord,
};

const SECTION_LIMIT: i64 = 5;
const TIMELINE_SOURCE_LIMIT: i64 = 10;
const TIMELINE_LIMIT: usize = 20;

#[derive(Debug, FromRow)]
struct RecentBookRow {
    id: i64,
    title: String,
    author: String,
    format: String,
    cover_path: Option<String>,
    percent: f64,
    updated_at: i64,
}

#[derive(Debug, FromRow)]
struct StatsRow {
    anime_total: i64,
    anime_watching: i64,
    anime_completed: i64,
    anime_favorite: i64,
    books_total: i64,
    books_reading: i64,
    books_favorite: i64,
    notes_total: i64,
    notes_favorite: i64,
    watched_episodes: i64,
    watch_records_total: i64,
    reading_sessions: i64,
    reading_ms: i64,
}

pub async fn get(pool: &SqlitePool, paths: &AppPaths) -> Result<DashboardData, AppError> {
    let watching_anime: Vec<(i64, String, Option<i64>, i64)> = sqlx::query_as(
        "SELECT a.id, a.title, a.total_episodes,
                (SELECT COUNT(DISTINCT w.episode_number)
                 FROM watch_records w WHERE w.anime_id = a.id) AS watched_episodes
         FROM anime a
         WHERE a.watch_status = 'watching'
         ORDER BY a.updated_at DESC, a.id DESC
         LIMIT ?",
    )
    .bind(SECTION_LIMIT)
    .fetch_all(pool)
    .await?;

    let recent_watch_records: Vec<(i64, i64, String, i64, i64, String)> = sqlx::query_as(
        "SELECT w.id, w.anime_id, a.title, w.episode_number, w.watched_at, w.comment
         FROM watch_records w
         JOIN anime a ON a.id = w.anime_id
         ORDER BY w.watched_at DESC, w.id DESC
         LIMIT ?",
    )
    .bind(SECTION_LIMIT)
    .fetch_all(pool)
    .await?;

    let recent_books: Vec<RecentBookRow> = sqlx::query_as(
        "SELECT b.id, b.title, b.author, b.format, b.cover_path, p.percent,
                p.updated_at AS updated_at
         FROM reading_progress p
         JOIN books b ON b.id = p.book_id
         ORDER BY p.updated_at DESC, b.id DESC
         LIMIT ?",
    )
    .bind(SECTION_LIMIT)
    .fetch_all(pool)
    .await?;

    let recent_notes: Vec<(i64, String, i64)> = sqlx::query_as(
        "SELECT id, COALESCE(NULLIF(title, ''), '无标题笔记'), updated_at
         FROM notes
         ORDER BY updated_at DESC, id DESC
         LIMIT ?",
    )
    .bind(SECTION_LIMIT)
    .fetch_all(pool)
    .await?;

    let stats_row: StatsRow = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM anime) AS anime_total,
            (SELECT COUNT(*) FROM anime WHERE watch_status = 'watching') AS anime_watching,
            (SELECT COUNT(*) FROM anime WHERE watch_status = 'completed') AS anime_completed,
            (SELECT COUNT(*) FROM favorites WHERE entity_type = 'anime') AS anime_favorite,
            (SELECT COUNT(*) FROM books) AS books_total,
            (SELECT COUNT(*) FROM reading_progress) AS books_reading,
            (SELECT COUNT(*) FROM favorites WHERE entity_type = 'book') AS books_favorite,
            (SELECT COUNT(*) FROM notes) AS notes_total,
            (SELECT COUNT(*) FROM favorites WHERE entity_type = 'note') AS notes_favorite,
            (SELECT COUNT(DISTINCT anime_id || '-' || episode_number)
             FROM watch_records) AS watched_episodes,
            (SELECT COUNT(*) FROM watch_records) AS watch_records_total,
            (SELECT COUNT(*) FROM reading_sessions WHERE ended_at IS NOT NULL)
                AS reading_sessions,
            (SELECT COALESCE(SUM(duration_ms), 0) FROM reading_sessions
             WHERE ended_at IS NOT NULL) AS reading_ms",
    )
    .fetch_one(pool)
    .await?;

    let timeline = build_timeline(pool).await?;

    Ok(DashboardData {
        watching_anime: watching_anime
            .into_iter()
            .map(
                |(id, title, total_episodes, watched_episodes)| DashboardAnime {
                    id,
                    title,
                    total_episodes,
                    watched_episodes,
                },
            )
            .collect(),
        recent_watch_records: recent_watch_records
            .into_iter()
            .map(
                |(id, anime_id, anime_title, episode_number, watched_at, comment)| {
                    DashboardWatchRecord {
                        id,
                        anime_id,
                        anime_title,
                        episode_number,
                        watched_at,
                        comment,
                    }
                },
            )
            .collect(),
        recent_books: recent_books
            .into_iter()
            .map(|row| DashboardBook {
                id: row.id,
                title: row.title,
                author: row.author,
                format: row.format,
                cover_url: row
                    .cover_path
                    .map(|relative| paths.data_dir.join(relative).to_string_lossy().into_owned()),
                percent: row.percent,
                last_read_at: row.updated_at,
            })
            .collect(),
        recent_notes: recent_notes
            .into_iter()
            .map(|(id, title, updated_at)| DashboardNote {
                id,
                title,
                updated_at,
            })
            .collect(),
        stats: DashboardStats {
            anime_total: stats_row.anime_total,
            anime_watching: stats_row.anime_watching,
            anime_completed: stats_row.anime_completed,
            anime_favorite: stats_row.anime_favorite,
            books_total: stats_row.books_total,
            books_reading: stats_row.books_reading,
            books_favorite: stats_row.books_favorite,
            notes_total: stats_row.notes_total,
            notes_favorite: stats_row.notes_favorite,
            watched_episodes: stats_row.watched_episodes,
            watch_records_total: stats_row.watch_records_total,
            reading_sessions: stats_row.reading_sessions,
            reading_ms: stats_row.reading_ms,
        },
        timeline,
    })
}

async fn build_timeline(pool: &SqlitePool) -> Result<Vec<ActivityItem>, AppError> {
    let mut events = Vec::new();

    let watch_events: Vec<(i64, String, i64, i64)> = sqlx::query_as(
        "SELECT w.anime_id, a.title, w.episode_number, w.watched_at
         FROM watch_records w
         JOIN anime a ON a.id = w.anime_id
         ORDER BY w.watched_at DESC, w.id DESC
         LIMIT ?",
    )
    .bind(TIMELINE_SOURCE_LIMIT)
    .fetch_all(pool)
    .await?;
    events.extend(watch_events.into_iter().map(
        |(anime_id, title, episode_number, occurred_at)| ActivityItem {
            kind: "watch".to_string(),
            entity_id: anime_id,
            title,
            subtitle: format!("观看了第 {episode_number} 集"),
            occurred_at,
        },
    ));

    let reading_events: Vec<(i64, String, f64, i64)> = sqlx::query_as(
        "SELECT b.id, b.title, p.percent, p.updated_at
         FROM reading_progress p
         JOIN books b ON b.id = p.book_id
         ORDER BY p.updated_at DESC, b.id DESC
         LIMIT ?",
    )
    .bind(TIMELINE_SOURCE_LIMIT)
    .fetch_all(pool)
    .await?;
    events.extend(
        reading_events
            .into_iter()
            .map(|(book_id, title, percent, occurred_at)| ActivityItem {
                kind: "read".to_string(),
                entity_id: book_id,
                title,
                subtitle: format!("阅读进度 {:.1}%", percent * 100.0),
                occurred_at,
            }),
    );

    let note_events: Vec<(i64, String, i64, i64)> = sqlx::query_as(
        "SELECT id, COALESCE(NULLIF(title, ''), '无标题笔记'), created_at, updated_at
         FROM notes
         ORDER BY updated_at DESC, id DESC
         LIMIT ?",
    )
    .bind(TIMELINE_SOURCE_LIMIT)
    .fetch_all(pool)
    .await?;
    events.extend(
        note_events
            .into_iter()
            .map(|(note_id, title, created_at, occurred_at)| ActivityItem {
                kind: "note".to_string(),
                entity_id: note_id,
                title,
                subtitle: if created_at == occurred_at {
                    "创建了笔记".to_string()
                } else {
                    "编辑了笔记".to_string()
                },
                occurred_at,
            }),
    );

    let book_events: Vec<(i64, String, i64)> = sqlx::query_as(
        "SELECT id, title, imported_at FROM books ORDER BY imported_at DESC, id DESC LIMIT ?",
    )
    .bind(TIMELINE_SOURCE_LIMIT)
    .fetch_all(pool)
    .await?;
    events.extend(
        book_events
            .into_iter()
            .map(|(book_id, title, occurred_at)| ActivityItem {
                kind: "book".to_string(),
                entity_id: book_id,
                title,
                subtitle: "导入了书籍".to_string(),
                occurred_at,
            }),
    );

    let anime_events: Vec<(i64, String, i64)> = sqlx::query_as(
        "SELECT id, title, created_at FROM anime ORDER BY created_at DESC, id DESC LIMIT ?",
    )
    .bind(TIMELINE_SOURCE_LIMIT)
    .fetch_all(pool)
    .await?;
    events.extend(
        anime_events
            .into_iter()
            .map(|(anime_id, title, occurred_at)| ActivityItem {
                kind: "anime".to_string(),
                entity_id: anime_id,
                title,
                subtitle: "添加了番剧".to_string(),
                occurred_at,
            }),
    );

    events.sort_by(|left, right| {
        right
            .occurred_at
            .cmp(&left.occurred_at)
            .then_with(|| left.kind.cmp(&right.kind))
    });
    events.truncate(TIMELINE_LIMIT);

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect, migrate};

    use tempfile::{TempDir, tempdir};

    async fn setup() -> (TempDir, SqlitePool, AppPaths) {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("dashboard-test.sqlite");
        let backups_dir = dir.path().join("backups");
        let pool = connect(&database_file).await.expect("connect");
        migrate::run(&pool, &backups_dir).await.expect("migrate");
        let paths = AppPaths {
            data_dir: dir.path().to_path_buf(),
            database_file: database_file.clone(),
            covers_dir: dir.path().join("covers"),
            attachments_dir: dir.path().join("attachments"),
            cache_dir: dir.path().join("cache"),
            backups_dir,
            logs_dir: dir.path().join("logs"),
        };
        (dir, pool, paths)
    }

    #[tokio::test]
    async fn empty_dashboard_has_zero_stats_and_empty_sections() {
        let (_dir, pool, paths) = setup().await;

        let dashboard = get(&pool, &paths).await.expect("dashboard");
        assert!(dashboard.watching_anime.is_empty());
        assert!(dashboard.recent_watch_records.is_empty());
        assert!(dashboard.recent_books.is_empty());
        assert!(dashboard.recent_notes.is_empty());
        assert!(dashboard.timeline.is_empty());
        assert_eq!(dashboard.stats.anime_total, 0);
        assert_eq!(dashboard.stats.books_total, 0);
        assert_eq!(dashboard.stats.notes_total, 0);
        assert_eq!(dashboard.stats.reading_ms, 0);
    }

    #[tokio::test]
    async fn dashboard_reports_real_sections_stats_and_timeline() {
        let (_dir, pool, paths) = setup().await;

        let anime_id = sqlx::query(
            "INSERT INTO anime (title, watch_status, created_at, updated_at)
             VALUES ('在看番剧', 'watching', 10, 20)",
        )
        .execute(&pool)
        .await
        .expect("anime")
        .last_insert_rowid();

        sqlx::query(
            "INSERT INTO watch_records
             (anime_id, episode_number, watched_at, comment, created_at)
             VALUES (?, 3, 30, '不错', 30)",
        )
        .bind(anime_id)
        .execute(&pool)
        .await
        .expect("watch record");

        let book_id = sqlx::query(
            "INSERT INTO books
             (title, author, format, file_path, normalized_path, file_size, imported_at,
              created_at, updated_at)
             VALUES ('在读的书', '作者', 'epub', '/library/read.epub', '/library/read.epub',
                     10, 40, 40, 40)",
        )
        .execute(&pool)
        .await
        .expect("book")
        .last_insert_rowid();

        sqlx::query(
            "INSERT INTO reading_progress
             (book_id, location_type, location_json, percent, updated_at)
             VALUES (?, 'epub_cfi', '{}', 0.42, 50)",
        )
        .bind(book_id)
        .execute(&pool)
        .await
        .expect("progress");

        sqlx::query(
            "INSERT INTO reading_sessions
             (book_id, started_at, ended_at, duration_ms)
             VALUES (?, 1, 2, 60_000)",
        )
        .bind(book_id)
        .execute(&pool)
        .await
        .expect("session");

        let note_id = sqlx::query(
            "INSERT INTO notes (title, content_md, created_at, updated_at)
             VALUES ('最近笔记', '', 60, 60)",
        )
        .execute(&pool)
        .await
        .expect("note")
        .last_insert_rowid();

        for (entity_type, entity_id) in [("anime", anime_id), ("book", book_id), ("note", note_id)]
        {
            sqlx::query(
                "INSERT INTO favorites (entity_type, entity_id, created_at) VALUES (?, ?, 1)",
            )
            .bind(entity_type)
            .bind(entity_id)
            .execute(&pool)
            .await
            .expect("favorite");
        }

        let dashboard = get(&pool, &paths).await.expect("dashboard");

        assert_eq!(dashboard.watching_anime.len(), 1);
        assert_eq!(dashboard.watching_anime[0].watched_episodes, 1);
        assert_eq!(dashboard.recent_watch_records.len(), 1);
        assert_eq!(dashboard.recent_watch_records[0].episode_number, 3);
        assert_eq!(dashboard.recent_books.len(), 1);
        assert!((dashboard.recent_books[0].percent - 0.42).abs() < f64::EPSILON);
        assert_eq!(dashboard.recent_notes.len(), 1);
        assert_eq!(dashboard.stats.anime_watching, 1);
        assert_eq!(dashboard.stats.watched_episodes, 1);
        assert_eq!(dashboard.stats.watch_records_total, 1);
        assert_eq!(dashboard.stats.reading_sessions, 1);
        assert_eq!(dashboard.stats.reading_ms, 60_000);
        assert_eq!(dashboard.stats.anime_favorite, 1);
        assert_eq!(dashboard.stats.books_favorite, 1);
        assert_eq!(dashboard.stats.notes_favorite, 1);

        assert!(dashboard.timeline.iter().any(|item| item.kind == "watch"));
        assert!(dashboard.timeline.iter().any(|item| item.kind == "read"));
        assert!(dashboard.timeline.iter().any(|item| item.kind == "note"));
        assert!(dashboard.timeline.iter().any(|item| item.kind == "book"));
        assert!(dashboard.timeline.iter().any(|item| item.kind == "anime"));
        assert!(dashboard.timeline.len() <= TIMELINE_LIMIT);
        assert_eq!(dashboard.timeline[0].kind, "note");
    }
}
