use sqlx::{QueryBuilder, Sqlite, SqliteConnection, SqlitePool, query_as};

use crate::models::note::{NoteFilter, NoteLink, NoteRelationRow, NoteRow};
use crate::models::tag::Tag;
use crate::utils::escape_like;

const NOTE_SELECT: &str = "SELECT n.id, n.title, n.content_md, n.created_at, n.updated_at
    FROM notes n";

pub async fn get_row(
    connection: &mut SqliteConnection,
    id: i64,
) -> Result<Option<NoteRow>, sqlx::Error> {
    query_as::<_, NoteRow>(
        "SELECT id, title, content_md, created_at, updated_at FROM notes WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut *connection)
    .await
}

pub async fn insert(
    connection: &mut SqliteConnection,
    title: &str,
    content: &str,
    now: i64,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO notes (title, content_md, created_at, updated_at) VALUES (?, ?, ?, ?)",
    )
    .bind(title)
    .bind(content)
    .bind(now)
    .bind(now)
    .execute(&mut *connection)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update(
    connection: &mut SqliteConnection,
    id: i64,
    title: &str,
    content: &str,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notes SET title = ?, content_md = ?, updated_at = ? WHERE id = ?")
        .bind(title)
        .bind(content)
        .bind(now)
        .bind(id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn delete(connection: &mut SqliteConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn cleanup_polymorphic(
    connection: &mut SqliteConnection,
    note_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM entity_tags WHERE entity_type = 'note' AND entity_id = ?")
        .bind(note_id)
        .execute(&mut *connection)
        .await?;

    sqlx::query("DELETE FROM favorites WHERE entity_type = 'note' AND entity_id = ?")
        .bind(note_id)
        .execute(&mut *connection)
        .await?;

    Ok(())
}

pub async fn list(pool: &SqlitePool, filter: &NoteFilter) -> Result<Vec<NoteRow>, sqlx::Error> {
    let mut query = QueryBuilder::<Sqlite>::new(NOTE_SELECT);
    query.push(" WHERE 1 = 1");

    if let Some(text) = filter.query.as_deref().filter(|value| !value.is_empty()) {
        if text.chars().count() >= 3 {
            query
                .push(" AND n.id IN (SELECT rowid FROM notes_fts WHERE notes_fts MATCH ")
                .push_bind(phrase(text))
                .push(")");
        } else {
            let pattern = format!("%{}%", escape_like(text));
            query
                .push(" AND (n.title LIKE ")
                .push_bind(pattern.clone())
                .push(" ESCAPE '\\' OR n.content_md LIKE ")
                .push_bind(pattern)
                .push(" ESCAPE '\\')");
        }
    }

    if let Some(tag_id) = filter.tag_id {
        query
            .push(
                " AND n.id IN (SELECT entity_id FROM entity_tags
                           WHERE entity_type = 'note' AND tag_id = ",
            )
            .push_bind(tag_id)
            .push(")");
    }

    if let (Some(target_type), Some(target_id)) =
        (filter.relation_type.as_deref(), filter.relation_id)
    {
        query
            .push(
                " AND n.id IN (SELECT note_id FROM note_relations
                           WHERE target_type = ",
            )
            .push_bind(target_type)
            .push(" AND target_id = ")
            .push_bind(target_id)
            .push(")");
    }

    match filter.sort.as_deref().unwrap_or("updated_desc") {
        "created_desc" => {
            query.push(" ORDER BY n.created_at DESC, n.id DESC");
        }
        _ => {
            query.push(" ORDER BY n.updated_at DESC, n.id DESC");
        }
    }

    query
        .push(" LIMIT ")
        .push_bind(filter.limit.unwrap_or(100))
        .push(" OFFSET ")
        .push_bind(filter.offset.unwrap_or(0));

    query.build_query_as::<NoteRow>().fetch_all(pool).await
}

pub async fn tags_for_many(
    pool: &SqlitePool,
    note_ids: &[i64],
) -> Result<Vec<(i64, i64, String, Option<String>)>, sqlx::Error> {
    if note_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT et.entity_id, t.id, t.name, t.color
         FROM entity_tags et
         JOIN tags t ON t.id = et.tag_id
         WHERE et.entity_type = 'note' AND et.entity_id IN (",
    );
    let mut separated = query.separated(", ");
    for note_id in note_ids {
        separated.push_bind(*note_id);
    }
    separated.push_unseparated(") ORDER BY t.name COLLATE NOCASE");

    query.build_query_as().fetch_all(pool).await
}

pub async fn set_tags(
    connection: &mut SqliteConnection,
    note_id: i64,
    names: &[String],
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM entity_tags WHERE entity_type = 'note' AND entity_id = ?")
        .bind(note_id)
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
             VALUES ('note', ?, ?, ?)",
        )
        .bind(note_id)
        .bind(tag_id)
        .bind(now)
        .execute(&mut *connection)
        .await?;
    }

    Ok(())
}

pub async fn tags_for(pool: &SqlitePool, note_id: i64) -> Result<Vec<Tag>, sqlx::Error> {
    query_as::<_, Tag>(
        "SELECT t.id, t.name, t.color
         FROM tags t
         JOIN entity_tags et ON et.tag_id = t.id
         WHERE et.entity_type = 'note' AND et.entity_id = ?
         ORDER BY t.name COLLATE NOCASE",
    )
    .bind(note_id)
    .fetch_all(pool)
    .await
}

pub async fn outgoing_links(pool: &SqlitePool, note_id: i64) -> Result<Vec<NoteLink>, sqlx::Error> {
    query_as::<_, NoteLink>(
        "SELECT n.id AS note_id, COALESCE(NULLIF(n.title, ''), '无标题笔记') AS title,
                n.updated_at
         FROM note_links l
         JOIN notes n ON n.id = l.target_note_id
         WHERE l.source_note_id = ?
         ORDER BY n.updated_at DESC, n.id DESC",
    )
    .bind(note_id)
    .fetch_all(pool)
    .await
}

pub async fn backlinks(pool: &SqlitePool, note_id: i64) -> Result<Vec<NoteLink>, sqlx::Error> {
    query_as::<_, NoteLink>(
        "SELECT n.id AS note_id, COALESCE(NULLIF(n.title, ''), '无标题笔记') AS title,
                n.updated_at
         FROM note_links l
         JOIN notes n ON n.id = l.source_note_id
         WHERE l.target_note_id = ?
         ORDER BY n.updated_at DESC, n.id DESC",
    )
    .bind(note_id)
    .fetch_all(pool)
    .await
}

pub async fn add_link(
    connection: &mut SqliteConnection,
    source_note_id: i64,
    target_note_id: i64,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO note_links (source_note_id, target_note_id, created_at)
         VALUES (?, ?, ?)",
    )
    .bind(source_note_id)
    .bind(target_note_id)
    .bind(now)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn remove_link(
    connection: &mut SqliteConnection,
    source_note_id: i64,
    target_note_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM note_links WHERE source_note_id = ? AND target_note_id = ?")
        .bind(source_note_id)
        .bind(target_note_id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn link_candidates(
    pool: &SqlitePool,
    note_id: i64,
    text: &str,
    limit: i64,
) -> Result<Vec<NoteRow>, sqlx::Error> {
    let mut query = QueryBuilder::<Sqlite>::new(NOTE_SELECT);
    query.push(" WHERE n.id <> ").push_bind(note_id);

    if !text.is_empty() {
        if text.chars().count() >= 3 {
            query
                .push(" AND n.id IN (SELECT rowid FROM notes_fts WHERE notes_fts MATCH ")
                .push_bind(phrase(text))
                .push(")");
        } else {
            let pattern = format!("%{}%", escape_like(text));
            query
                .push(" AND (n.title LIKE ")
                .push_bind(pattern.clone())
                .push(" ESCAPE '\\' OR n.content_md LIKE ")
                .push_bind(pattern)
                .push(" ESCAPE '\\')");
        }
    }

    query
        .push(" ORDER BY n.updated_at DESC, n.id DESC LIMIT ")
        .push_bind(limit);

    query.build_query_as::<NoteRow>().fetch_all(pool).await
}

pub async fn relations(
    connection: &mut SqliteConnection,
    note_id: i64,
) -> Result<Vec<NoteRelationRow>, sqlx::Error> {
    let rows: Vec<(i64, String, i64)> = query_as(
        "SELECT id, target_type, target_id FROM note_relations
         WHERE note_id = ? ORDER BY id",
    )
    .bind(note_id)
    .fetch_all(&mut *connection)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, target_type, target_id)| NoteRelationRow {
            id,
            target_type,
            target_id,
        })
        .collect())
}

pub async fn add_relation(
    connection: &mut SqliteConnection,
    note_id: i64,
    target_type: &str,
    target_id: i64,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO note_relations (note_id, target_type, target_id, created_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(note_id)
    .bind(target_type)
    .bind(target_id)
    .bind(now)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn remove_relation(
    connection: &mut SqliteConnection,
    note_id: i64,
    relation_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM note_relations WHERE id = ? AND note_id = ?")
        .bind(relation_id)
        .bind(note_id)
        .execute(&mut *connection)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn relation_count(pool: &SqlitePool, note_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM note_relations WHERE note_id = ?")
        .bind(note_id)
        .fetch_one(pool)
        .await
}

pub async fn relation_impact_notes(
    pool: &SqlitePool,
    target_type: &str,
    target_id: i64,
) -> Result<Vec<NoteLink>, sqlx::Error> {
    let sql = match target_type {
        "anime" => {
            "SELECT DISTINCT n.id AS note_id,
                    COALESCE(NULLIF(n.title, ''), '无标题笔记') AS title, n.updated_at
             FROM note_relations r
             JOIN notes n ON n.id = r.note_id
             WHERE (r.target_type = 'anime' AND r.target_id = ?)
                OR (r.target_type = 'episode'
                    AND r.target_id IN (SELECT id FROM anime_episodes WHERE anime_id = ?))
                OR (r.target_type = 'watch_record'
                    AND r.target_id IN (SELECT id FROM watch_records WHERE anime_id = ?))
             ORDER BY n.updated_at DESC, n.id DESC"
        }
        "book" => {
            "SELECT DISTINCT n.id AS note_id,
                    COALESCE(NULLIF(n.title, ''), '无标题笔记') AS title, n.updated_at
             FROM note_relations r
             JOIN notes n ON n.id = r.note_id
             WHERE (r.target_type = 'book' AND r.target_id = ?)
                OR (r.target_type = 'annotation'
                    AND r.target_id IN (SELECT id FROM annotations WHERE book_id = ?))
             ORDER BY n.updated_at DESC, n.id DESC"
        }
        _ => {
            "SELECT DISTINCT n.id AS note_id,
                    COALESCE(NULLIF(n.title, ''), '无标题笔记') AS title, n.updated_at
             FROM note_relations r
             JOIN notes n ON n.id = r.note_id
             WHERE r.target_type = 'episode' AND r.target_id = ?
             ORDER BY n.updated_at DESC, n.id DESC"
        }
    };

    let query = query_as::<_, NoteLink>(sql).bind(target_id);
    let query = if target_type == "anime" || target_type == "book" {
        query.bind(target_id)
    } else {
        query
    };

    query.fetch_all(pool).await
}

pub async fn anime_candidates(
    pool: &SqlitePool,
    text: &str,
    limit: i64,
) -> Result<Vec<(i64, String, Option<String>, Option<i64>)>, sqlx::Error> {
    let pattern = format!("%{}%", escape_like(text));
    query_as(
        "SELECT id, title, original_title, year FROM anime
         WHERE ? = '' OR title LIKE ? ESCAPE '\\' OR original_title LIKE ? ESCAPE '\\'
         ORDER BY updated_at DESC, id DESC
         LIMIT ?",
    )
    .bind(text)
    .bind(&pattern)
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn book_candidates(
    pool: &SqlitePool,
    text: &str,
    limit: i64,
) -> Result<Vec<(i64, String, String, String)>, sqlx::Error> {
    let pattern = format!("%{}%", escape_like(text));
    query_as(
        "SELECT id, title, author, format FROM books
         WHERE ? = '' OR title LIKE ? ESCAPE '\\' OR author LIKE ? ESCAPE '\\'
         ORDER BY updated_at DESC, id DESC
         LIMIT ?",
    )
    .bind(text)
    .bind(&pattern)
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn episode_candidates(
    pool: &SqlitePool,
    text: &str,
    limit: i64,
) -> Result<Vec<(i64, i64, String, String)>, sqlx::Error> {
    let pattern = format!("%{}%", escape_like(text));
    query_as(
        "SELECT e.id, e.episode_number, e.title, a.title
         FROM anime_episodes e
         JOIN anime a ON a.id = e.anime_id
         WHERE ? = ''
            OR a.title LIKE ? ESCAPE '\\'
            OR e.title LIKE ? ESCAPE '\\'
            OR CAST(e.episode_number AS TEXT) LIKE ? ESCAPE '\\'
         ORDER BY a.updated_at DESC, a.id DESC, e.episode_number ASC
         LIMIT ?",
    )
    .bind(text)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn anime_infos(
    pool: &SqlitePool,
    ids: &[i64],
) -> Result<Vec<(i64, String, Option<String>, Option<i64>)>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT id, title, original_title, year FROM anime WHERE id IN (",
    );
    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(*id);
    }
    separated.push_unseparated(")");

    query.build_query_as().fetch_all(pool).await
}

pub async fn book_infos(
    pool: &SqlitePool,
    ids: &[i64],
) -> Result<Vec<(i64, String, String, String)>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query =
        QueryBuilder::<Sqlite>::new("SELECT id, title, author, format FROM books WHERE id IN (");
    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(*id);
    }
    separated.push_unseparated(")");

    query.build_query_as().fetch_all(pool).await
}

pub async fn episode_infos(
    pool: &SqlitePool,
    ids: &[i64],
) -> Result<Vec<(i64, i64, String, String)>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT e.id, e.episode_number, e.title, a.title
         FROM anime_episodes e
         JOIN anime a ON a.id = e.anime_id
         WHERE e.id IN (",
    );
    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(*id);
    }
    separated.push_unseparated(")");

    query.build_query_as().fetch_all(pool).await
}

fn phrase(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "\"\""))
}
