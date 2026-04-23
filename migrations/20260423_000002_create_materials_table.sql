CREATE TABLE IF NOT EXISTS materials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_url TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK(source_type IN ('github', 'tutorial')),
    language TEXT NOT NULL,
    title TEXT NOT NULL,
    difficulty TEXT NOT NULL CHECK(difficulty IN ('beginner', 'intermediate', 'advanced')),
    local_path TEXT NOT NULL,
    fetched_at TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_url)
);

CREATE INDEX IF NOT EXISTS idx_materials_language ON materials(language);
CREATE INDEX IF NOT EXISTS idx_materials_difficulty ON materials(difficulty);
CREATE INDEX IF NOT EXISTS idx_materials_source_type ON materials(source_type);