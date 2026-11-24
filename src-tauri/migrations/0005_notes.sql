-- Notes, tags, note links and polymorphic entity tags.
CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL DEFAULT '',
    content_md TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    CHECK (length(trim(title)) > 0 OR length(trim(content_md)) > 0)
);

CREATE INDEX idx_notes_updated_at ON notes (updated_at DESC);
CREATE INDEX idx_notes_created_at ON notes (created_at DESC);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
    color TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE entity_tags (
    entity_type TEXT NOT NULL
        CHECK (entity_type IN ('anime', 'book', 'note', 'annotation')),
    entity_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (entity_type, entity_id, tag_id)
);

CREATE INDEX idx_entity_tags_tag ON entity_tags (tag_id);
CREATE INDEX idx_entity_tags_entity ON entity_tags (entity_type, entity_id);

CREATE TABLE note_links (
    source_note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    target_note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (source_note_id, target_note_id),
    CHECK (source_note_id <> target_note_id)
);

CREATE INDEX idx_note_links_target ON note_links (target_note_id);
