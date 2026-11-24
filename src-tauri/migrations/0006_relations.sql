-- Cross-entity relations, annotation notes, favorites and attachments.
CREATE TABLE note_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    target_type TEXT NOT NULL
        CHECK (target_type IN ('anime', 'book', 'episode', 'watch_record', 'annotation')),
    target_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE (note_id, target_type, target_id)
);

CREATE INDEX idx_note_relations_target ON note_relations (target_type, target_id);
CREATE INDEX idx_note_relations_note ON note_relations (note_id);

CREATE TABLE annotation_notes (
    annotation_id INTEGER NOT NULL REFERENCES annotations(id) ON DELETE CASCADE,
    note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (annotation_id, note_id)
);

CREATE INDEX idx_annotation_notes_note ON annotation_notes (note_id);

CREATE TABLE favorites (
    entity_type TEXT NOT NULL CHECK (entity_type IN ('anime', 'book', 'note')),
    entity_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (entity_type, entity_id)
);

CREATE TABLE attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_type TEXT NOT NULL CHECK (owner_type IN ('note', 'anime', 'book')),
    owner_id INTEGER NOT NULL,
    file_path TEXT NOT NULL CHECK (length(trim(file_path)) > 0),
    file_name TEXT NOT NULL CHECK (length(trim(file_name)) > 0),
    mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    file_size INTEGER NOT NULL CHECK (file_size >= 0),
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_attachments_owner ON attachments (owner_type, owner_id);
