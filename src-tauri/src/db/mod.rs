use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use tauri::{AppHandle, Manager};

use crate::error::AppError;

pub mod migrate;

/// Directories created on startup. Import, reader and backup phases consume
/// some of these fields later, so not every path is read yet.
#[allow(dead_code)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub database_file: PathBuf,
    pub covers_dir: PathBuf,
    pub attachments_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppPaths {
    pub fn resolve(app: &AppHandle) -> Result<Self, AppError> {
        let data_dir = app
            .path()
            .app_local_data_dir()
            .map_err(|error| AppError::Internal(format!("无法获取应用数据目录：{error}")))?;

        let database_dir = data_dir.join("database");
        let covers_dir = data_dir.join("covers");
        let attachments_dir = data_dir.join("attachments");
        let cache_dir = data_dir.join("cache");
        let backups_dir = data_dir.join("backups");
        let logs_dir = data_dir.join("logs");

        for dir in [
            &data_dir,
            &database_dir,
            &covers_dir,
            &attachments_dir,
            &cache_dir,
            &backups_dir,
            &logs_dir,
        ] {
            std::fs::create_dir_all(dir)?;
        }

        Ok(Self {
            data_dir,
            database_file: database_dir.join("anishelf.sqlite"),
            covers_dir,
            attachments_dir,
            cache_dir,
            backups_dir,
            logs_dir,
        })
    }
}

pub struct AppState {
    pub pool: SqlitePool,
    pub paths: AppPaths,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatus {
    pub path: String,
    pub journal_mode: String,
    pub foreign_keys: bool,
    pub migration_count: i64,
    pub size_bytes: u64,
}

pub async fn init(app: &AppHandle) -> Result<AppState, AppError> {
    let paths = AppPaths::resolve(app)?;
    let pool = connect(&paths.database_file).await?;

    let report = migrate::run(&pool, &paths.backups_dir).await?;
    if report.applied > 0 {
        eprintln!(
            "[AniShelf] 已应用 {} 个数据库迁移，当前版本 {}，迁移前备份：{:?}",
            report.applied, report.current_version, report.backup
        );
    }

    Ok(AppState { pool, paths })
}

pub async fn connect(database_file: &Path) -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::new()
        .filename(database_file)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn status(pool: &SqlitePool, database_file: &Path) -> Result<DatabaseStatus, AppError> {
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode;")
        .fetch_one(pool)
        .await?;
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys;")
        .fetch_one(pool)
        .await?;

    let has_migrations_table: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?;

    let migration_count: i64 = if has_migrations_table > 0 {
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = 1")
            .fetch_one(pool)
            .await?
    } else {
        0
    };

    let size_bytes = std::fs::metadata(database_file)
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    Ok(DatabaseStatus {
        path: database_file.to_string_lossy().into_owned(),
        journal_mode,
        foreign_keys: foreign_keys != 0,
        migration_count,
        size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn connect_applies_pragmas_and_reports_status() {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("anishelf.sqlite");

        let pool = connect(&database_file).await.expect("connect");
        let status = status(&pool, &database_file).await.expect("status");

        assert_eq!(status.journal_mode, "wal");
        assert!(status.foreign_keys);
        assert_eq!(status.migration_count, 0);
        assert!(status.size_bytes > 0);
    }

    #[tokio::test]
    async fn foreign_keys_are_enforced() {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("fk.sqlite");
        let pool = connect(&database_file).await.expect("connect");

        sqlx::query("CREATE TABLE parent (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .expect("create parent");
        sqlx::query(
            "CREATE TABLE child (
                id INTEGER PRIMARY KEY,
                parent_id INTEGER NOT NULL REFERENCES parent(id)
            )",
        )
        .execute(&pool)
        .await
        .expect("create child");

        let result = sqlx::query("INSERT INTO child (id, parent_id) VALUES (1, 999)")
            .execute(&pool)
            .await;

        assert!(result.is_err(), "foreign key violation must be rejected");
    }
}
