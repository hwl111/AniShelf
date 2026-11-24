use sqlx::query_as;
use sqlx::{QueryBuilder, Sqlite, SqliteConnection, SqlitePool};

use crate::models::book::{BookFilter, BookInsert, BookRelocation, BookRow, BookUpdateInput};
use crate::models::tag::Tag;

const BOOK_SELECT: &str = "SELECT b.id, b.title, b.author, b.format, b.file_path,
    b.source_path, b.file_size, b.file_mtime, b.cover_path, b.description,
    b.category, b.imported_at, b.last_opened_at, b.created_at, b.updated_at
    FROM books b";

pub async fn list(pool: &SqlitePool, filter: &BookFilter) -> Result<Vec<BookRow>, sqlx::Error> {
    let mut query = QueryBuilder::<Sqlite>::new(BOOK_SELECT);
    query.push(" WHERE 1 = 1");

    if let Some(format) = filter.format.as_deref() {
        query.push(" AND b.format = ").push_bind(format);
    }
    if let Some(category) = filter.category.as_deref() {
        query.push(" AND b.category = ").push_bind(category);
    }
    if let Some(tag_id) = filter.tag_id {
        query
            .push(
                " AND b.id IN (SELECT entity_id FROM entity_tags
                           WHERE entity_type = 'book' AND tag_id = ",
            )
            .push_bind(tag_id)
            .push(")");
    }
    match filter.favorite {
        Some(true) => {
            query.push(" AND b.id IN (SELECT entity_id FROM favorites WHERE entity_type = 'book')");
        }
        Some(false) => {
            query.push(
                " AND b.id NOT IN (SELECT entity_id FROM favorites WHERE entity_type = 'book')",
            );
        }
        None => {}
    }

    match filter.sort.as_deref().unwrap_or("updated_desc") {
        "imported_desc" => {
            query.push(" ORDER BY b.imported_at DESC");
        }
        "title_asc" => {
            query.push(" ORDER BY b.title COLLATE NOCASE ASC");
        }
        "size_desc" => {
            query.push(" ORDER BY b.file_size DESC");
        }
        _ => {
            query.push(" ORDER BY b.updated_at DESC");
        }
    }

    query.build_query_as::<BookRow>().fetch_all(pool).await
}

