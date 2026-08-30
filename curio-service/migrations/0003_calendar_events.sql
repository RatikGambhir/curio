CREATE TABLE IF NOT EXISTS calendar_events (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT,
    priority TEXT,
    all_day INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0, 1)),
    -- The wire strings, stored verbatim and returned unchanged. A bare
    -- YYYY-MM-DD means all-day; a full timestamp means timed. Normalizing
    -- either one would turn every all-day event into a midnight block.
    start_date TEXT NOT NULL,
    end_date TEXT,
    -- Normalized half-open UTC instants, derived once at write time. Range
    -- queries read these, responses never do.
    starts_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS calendar_events_range_idx
    ON calendar_events (user_id, starts_at, ends_at);
