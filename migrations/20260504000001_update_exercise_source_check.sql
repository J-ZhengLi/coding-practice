-- Expand exercises.source CHECK constraint to accept github and web source labels (D-12)
-- SQLite 3.45.1 does not support ALTER COLUMN CHECK modification, so we recreate the table

CREATE TABLE exercises_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    material_id INTEGER,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    language TEXT NOT NULL,
    difficulty TEXT NOT NULL CHECK(difficulty IN ('beginner', 'intermediate', 'advanced')),
    todo_comment TEXT NOT NULL,
    original_code TEXT NOT NULL,
    exercise_code TEXT NOT NULL,
    concept TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    source TEXT NOT NULL DEFAULT 'scraped' CHECK(source IN ('scraped', 'ai_generated', 'github', 'web')),
    generated_at TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (material_id) REFERENCES materials(id)
);

INSERT INTO exercises_new (id, material_id, title, description, language, difficulty, todo_comment, original_code, exercise_code, concept, start_line, end_line, source, generated_at)
SELECT id, material_id, title, description, language, difficulty, todo_comment, original_code, exercise_code, concept, start_line, end_line, source, generated_at
FROM exercises;

DROP TABLE exercises;
ALTER TABLE exercises_new RENAME TO exercises;

CREATE INDEX IF NOT EXISTS idx_exercises_language ON exercises(language);
CREATE INDEX IF NOT EXISTS idx_exercises_difficulty ON exercises(difficulty);
CREATE INDEX IF NOT EXISTS idx_exercises_material ON exercises(material_id);
CREATE INDEX IF NOT EXISTS idx_exercises_source ON exercises(source);
