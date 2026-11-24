use sqlx::query_as;
use sqlx::{QueryBuilder, Sqlite, SqliteConnection, SqlitePool};

use crate::models::anime::{AnimeFilter, AnimeInput, AnimeRow, ExternalAnimeFields};
use crate::models::tag::Tag;

const ANIME_SELECT: &str = "SELECT a.id, a.bangumi_subject_id, a.title, a.original_title,
    a.aliases_json, a.summary, a.cover_path, a.cover_source_url, a.year, a.season,
    a.total_episodes, a.external_updated_at, a.watch_status, a.rating, a.review,
    a.created_at, a.updated_at,
    (SELECT COUNT(DISTINCT w.episode_number) FROM watch_records w WHERE w.anime_id = a.id)
        AS watched_episodes
    FROM anime a";

pub async fn list(pool: &SqlitePool, filter: &AnimeFilter) -> Result<Vec<AnimeRow>, sqlx::Error> {
    let mut query = QueryBuilder::<Sqlite>::new(ANIME_SELECT);
    query.push(" WHERE 1 = 1");

    if let Some(status) = filter.status.as_deref() {
        query.push(" AND a.watch_status = ").push_bind(status);
    }
    if let Some(year) = filter.year {
        query.push(" AND a.year = ").push_bind(year);
    }
    if let Some(tag_id) = filter.tag_id {
        query
            .push(
                " AND a.id IN (SELECT entity_id FROM entity_tags
                           WHERE entity_type = 'anime' AND tag_id = ",
            )
            .push_bind(tag_id)
            .push(")");
    }

    match filter.sort.as_deref().unwrap_or("updated_desc") {
        "created_desc" => {
            query.push(" ORDER BY a.created_at DESC");
        }
        "title_asc" => {
            query.push(" ORDER BY a.title COLLATE NOCASE ASC");
        }
        "rating_desc" => {
            query.push(" ORDER BY (a.rating IS NULL) ASC, a.rating DESC, a.updated_at DESC");
        }
        "year_desc" => {
            query.push(" ORDER BY (a.year IS NULL) ASC, a.year DESC, a.updated_at DESC");
        }
        _ => {
            query.push(" ORDER BY a.updated_at DESC");
        }
    }

    query.build_query_as::<AnimeRow>().fetch_all(pool).await
}

