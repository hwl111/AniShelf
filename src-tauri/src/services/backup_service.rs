use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::{Connection, SqlitePool};

use crate::db::{AppPaths, connect, migrate};
use crate::error::AppError;
use crate::models::backup::{AppInfo, BackupInfo, BackupSidecar, RestoreResult};
use crate::utils::now_ms;

const MAX_BACKUP_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
const PENDING_FILE: &str = "pending_restore.sqlite";

pub async fn list_backups(paths: &AppPaths) -> Result<Vec<BackupInfo>, AppError> {
    let mut backups = Vec::new();

    for entry in std::fs::read_dir(&paths.backups_dir).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("sqlite") {
            continue;
        }

        let metadata = entry.metadata().map_err(AppError::Io)?;
        if !metadata.is_file() {
            continue;
        }

        let file_name = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        let kind = backup_kind(&file_name);
        let sidecar = read_sidecar(&path);
        let created_at = sidecar
            .as_ref()
            .map(|value| value.created_at)
            .or_else(|| {
                metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_millis() as i64)
            })
            .unwrap_or(0);

        backups.push(BackupInfo {
            file_name,
            path: display_path(&path),
            kind,
            created_at,
            app_version: sidecar.as_ref().map(|value| value.app_version.clone()),
            schema_version: sidecar.as_ref().map(|value| value.schema_version),
            size_bytes: metadata.len(),
        });
    }

    backups.sort_by_key(|backup| std::cmp::Reverse(backup.created_at));
    Ok(backups)
}

pub async fn create_backup(
    pool: &SqlitePool,
    paths: &AppPaths,
    destination: Option<String>,
) -> Result<BackupInfo, AppError> {
    let schema_version = migrate::current_version(pool).await?;

    let (path, kind) = match destination {
        Some(value) => {
            let path = validate_destination(paths, &value)?;
            (path, "manual".to_string())
        }
        None => (
            unique_sqlite_path(&paths.backups_dir, "backup"),
            "manual".to_string(),
        ),
    };

    let parent = path
        .parent()
        .ok_or_else(|| AppError::Validation("备份目标路径无效".to_string()))?;

    let temp = unique_sqlite_path(parent, "backup_tmp");
    let escaped = temp.to_string_lossy().replace('\'', "''");
    let result = sqlx::raw_sql(sqlx::AssertSqlSafe(format!("VACUUM INTO '{escaped}'")))
        .execute(pool)
        .await;

    if let Err(error) = result {
        let _ = std::fs::remove_file(&temp);
        return Err(AppError::Database(error));
    }

    if let Err(error) = replace_file(&temp, &path) {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }

    write_sidecar(&path, schema_version)?;
    let metadata = std::fs::metadata(&path).map_err(AppError::Io)?;
    log::info!("已创建数据库备份：{}", path.display());

    Ok(BackupInfo {
        file_name: path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default(),
        path: display_path(&path),
        kind,
        created_at: now_ms(),
        app_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        schema_version: Some(schema_version),
        size_bytes: metadata.len(),
    })
}

