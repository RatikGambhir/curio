CREATE TABLE spaces (
    id text PRIMARY KEY,
    owner_id text NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name text NOT NULL,
    description text,
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT spaces_owner_id_key UNIQUE (owner_id, id),
    CONSTRAINT spaces_name_check
        CHECK (name <> '' AND name = btrim(name) AND char_length(name) <= 120),
    CONSTRAINT spaces_description_check
        CHECK (description IS NULL OR char_length(description) <= 2000)
);

CREATE UNIQUE INDEX spaces_owner_name_ci_idx
    ON spaces (owner_id, lower(name));

CREATE INDEX spaces_owner_created_idx
    ON spaces (owner_id, created_at DESC, id DESC);
