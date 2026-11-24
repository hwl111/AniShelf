-- Keep the original path when the user copies a book into the app library.
ALTER TABLE books ADD COLUMN source_path TEXT;