pub async fn stage_restore(
    pool: &SqlitePool,
    paths: &AppPaths,
    source_path: &str,
) -> Result<RestoreResult, AppError> {
    let source = PathBuf::from(source_path);
    let canonical = std::fs::canonicalize(&source).map_err(AppError::Io)?;
    let schema_version = validate_backup_file(&canonical).await?;
    log::info!(
        "准备恢复数据库：{}（schema 版本 {}）",
        canonical.display(),
        schema_version
    );

    let pending = paths.database_file.with_file_name(PENDING_FILE);
    let database_canonical = std::fs::canonicalize(&paths.database_file).ok();
    if database_canonical.as_deref() == Some(canonical.as_path()) {
        return Err(AppError::Validation(
            "不能用当前数据库文件作为恢复来源".to_string(),
        ));
    }
    if let Ok(pending_canonical) = std::fs::canonicalize(&pending)
        && pending_canonical == canonical
    {
        return Err(AppError::Validation(
            "该文件已是待恢复的数据库文件".to_string(),
        ));
    }

    let pre_restore = migrate::backup_database(pool, &paths.backups_dir, "pre_restore").await?;
    let current_version = migrate::current_version(pool).await?;
    write_sidecar(&pre_restore, current_version)?;
    log::info!("恢复前已备份当前数据库：{}", pre_restore.display());

    if pending.exists() {
        std::fs::remove_file(&pending).map_err(AppError::Io)?;
    }

    let temp = unique_sqlite_path(
        pending
            .parent()
            .ok_or_else(|| AppError::Internal("待恢复目录无效".to_string()))?,
        "pending_restore_tmp",
    );
    std::fs::copy(&canonical, &temp).map_err(AppError::Io)?;
    std::fs::rename(&temp, &pending).map_err(|error| {
        let _ = std::fs::remove_file(&temp);
        AppError::Io(error)
    })?;

    log::info!("已暂存恢复文件，等待重启应用：{}", pending.display());

    Ok(RestoreResult {
        source_path: display_path(&canonical),
        pre_restore_backup: display_path(&pre_restore),
        restart_required: true,
    })
}

pub async fn cancel_pending_restore(paths: &AppPaths) -> Result<bool, AppError> {
    let pending = paths.database_file.with_file_name(PENDING_FILE);
    if !pending.exists() {
        return Ok(false);
    }

    std::fs::remove_file(&pending).map_err(AppError::Io)?;
    log::info!("已取消待恢复的数据库文件：{}", pending.display());
    Ok(true)
}

pub async fn apply_pending_restore(paths: &AppPaths) -> Result<Option<PathBuf>, AppError> {
    let pending = paths.database_file.with_file_name(PENDING_FILE);
    if !pending.exists() {
        return Ok(None);
    }

    if let Err(error) = validate_backup_file(&pending).await {
        let invalid = unique_sqlite_path(
            &paths.backups_dir,
            &format!("pending_restore_invalid_{}", now_ms()),
        );
        let _ = std::fs::rename(&pending, &invalid);
        log::error!(
            "待恢复数据库校验失败，已取消恢复并保留当前数据库：{error}；文件：{}",
            invalid.display()
        );
        return Ok(None);
    }

    if paths.database_file.exists() {
        let pool = match connect(&paths.database_file).await {
            Ok(pool) => pool,
            Err(error) => {
                log::error!("恢复前无法打开当前数据库，已取消恢复：{error}");
                return Ok(None);
            }
        };

        let backup_result =
            migrate::backup_database(&pool, &paths.backups_dir, "pre_restore").await;
        let backup_result = match backup_result {
            Ok(path) => {
                let current_version = migrate::current_version(&pool).await.unwrap_or(0);
                if let Err(error) = write_sidecar(&path, current_version) {
                    log::warn!("恢复前备份元数据写入失败：{error}");
                }
                log::info!("恢复前已备份当前数据库：{}", path.display());
                Ok(path)
            }
            Err(error) => Err(error),
        };
        pool.close().await;

        if let Err(error) = backup_result {
            log::error!("恢复前备份当前数据库失败，已取消恢复：{error}");
            return Ok(None);
        }
    }

    remove_sqlite_side_files(&paths.database_file);

    let old_path = unique_sqlite_path(
        &paths.backups_dir,
        &format!("pre_restore_replaced_{}", now_ms()),
    );

    if paths.database_file.exists() {
        std::fs::rename(&paths.database_file, &old_path).map_err(AppError::Io)?;
    }

    if let Err(error) = std::fs::rename(&pending, &paths.database_file) {
        if old_path.exists() {
            let _ = std::fs::rename(&old_path, &paths.database_file);
        }
        return Err(AppError::Io(error));
    }

    let _ = std::fs::remove_file(&old_path);
    log::info!("已应用暂存的数据库恢复：{}", paths.database_file.display());

    Ok(Some(paths.database_file.clone()))
}

pub async fn app_info(pool: &SqlitePool, paths: &AppPaths) -> Result<AppInfo, AppError> {
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        data_dir: display_path(&paths.data_dir),
        database_path: display_path(&paths.database_file),
        backups_dir: display_path(&paths.backups_dir),
        logs_dir: display_path(&paths.logs_dir),
        schema_version: migrate::current_version(pool).await?,
    })
}

