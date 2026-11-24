-- Bookmarks, annotations and reader preferences.
CREATE TABLE bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    location_type TEXT NOT NULL
        CHECK (location_type IN ('epub_cfi', 'pdf_page', 'txt_offset', 'md_scroll')),
    location_json TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_bookmarks_book ON bookmarks (book_id, created_at DESC);

CREATE TABLE annotations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('highlight', 'note')),
    selected_text TEXT NOT NULL DEFAULT '',
    location_type TEXT NOT NULL
        CHECK (location_type IN ('epub_cfi', 'pdf_page', 'txt_offset', 'md_scroll')),
    location_json TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    color TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_annotations_book ON annotations (book_id, created_at DESC);

CREATE TABLE reader_settings (
    scope TEXT PRIMARY KEY CHECK (scope IN ('global', 'epub', 'pdf', 'txt', 'markdown')),
    preferences_json TEXT NOT NULL DEFAULT '{}',
    updated_at INTEGER NOT NULL
);
