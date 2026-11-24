-- Anime domain: anime, episodes and watch history.
CREATE TABLE anime (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bangumi_subject_id INTEGER UNIQUE,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    original_title TEXT,
    aliases_json TEXT NOT NULL DEFAULT '[]',
    summary TEXT NOT NULL DEFAULT '',
    cover_path TEXT,
    cover_source_url TEXT,
    year INTEGER CHECK (year IS NULL OR (year BETWEEN 1900 AND 2200)),
    season TEXT CHECK (season IS NULL OR season IN ('winter', 'spring', 'summer', 'fall')),
    total_episodes INTEGER CHECK (total_episodes IS NULL OR total_episodes >= 0),
    external_updated_at INTEGER,
    watch_status TEXT NOT NULL DEFAULT 'wish'
        CHECK (watch_status IN ('wish', 'watching', 'completed', 'on_hold', 'dropped')),
    rating INTEGER CHECK (rating IS NULL OR (rating BETWEEN 1 AND 10)),
    review TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_anime_watch_status ON anime (watch_status);
CREATE INDEX idx_anime_year ON anime (year);
CREATE INDEX idx_anime_updated_at ON anime (updated_at DESC);

CREATE TABLE anime_episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anime_id INTEGER NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    episode_number INTEGER NOT NULL CHECK (episode_number >= 1),
    title TEXT NOT NULL DEFAULT '',
    aired_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (anime_id, episode_number)
);

CREATE INDEX idx_anime_episodes_anime ON anime_episodes (anime_id);

CREATE TABLE watch_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anime_id INTEGER NOT NULL REFERENCES anime(id) ON DELETE CASCADE,
    episode_id INTEGER REFERENCES anime_episodes(id) ON DELETE SET NULL,
    episode_number INTEGER NOT NULL CHECK (episode_number >= 1),
    watched_at INTEGER NOT NULL,
    comment TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_watch_records_anime_watched ON watch_records (anime_id, watched_at DESC);
CREATE INDEX idx_watch_records_episode ON watch_records (episode_id);
CREATE INDEX idx_watch_records_watched ON watch_records (watched_at DESC);