async fn validate_backup_file(path: &Path) -> Result<i64, AppError> {
    let metadata = std::fs::metadata(path).map_err(AppError::Io)?;
    if !metadata.is_file() {
        return Err(AppError::Validation("备份路径不是文件".to_string()));
    }
    if metadata.len() == 0 {
        return Err(AppError::Validation("备份文件为空".to_string()));
    }
    if metadata.len() > MAX_BACKUP_BYTES {
        return Err(AppError::Validation("备份文件超过 2GB 上限".to_string()));
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_lowercase())
        .unwrap_or_default();
    if !matches!(extension.as_str(), "sqlite" | "sqlite3" | "db") {
        return Err(AppError::Validation(
            "备份文件扩展名必须是 .sqlite、.sqlite3 或 .db".to_string(),
        ));
    }

    let mut file = std::fs::File::open(path).map_err(AppError::Io)?;
    let mut header = [0u8; 16];
    file.read_exact(&mut header)
        .map_err(|_| AppError::Validation("备份文件不是有效的 SQLite 数据库".to_string()))?;
    if &header != SQLITE_HEADER {
        return Err(AppError::Validation(
            "备份文件不是有效的 SQLite 数据库".to_string(),
        ));
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .busy_timeout(Duration::from_secs(5));
    let mut connection = SqliteConnection::connect_with(&options).await?;

    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check;")
        .fetch_one(&mut connection)
        .await?;
    if integrity != "ok" {
        connection.close().await.ok();
        return Err(AppError::Validation(format!(
            "备份文件完整性检查失败：{integrity}"
        )));
    }

    let has_migrations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(&mut connection)
    .await?;
    if has_migrations == 0 {
        connection.close().await.ok();
        return Err(AppError::Validation(
            "该文件不是 AniShelf 数据库备份".to_string(),
        ));
    }

    let version: Option<i64> =
        sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1")
            .fetch_one(&mut connection)
            .await?;
    connection.close().await.ok();

    let version = version.unwrap_or(0);
    let latest = migrate::latest_version();
    if version > latest {
        return Err(AppError::Validation(format!(
            "备份数据库版本 {version} 高于当前应用支持的版本 {latest}，请使用更新版本的 AniShelf 恢复"
        )));
    }

    Ok(version)
}

fn validate_destination(paths: &AppPaths, value: &str) -> Result<PathBuf, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("备份目标路径不能为空".to_string()));
    }

    let path = PathBuf::from(trimmed);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_lowercase())
        .unwrap_or_default();
    if !matches!(extension.as_str(), "sqlite" | "sqlite3" | "db") {
        return Err(AppError::Validation(
            "备份文件扩展名必须是 .sqlite、.sqlite3 或 .db".to_string(),
        ));
    }

    let parent = path
        .parent()
        .ok_or_else(|| AppError::Validation("备份目标路径无效".to_string()))?;
    let parent_metadata = std::fs::metadata(parent).map_err(AppError::Io)?;
    if !parent_metadata.is_dir() {
        return Err(AppError::Validation("备份目标目录不存在".to_string()));
    }

    let canonical_parent = std::fs::canonicalize(parent).map_err(AppError::Io)?;
    let absolute = canonical_parent.join(
        path.file_name()
            .ok_or_else(|| AppError::Validation("备份目标路径无效".to_string()))?,
    );
    let database_canonical = std::fs::canonicalize(&paths.database_file).ok();
    if database_canonical.as_deref() == Some(absolute.as_path()) {
        return Err(AppError::Validation(
            "不能用备份覆盖当前正在使用的数据库".to_string(),
        ));
    }

    Ok(absolute)
}

fn unique_sqlite_path(dir: &Path, prefix: &str) -> PathBuf {
    let timestamp = now_ms();
    let mut path = dir.join(format!("{prefix}_{timestamp}.sqlite"));
    let mut counter = 1;
    while path.exists() {
        path = dir.join(format!("{prefix}_{timestamp}_{counter}.sqlite"));
        counter += 1;
    }
    path
}

