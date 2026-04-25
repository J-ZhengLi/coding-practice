CREATE TABLE IF NOT EXISTS review_schedule (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    concept TEXT NOT NULL,
    language TEXT NOT NULL,
    current_interval INTEGER NOT NULL DEFAULT 0,
    last_completed_at TIMESTAMP NOT NULL,
    next_review_at TIMESTAMP NOT NULL,
    last_exercise_id INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK(status IN ('active', 'completed')) DEFAULT 'active'
);

CREATE INDEX IF NOT EXISTS idx_review_schedule_language_status ON review_schedule(language, status);
CREATE INDEX IF NOT EXISTS idx_review_schedule_concept_language ON review_schedule(concept, language);
CREATE INDEX IF NOT EXISTS idx_review_schedule_due ON review_schedule(status, next_review_at);