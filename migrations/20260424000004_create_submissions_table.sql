CREATE TABLE IF NOT EXISTS submissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exercise_id INTEGER NOT NULL,
    user_code TEXT NOT NULL,
    score INTEGER NOT NULL CHECK(score >= 0 AND score <= 100),
    letter_grade TEXT NOT NULL CHECK(letter_grade IN ('A', 'B', 'C', 'D', 'F')),
    is_partial BOOLEAN NOT NULL DEFAULT 0,
    strengths TEXT NOT NULL,
    improvements TEXT NOT NULL,
    summary TEXT NOT NULL,
    submitted_at TIMESTAMP NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (exercise_id) REFERENCES exercises(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_submissions_exercise ON submissions(exercise_id);
CREATE INDEX IF NOT EXISTS idx_submissions_submitted_at ON submissions(submitted_at);
CREATE INDEX IF NOT EXISTS idx_submissions_score ON submissions(score);