fn sidecar_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.json", path.to_string_lossy()))
}

fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(stripped) => stripped.to_string(),
        None => text.into_owned(),
    }
}

fn backup_kind(file_name: &str) -> String {
    if file_name.starts_with("pre_migration_") {
        "pre_migration".to_string()
    } else if file_name.starts_with("pre_restore_") {
        "pre_restore".to_string()
    } else if file_name.starts_with("backup_") {
        "manual".to_string()
    } else {
        "other".to_string()
    }
}

fn read_sidecar(path: &Path) -> Option<BackupSidecar> {
    let text = std::fs::read_to_string(sidecar_path(path)).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_sidecar(path: &Path, schema_version: i64) -> Result<(), AppError> {
    let sidecar = BackupSidecar {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        created_at: now_ms(),
    };
    let text = serde_json::to_string_pretty(&sidecar)
        .map_err(|error| AppError::Internal(format!("备份元数据序列化失败：{error}")))?;
    std::fs::write(sidecar_path(path), text).map_err(AppError::Io)
}

fn replace_file(temp: &Path, destination: &Path) -> Result<(), AppError> {
    if !destination.exists() {
        return std::fs::rename(temp, destination).map_err(AppError::Io);
    }

    let old = destination.with_file_name(format!(
        "{}.old-{}",
        destination
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "backup".to_string()),
        now_ms()
    ));
    std::fs::rename(destination, &old).map_err(AppError::Io)?;

    match std::fs::rename(temp, destination) {
        Ok(()) => {
            let _ = std::fs::remove_file(&old);
            Ok(())
        }
        Err(error) => {
            let _ = std::fs::rename(&old, destination);
            Err(AppError::Io(error))
        }
    }
}

