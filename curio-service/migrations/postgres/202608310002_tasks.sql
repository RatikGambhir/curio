CREATE TABLE tasks (
    id text PRIMARY KEY,
    user_id text NOT NULL,
    name text NOT NULL,
    description text,
    status text NOT NULL DEFAULT 'scheduled',
    priority text,
    active boolean NOT NULL DEFAULT true,
    set_at timestamptz(3) NOT NULL,
    due_at timestamptz(3),
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT tasks_user_fk
        FOREIGN KEY (user_id)
        REFERENCES users (id)
        ON DELETE CASCADE,
    CONSTRAINT tasks_status_check
        CHECK (status IN ('scheduled', 'in-progress', 'blocked', 'done', 'cancelled')),
    CONSTRAINT tasks_priority_check
        CHECK (priority IS NULL OR priority IN ('low', 'medium', 'high')),
    CONSTRAINT tasks_due_check
        CHECK (due_at IS NULL OR due_at >= set_at)
);

CREATE INDEX tasks_user_created_idx
    ON tasks (user_id, created_at DESC, id);