pub async fn get_row(
    connection: &mut SqliteConnection,
    id: i64,
) -> Result<Option<AnimeRow>, sqlx::Error> {
    let sql = format!("{ANIME_SELECT} WHERE a.id = ?");
    query_as::<_, AnimeRow>(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(&mut *connection)
        .await
}

pub async fn insert(
    connection: &mut SqliteConnection,
    input: &AnimeInput,
    aliases_json: &str,
    now: i64,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO anime
         (title, original_title, aliases_json, summary, year, season, total_episodes,
          watch_status, rating, review, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&input.title)
    .bind(input.original_title.as_deref())
    .bind(aliases_json)
    .bind(&input.summary)
    .bind(input.year)
    .bind(input.season.as_deref())
    .bind(input.total_episodes)
    .bind(&input.watch_status)
    .bind(input.rating)
    .bind(&input.review)
    .bind(now)
    .bind(now)
    .execute(&mut *connection)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update(
    connection: &mut SqliteConnection,
    id: i64,
    input: &AnimeInput,
    aliases_json: &str,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE anime
         SET title = ?, original_title = ?, aliases_json = ?, summary = ?, year = ?,
             season = ?, total_episodes = ?, watch_status = ?, rating = ?, review = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(&input.title)
    .bind(input.original_title.as_deref())
    .bind(aliases_json)
    .bind(&input.summary)
    .bind(input.year)
    .bind(input.season.as_deref())
    .bind(input.total_episodes)
    .bind(&input.watch_status)
    .bind(input.rating)
    .bind(&input.review)
    .bind(now)
    .bind(id)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn delete(connection: &mut SqliteConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM anime WHERE id = ?")
        .bind(id)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

pub async fn find_id_by_bangumi_subject_id(
    pool: &SqlitePool,
    subject_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM anime WHERE bangumi_subject_id = ?")
        .bind(subject_id)
        .fetch_optional(pool)
        .await
}

pub async fn update_external(
    connection: &mut SqliteConnection,
    id: i64,
    fields: &ExternalAnimeFields,
    aliases_json: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE anime
         SET bangumi_subject_id = ?, title = ?, original_title = ?, aliases_json = ?,
             summary = ?, cover_source_url = ?, year = ?, season = ?, total_episodes = ?,
             external_updated_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(fields.bangumi_subject_id)
    .bind(&fields.title)
    .bind(fields.original_title.as_deref())
    .bind(aliases_json)
    .bind(&fields.summary)
    .bind(fields.cover_source_url.as_deref())
    .bind(fields.year)
    .bind(fields.season.as_deref())
    .bind(fields.total_episodes)
    .bind(fields.updated_at)
    .bind(fields.updated_at)
    .bind(id)
    .execute(&mut *connection)
    .await?;

    Ok(())
}

pub async fn cleanup_polymorphic(
    connection: &mut SqliteConnection,
    anime_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM note_relations
         WHERE (target_type = 'anime' AND target_id = ?)
            OR (target_type = 'episode'
                AND target_id IN (SELECT id FROM anime_episodes WHERE anime_id = ?))
            OR (target_type = 'watch_record'
                AND target_id IN (SELECT id FROM watch_records WHERE anime_id = ?))",
    )
    .bind(anime_id)
    .bind(anime_id)
    .bind(anime_id)
    .execute(&mut *connection)
    .await?;

    sqlx::query("DELETE FROM entity_tags WHERE entity_type = 'anime' AND entity_id = ?")
        .bind(anime_id)
        .execute(&mut *connection)
        .await?;

    sqlx::query("DELETE FROM favorites WHERE entity_type = 'anime' AND entity_id = ?")
        .bind(anime_id)
        .execute(&mut *connection)
        .await?;

    Ok(())
}

pub async fn set_tags(
    connection: &mut SqliteConnection,
    anime_id: i64,
    names: &[String],
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM entity_tags WHERE entity_type = 'anime' AND entity_id = ?")
        .bind(anime_id)
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
             VALUES ('anime', ?, ?, ?)",
        )
        .bind(anime_id)
        .bind(tag_id)
        .bind(now)
        .execute(&mut *connection)
        .await?;
    }

    Ok(())
}

pub async fn set_favorite(
    connection: &mut SqliteConnection,
    anime_id: i64,
    favorite: bool,
    now: i64,
) -> Result<(), sqlx::Error> {
    if favorite {
        sqlx::query(
            "INSERT OR IGNORE INTO favorites (entity_type, entity_id, created_at)
             VALUES ('anime', ?, ?)",
        )
        .bind(anime_id)
        .bind(now)
        .execute(&mut *connection)
        .await?;
    } else {
        sqlx::query("DELETE FROM favorites WHERE entity_type = 'anime' AND entity_id = ?")
            .bind(anime_id)
            .execute(&mut *connection)
            .await?;
    }

    Ok(())
}

pub async fn tags_for(pool: &SqlitePool, anime_id: i64) -> Result<Vec<Tag>, sqlx::Error> {
    query_as::<_, Tag>(
        "SELECT t.id, t.name, t.color
         FROM tags t
         JOIN entity_tags et ON et.tag_id = t.id
         WHERE et.entity_type = 'anime' AND et.entity_id = ?
         ORDER BY t.name COLLATE NOCASE",
    )
    .bind(anime_id)
    .fetch_all(pool)
    .await
}

pub async fn all_anime_tags(pool: &SqlitePool) -> Result<Vec<(i64, Tag)>, sqlx::Error> {
    let rows: Vec<(i64, i64, String, Option<String>)> = query_as(
        "SELECT et.entity_id, t.id, t.name, t.color
         FROM entity_tags et
         JOIN tags t ON t.id = et.tag_id
         WHERE et.entity_type = 'anime'
         ORDER BY t.name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(anime_id, id, name, color)| (anime_id, Tag { id, name, color }))
        .collect())
}

pub async fn favorite_ids(pool: &SqlitePool) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT entity_id FROM favorites WHERE entity_type = 'anime'")
        .fetch_all(pool)
        .await
}

pub async fn is_favorite(pool: &SqlitePool, anime_id: i64) -> Result<bool, sqlx::Error> {
    let value: i64 = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM favorites WHERE entity_type = 'anime' AND entity_id = ?)",
    )
    .bind(anime_id)
    .fetch_one(pool)
    .await?;
    Ok(value != 0)
}

pub async fn list_tags(pool: &SqlitePool) -> Result<Vec<Tag>, sqlx::Error> {
    query_as::<_, Tag>("SELECT id, name, color FROM tags ORDER BY name COLLATE NOCASE")
        .fetch_all(pool)
        .await
}
