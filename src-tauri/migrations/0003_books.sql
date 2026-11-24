-- Books and reading sessions.
CREATE TABLE books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    author TEXT NOT NULL DEFAULT '',
    format TEXT NOT NULL CHECK (format IN ('epub', 'pdf', 'txt', 'markdown')),
    file_path TEXT NOT NULL,
    normalized_path TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL CHECK (file_size >= 0),
    file_mtime INTEGER,
    file_hash TEXT,
    cover_path TEXT,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT '',
    imported_at INTEGER NOT NULL,
    last_opened_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_books_format ON books (format);
CREATE INDEX idx_books_file_hash ON books (file_hash);
CREATE INDEX idx_books_updated_at ON books (updated_at DESC);
CREATE INDEX idx_books_last_opened ON books (last_opened_at DESC);
CREATE INDEX idx_books_category ON books (category);

CREATE TABLE reading_progress (
    book_id INTEGER PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
    location_type TEXT NOT NULL
        CHECK (location_type IN ('epub_cfi', 'pdf_page', 'txt_offset', 'md_scroll')),
    location_json TEXT NOT NULL,
    percent REAL NOT NULL DEFAULT 0 CHECK (percent >= 0 AND percent <= 1),
    updated_at INTEGER NOT NULL
);

CREATE TABLE reading_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    duration_ms INTEGER CHECK (duration_ms IS NULL OR duration_ms >= 0),
    start_location_json TEXT,
    end_location_json TEXT
);

CREATE INDEX idx_reading_sessions_book ON reading_sessions (book_id, started_at DESC);
CREATE INDEX idx_reading_sessions_started ON reading_sessions (started_at DESC);