fn remove_sqlite_side_files(database_file: &Path) {
    for suffix in ["-wal", "-shm"] {
        let path = PathBuf::from(format!("{}{suffix}", database_file.to_string_lossy()));
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect, migrate};

    use tempfile::{TempDir, tempdir};

    async fn setup() -> (TempDir, SqlitePool, AppPaths) {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("database").join("anishelf.sqlite");
        std::fs::create_dir_all(database_file.parent().expect("parent")).expect("database dir");
        let backups_dir = dir.path().join("backups");
        std::fs::create_dir_all(&backups_dir).expect("backups dir");

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

    async fn insert_note(pool: &SqlitePool, title: &str) -> i64 {
        sqlx::query(
            "INSERT INTO notes (title, content_md, created_at, updated_at) VALUES (?, '', 1, 1)",
        )
        .bind(title)
        .execute(pool)
        .await
        .expect("note")
        .last_insert_rowid()
    }

    async fn note_titles(pool: &SqlitePool) -> Vec<String> {
        sqlx::query_scalar("SELECT title FROM notes ORDER BY id")
            .fetch_all(pool)
            .await
            .expect("titles")
    }

    #[tokio::test]
    async fn create_backup_writes_database_metadata_and_list_entry() {
        let (_dir, pool, paths) = setup().await;
        insert_note(&pool, "备份笔记").await;

        let info = create_backup(&pool, &paths, None).await.expect("backup");
        assert!(Path::new(&info.path).is_file());
        assert_eq!(info.kind, "manual");
        assert!(info.size_bytes > 0);
        assert_eq!(info.app_version.as_deref(), Some(env!("CARGO_PKG_VERSION")));
        assert!(info.schema_version.unwrap_or(0) > 0);
        assert!(Path::new(&format!("{}.json", info.path)).is_file());

        let backups = list_backups(&paths).await.expect("list");
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].file_name, info.file_name);
    }

    #[tokio::test]
    async fn create_backup_supports_custom_destination() {
        let (dir, pool, paths) = setup().await;
        let destination = dir.path().join("manual-backup.sqlite");

        let info = create_backup(
            &pool,
            &paths,
            Some(destination.to_string_lossy().into_owned()),
        )
        .await
        .expect("backup");

        assert_eq!(Path::new(&info.path), destination);
        assert!(destination.is_file());
    }

    #[tokio::test]
    async fn staged_restore_replaces_database_after_restart() {
        let (_dir, pool, paths) = setup().await;
        insert_note(&pool, "恢复前数据").await;
        let backup = create_backup(&pool, &paths, None).await.expect("backup");

        insert_note(&pool, "恢复后应消失").await;
        assert_eq!(note_titles(&pool).await.len(), 2);

        let staged = stage_restore(&pool, &paths, &backup.path)
            .await
            .expect("stage");
        assert!(staged.restart_required);
        assert!(Path::new(&staged.pre_restore_backup).is_file());
        assert!(paths.database_file.with_file_name(PENDING_FILE).is_file());

        pool.close().await;
        let applied = apply_pending_restore(&paths).await.expect("apply");
        assert!(applied.is_some());

        let reopened = connect(&paths.database_file).await.expect("reopen");
        let titles = note_titles(&reopened).await;
        assert_eq!(titles, vec!["恢复前数据".to_string()]);
        reopened.close().await;
    }

    #[tokio::test]
    async fn invalid_restore_file_is_rejected_without_touching_database() {
        let (dir, pool, paths) = setup().await;
        insert_note(&pool, "安全数据").await;

        let bad = dir.path().join("not-a-database.sqlite");
        std::fs::write(&bad, "not sqlite").expect("write");

        let result = stage_restore(&pool, &paths, &bad.to_string_lossy()).await;
        assert!(matches!(result, Err(AppError::Validation(_))));
        assert!(!paths.database_file.with_file_name(PENDING_FILE).exists());
        assert_eq!(note_titles(&pool).await, vec!["安全数据".to_string()]);
    }

    #[tokio::test]
    async fn cancelling_pending_restore_keeps_current_database() {
        let (_dir, pool, paths) = setup().await;
        insert_note(&pool, "当前数据").await;
        let backup = create_backup(&pool, &paths, None).await.expect("backup");
        stage_restore(&pool, &paths, &backup.path)
            .await
            .expect("stage");

        let pending = paths.database_file.with_file_name(PENDING_FILE);
        assert!(pending.exists());

        let cancelled = cancel_pending_restore(&paths).await.expect("cancel");
        assert!(cancelled);
        assert!(!pending.exists());
        assert_eq!(note_titles(&pool).await, vec!["当前数据".to_string()]);
    }

    #[tokio::test]
    async fn pending_restore_with_corrupt_file_is_cancelled_on_startup() {
        let (_dir, pool, paths) = setup().await;
        insert_note(&pool, "保留数据").await;

        let pending = paths.database_file.with_file_name(PENDING_FILE);
        std::fs::write(&pending, "corrupt").expect("pending");

        pool.close().await;
        let applied = apply_pending_restore(&paths).await.expect("apply");
        assert!(applied.is_none());

        let reopened = connect(&paths.database_file).await.expect("reopen");
        assert_eq!(note_titles(&reopened).await, vec!["保留数据".to_string()]);
        reopened.close().await;
    }

    #[tokio::test]
    async fn newer_schema_backup_is_rejected() {
        let (_dir, pool, paths) = setup().await;
        let backup = create_backup(&pool, &paths, None).await.expect("backup");

        let options = SqliteConnectOptions::new()
            .filename(&backup.path)
            .create_if_missing(false);
        let mut connection = SqliteConnection::connect_with(&options)
            .await
            .expect("open backup");
        sqlx::query(
            "UPDATE _sqlx_migrations SET version = 999999
             WHERE version = (SELECT MAX(version) FROM _sqlx_migrations)",
        )
        .execute(&mut connection)
        .await
        .expect("bump version");
        connection.close().await.expect("close");

        let result = stage_restore(&pool, &paths, &backup.path).await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn app_info_reports_paths_and_schema_version() {
        let (_dir, pool, paths) = setup().await;
        let info = app_info(&pool, &paths).await.expect("info");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.schema_version > 0);
        assert!(info.database_path.ends_with("anishelf.sqlite"));
    }
}
