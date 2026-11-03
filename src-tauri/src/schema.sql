PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE IF NOT EXISTS songs (
 id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, author TEXT NOT NULL DEFAULT '', category TEXT NOT NULL DEFAULT '',
 language TEXT NOT NULL DEFAULT 'English', copyright_info TEXT NOT NULL DEFAULT '', media_id INTEGER, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
 updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY(media_id) REFERENCES media_files(id) ON DELETE SET NULL
);
CREATE TABLE IF NOT EXISTS song_sections (
 id INTEGER PRIMARY KEY AUTOINCREMENT, song_id INTEGER NOT NULL, section_type TEXT NOT NULL, label TEXT NOT NULL,
 content TEXT NOT NULL, position INTEGER NOT NULL DEFAULT 0, FOREIGN KEY(song_id) REFERENCES songs(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS song_translations (
 id INTEGER PRIMARY KEY AUTOINCREMENT, song_id INTEGER NOT NULL, section_id INTEGER, language TEXT NOT NULL, title TEXT NOT NULL DEFAULT '',
 content TEXT NOT NULL, FOREIGN KEY(song_id) REFERENCES songs(id) ON DELETE CASCADE, FOREIGN KEY(section_id) REFERENCES song_sections(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS bible_versions (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, abbreviation TEXT NOT NULL UNIQUE, language TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS bible_books (id INTEGER PRIMARY KEY AUTOINCREMENT, version_id INTEGER NOT NULL, book_number INTEGER NOT NULL, name TEXT NOT NULL, abbreviation TEXT NOT NULL, FOREIGN KEY(version_id) REFERENCES bible_versions(id) ON DELETE CASCADE);
CREATE TABLE IF NOT EXISTS bible_verses (id INTEGER PRIMARY KEY AUTOINCREMENT, version_id INTEGER NOT NULL, book_id INTEGER NOT NULL, chapter INTEGER NOT NULL, verse INTEGER NOT NULL, text TEXT NOT NULL, FOREIGN KEY(version_id) REFERENCES bible_versions(id) ON DELETE CASCADE, FOREIGN KEY(book_id) REFERENCES bible_books(id) ON DELETE CASCADE, UNIQUE(version_id, book_id, chapter, verse));
CREATE TABLE IF NOT EXISTS media_files (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, file_name TEXT NOT NULL, path TEXT NOT NULL, media_type TEXT NOT NULL, is_default INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
CREATE TABLE IF NOT EXISTS projection_settings (id INTEGER PRIMARY KEY CHECK(id=1), font_family TEXT NOT NULL DEFAULT 'Georgia', font_size INTEGER NOT NULL DEFAULT 64, text_color TEXT NOT NULL DEFAULT '#ffffff', alignment TEXT NOT NULL DEFAULT 'center', overlay_opacity REAL NOT NULL DEFAULT 0.46, transition_ms INTEGER NOT NULL DEFAULT 450, default_media_id INTEGER);
CREATE TABLE IF NOT EXISTS app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title);
CREATE INDEX IF NOT EXISTS idx_bible_ref ON bible_verses(version_id, book_id, chapter, verse);
INSERT OR IGNORE INTO projection_settings(id) VALUES(1);
INSERT OR IGNORE INTO schema_migrations(version) VALUES(1);
UPDATE OR IGNORE bible_versions SET abbreviation='NKJV' WHERE upper(name) LIKE '%NEW KING JAMES%';
UPDATE OR IGNORE bible_versions SET abbreviation='LSG' WHERE upper(name) LIKE '%LOUIS SEGOND%';
UPDATE OR IGNORE bible_versions SET abbreviation='LUTH' WHERE upper(name) LIKE '%LUTHER%';
UPDATE OR IGNORE bible_versions SET abbreviation='NKJV', name='New King James Version' WHERE upper(name) LIKE '%NJKV%' OR upper(name) LIKE '%NKJV%';
UPDATE OR IGNORE bible_versions SET abbreviation='LSG', name='Louis Segond' WHERE upper(name) LIKE '%LSG%';
-- 2: indexes for lookups that run once per song, section and Bible book
CREATE INDEX IF NOT EXISTS idx_sections_song ON song_sections(song_id, position);
CREATE INDEX IF NOT EXISTS idx_translations_section ON song_translations(section_id);
CREATE INDEX IF NOT EXISTS idx_books_version ON bible_books(version_id, book_number);
DROP INDEX IF EXISTS idx_sections_content;
DROP INDEX IF EXISTS idx_bible_text;
INSERT OR IGNORE INTO schema_migrations(version) VALUES(2);
