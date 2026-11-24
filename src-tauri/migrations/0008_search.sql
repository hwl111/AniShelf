-- FTS5 trigram indexes with external-content sync triggers.
CREATE VIRTUAL TABLE anime_fts USING fts5(
    title,
    original_title,
    aliases_json,
    summary,
    content = 'anime',
    content_rowid = 'id',
    tokenize = 'trigram'
);

CREATE TRIGGER anime_fts_ai AFTER INSERT ON anime BEGIN
    INSERT INTO anime_fts (rowid, title, original_title, aliases_json, summary)
    VALUES (new.id, new.title, new.original_title, new.aliases_json, new.summary);
END;

CREATE TRIGGER anime_fts_ad AFTER DELETE ON anime BEGIN
    INSERT INTO anime_fts (anime_fts, rowid, title, original_title, aliases_json, summary)
    VALUES ('delete', old.id, old.title, old.original_title, old.aliases_json, old.summary);
END;

CREATE TRIGGER anime_fts_au AFTER UPDATE ON anime BEGIN
    INSERT INTO anime_fts (anime_fts, rowid, title, original_title, aliases_json, summary)
    VALUES ('delete', old.id, old.title, old.original_title, old.aliases_json, old.summary);
    INSERT INTO anime_fts (rowid, title, original_title, aliases_json, summary)
    VALUES (new.id, new.title, new.original_title, new.aliases_json, new.summary);
END;

CREATE VIRTUAL TABLE books_fts USING fts5(
    title,
    author,
    description,
    content = 'books',
    content_rowid = 'id',
    tokenize = 'trigram'
);

CREATE TRIGGER books_fts_ai AFTER INSERT ON books BEGIN
    INSERT INTO books_fts (rowid, title, author, description)
    VALUES (new.id, new.title, new.author, new.description);
END;

CREATE TRIGGER books_fts_ad AFTER DELETE ON books BEGIN
    INSERT INTO books_fts (books_fts, rowid, title, author, description)
    VALUES ('delete', old.id, old.title, old.author, old.description);
END;

CREATE TRIGGER books_fts_au AFTER UPDATE ON books BEGIN
    INSERT INTO books_fts (books_fts, rowid, title, author, description)
    VALUES ('delete', old.id, old.title, old.author, old.description);
    INSERT INTO books_fts (rowid, title, author, description)
    VALUES (new.id, new.title, new.author, new.description);
END;

CREATE VIRTUAL TABLE notes_fts USING fts5(
    title,
    content_md,
    content = 'notes',
    content_rowid = 'id',
    tokenize = 'trigram'
);

CREATE TRIGGER notes_fts_ai AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts (rowid, title, content_md)
    VALUES (new.id, new.title, new.content_md);
END;

CREATE TRIGGER notes_fts_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts (notes_fts, rowid, title, content_md)
    VALUES ('delete', old.id, old.title, old.content_md);
END;

CREATE TRIGGER notes_fts_au AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts (notes_fts, rowid, title, content_md)
    VALUES ('delete', old.id, old.title, old.content_md);
    INSERT INTO notes_fts (rowid, title, content_md)
    VALUES (new.id, new.title, new.content_md);
END;

CREATE VIRTUAL TABLE annotations_fts USING fts5(
    selected_text,
    note,
    content = 'annotations',
    content_rowid = 'id',
    tokenize = 'trigram'
);

CREATE TRIGGER annotations_fts_ai AFTER INSERT ON annotations BEGIN
    INSERT INTO annotations_fts (rowid, selected_text, note)
    VALUES (new.id, new.selected_text, new.note);
END;

CREATE TRIGGER annotations_fts_ad AFTER DELETE ON annotations BEGIN
    INSERT INTO annotations_fts (annotations_fts, rowid, selected_text, note)
    VALUES ('delete', old.id, old.selected_text, old.note);
END;

CREATE TRIGGER annotations_fts_au AFTER UPDATE ON annotations BEGIN
    INSERT INTO annotations_fts (annotations_fts, rowid, selected_text, note)
    VALUES ('delete', old.id, old.selected_text, old.note);
    INSERT INTO annotations_fts (rowid, selected_text, note)
    VALUES (new.id, new.selected_text, new.note);
END;

-- Populate indexes for any rows created before this migration.
INSERT INTO anime_fts (anime_fts) VALUES ('rebuild');
INSERT INTO books_fts (books_fts) VALUES ('rebuild');
INSERT INTO notes_fts (notes_fts) VALUES ('rebuild');
INSERT INTO annotations_fts (annotations_fts) VALUES ('rebuild');