pub async fn get_row(
    connection: &mut SqliteConnection,
    id: i64,
) -> Result<Option<BookRow>, sqlx::Error> {
    let sql = format!("{BOOK_SELECT} WHERE b.id = ?");
    query_as::<_, BookRow>(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(&mut *connection)
        .await
}

pub async fn insert(
    connection: &mut SqliteConnection,
    book: &BookInsert,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO books
         (title, author, format, file_path, source_path, normalized_path, file_size,
          file_mtime, file_hash, cover_path, description, category, imported_at,
          created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '', ?, ?, ?)",
    )
    .bind(&book.title)
    .bind(&book.author)
    .bind(&book.format)
    .bind(&book.file_path)
    .bind(book.source_path.as_deref())
    .bind(&book.normalized_path)
    .bind(book.file_size)
    .bind(book.file_mtime)
    .bind(&book.file_hash)
    .bind(book.cover_path.as_deref())
    .bind(&book.description)
    .bind(book.imported_at)
    .bind(book.imported_at)
    .bind(book.imported_at)
    .execute(&mut *connection)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update(
    connection: &mut SqliteConnection,
    id: i64,
    input: &BookUpdateInput,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE books
         SET title = ?, author = ?, description = ?, category = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&input.title)
    .bind(&input.author)
    .bind(&input.description)
    .bind(&input.category)
    .bind(now)
    .bind(id)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn relocate(
    connection: &mut SqliteConnection,
    id: i64,
    target: &BookRelocation,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE books
         SET file_path = ?, source_path = NULL, normalized_path = ?, file_size = ?,
             file_mtime = ?, file_hash = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&target.file_path)
    .bind(&target.normalized_path)
    .bind(target.file_size)
    .bind(target.file_mtime)
    .bind(&target.file_hash)
    .bind(now)
    .bind(id)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn delete(connection: &mut SqliteConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM books WHERE id = ?")
        .bind(id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn cleanup_polymorphic(
    connection: &mut SqliteConnection,
    book_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM note_relations
         WHERE (target_type = 'book' AND target_id = ?)
            OR (target_type = 'annotation'
                AND target_id IN (SELECT id FROM annotations WHERE book_id = ?))",
    )
    .bind(book_id)
    .bind(book_id)
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "DELETE FROM entity_tags
         WHERE (entity_type = 'book' AND entity_id = ?)
            OR (entity_type = 'annotation'
                AND entity_id IN (SELECT id FROM annotations WHERE book_id = ?))",
    )
    .bind(book_id)
    .bind(book_id)
    .execute(&mut *connection)
    .await?;

    sqlx::query("DELETE FROM favorites WHERE entity_type = 'book' AND entity_id = ?")
        .bind(book_id)
        .execute(&mut *connection)
        .await?;

    Ok(())
}

pub async fn set_tags(
    connection: &mut SqliteConnection,
    book_id: i64,
    names: &[String],
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM entity_tags WHERE entity_type = 'book' AND entity_id = ?")
        .bind(book_id)
        .execute(&mut *connection)
        .await?;

    for name in names {
        sqlx::query("INSERT OR IGNORE INTO tags (name, created_at) VALUES (?, ?)")
            .bind(name)
            .bind(now)
            .execute(&mut *connection)
            .await?;

        let tag_id: i64 = sqlx::query_scalar("SELECT id FROM tags WHERE name = ?")
            .bind(name)
            .fetch_one(&mut *connection)
            .await?;

        sqlx::query(
            "INSERT OR IGNORE INTO entity_tags (entity_type, entity_id, tag_id, created_at)
             VALUES ('book', ?, ?, ?)",
        )
        .bind(book_id)
        .bind(tag_id)
        .bind(now)
        .execute(&mut *connection)
        .await?;
    }

    Ok(())
}

pub async fn set_favorite(
    connection: &mut SqliteConnection,
    book_id: i64,
    favorite: bool,
    now: i64,
) -> Result<(), sqlx::Error> {
    if favorite {
        sqlx::query(
            "INSERT OR IGNORE INTO favorites (entity_type, entity_id, created_at)
             VALUES ('book', ?, ?)",
        )
        .bind(book_id)
        .bind(now)
        .execute(&mut *connection)
        .await?;
    } else {
        sqlx::query("DELETE FROM favorites WHERE entity_type = 'book' AND entity_id = ?")
            .bind(book_id)
            .execute(&mut *connection)
            .await?;
    }

    Ok(())
}

pub async fn tags_for(pool: &SqlitePool, book_id: i64) -> Result<Vec<Tag>, sqlx::Error> {
    query_as::<_, Tag>(
        "SELECT t.id, t.name, t.color
         FROM tags t
         JOIN entity_tags et ON et.tag_id = t.id
         WHERE et.entity_type = 'book' AND et.entity_id = ?
         ORDER BY t.name COLLATE NOCASE",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
}

pub async fn all_book_tags(pool: &SqlitePool) -> Result<Vec<(i64, Tag)>, sqlx::Error> {
    let rows: Vec<(i64, i64, String, Option<String>)> = query_as(
        "SELECT et.entity_id, t.id, t.name, t.color
         FROM entity_tags et
         JOIN tags t ON t.id = et.tag_id
         WHERE et.entity_type = 'book'
         ORDER BY t.name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(book_id, id, name, color)| (book_id, Tag { id, name, color }))
        .collect())
}

pub async fn favorite_ids(pool: &SqlitePool) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT entity_id FROM favorites WHERE entity_type = 'book'")
        .fetch_all(pool)
        .await
}

pub async fn is_favorite(pool: &SqlitePool, book_id: i64) -> Result<bool, sqlx::Error> {
    let value: i64 = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM favorites WHERE entity_type = 'book' AND entity_id = ?)",
    )
    .bind(book_id)
    .fetch_one(pool)
    .await?;
    Ok(value != 0)
}

pub async fn find_by_normalized_path(
    pool: &SqlitePool,
    normalized_path: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM books WHERE normalized_path = ?")
        .bind(normalized_path)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_hash(pool: &SqlitePool, file_hash: &str) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM books WHERE file_hash = ? LIMIT 1")
        .bind(file_hash)
        .fetch_optional(pool)
        .await
}
