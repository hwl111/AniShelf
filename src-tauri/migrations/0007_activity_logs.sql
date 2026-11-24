-- Activity timeline and Bangumi sync logs.
CREATE TABLE activity_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action_type TEXT NOT NULL CHECK (length(trim(action_type)) > 0),
    entity_type TEXT NOT NULL
        CHECK (entity_type IN ('anime', 'episode', 'watch_record', 'book', 'annotation', 'note', 'tag', 'system')),
    entity_id INTEGER,
    summary TEXT NOT NULL DEFAULT '',
    metadata_json TEXT NOT NULL DEFAULT '{}',
    occurred_at INTEGER NOT NULL
);

CREATE INDEX idx_activity_logs_occurred ON activity_logs (occurred_at DESC);
CREATE INDEX idx_activity_logs_entity ON activity_logs (entity_type, entity_id);

CREATE TABLE anime_metadata_sync_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anime_id INTEGER REFERENCES anime(id) ON DELETE SET NULL,
    bangumi_subject_id INTEGER,
    status TEXT NOT NULL CHECK (status IN ('success', 'partial', 'error')),
    message TEXT NOT NULL DEFAULT '',
    fields_updated_json TEXT NOT NULL DEFAULT '[]',
    started_at INTEGER NOT NULL,
    finished_at INTEGER
);

CREATE INDEX idx_sync_logs_anime ON anime_metadata_sync_logs (anime_id, started_at DESC);
