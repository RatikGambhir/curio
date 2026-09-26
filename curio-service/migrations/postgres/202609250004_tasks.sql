CREATE TABLE tasks (
    id text PRIMARY KEY,
    owner_id text NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    space_id text,
    parent_id text,
    title text NOT NULL,
    description text,
    status text NOT NULL DEFAULT 'todo',
    priority text,
    flagged boolean NOT NULL DEFAULT false,
    due_on date,
    due_at timestamptz(3),
    due_timezone text,
    completed_at timestamptz(3),
    sort_order bigint NOT NULL DEFAULT 0,
    version bigint NOT NULL DEFAULT 1,
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT tasks_owner_id_key UNIQUE(owner_id, id),
    CONSTRAINT tasks_space_fk FOREIGN KEY(owner_id, space_id) REFERENCES spaces(owner_id, id),
    CONSTRAINT tasks_parent_fk FOREIGN KEY(owner_id, parent_id) REFERENCES tasks(owner_id, id),
    CONSTRAINT tasks_title_check CHECK(title <> '' AND title = btrim(title) AND char_length(title) <= 240),
    CONSTRAINT tasks_description_check CHECK(description IS NULL OR char_length(description) <= 10000),
    CONSTRAINT tasks_status_check CHECK(status IN ('todo', 'in_progress', 'blocked', 'done', 'cancelled')),
    CONSTRAINT tasks_priority_check CHECK(priority IS NULL OR priority IN ('low', 'medium', 'high')),
    CONSTRAINT tasks_due_check CHECK(NOT (due_on IS NOT NULL AND due_at IS NOT NULL)),
    CONSTRAINT tasks_due_timezone_check CHECK((due_at IS NULL AND due_timezone IS NULL) OR (due_at IS NOT NULL AND due_timezone IS NOT NULL AND char_length(due_timezone) BETWEEN 1 AND 100)),
    CONSTRAINT tasks_completed_check CHECK((status = 'done') = (completed_at IS NOT NULL)),
    CONSTRAINT tasks_version_check CHECK(version > 0),
    CONSTRAINT tasks_order_check CHECK(sort_order >= 0),
    CONSTRAINT tasks_parent_check CHECK(parent_id <> id)
);
CREATE INDEX tasks_owner_created_idx ON tasks(owner_id, created_at DESC, id DESC);
CREATE INDEX tasks_space_order_idx ON tasks(owner_id, space_id, sort_order, id);
CREATE INDEX tasks_parent_order_idx ON tasks(owner_id, parent_id, sort_order, id);
CREATE INDEX tasks_due_on_idx ON tasks(owner_id, due_on, id) WHERE due_on IS NOT NULL;
CREATE INDEX tasks_due_at_idx ON tasks(owner_id, due_at, id) WHERE due_at IS NOT NULL;
CREATE INDEX tasks_status_created_idx ON tasks(owner_id, status, created_at DESC, id DESC);
CREATE INDEX tasks_flagged_created_idx ON tasks(owner_id, created_at DESC, id DESC) WHERE flagged;

CREATE TABLE task_tags (
    id text PRIMARY KEY,
    owner_id text NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name text NOT NULL,
    name_key text NOT NULL,
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT task_tags_owner_id_key UNIQUE(owner_id, id),
    CONSTRAINT task_tags_owner_name_key UNIQUE(owner_id, name_key),
    CONSTRAINT task_tags_name_check CHECK(name <> '' AND name = btrim(name) AND char_length(name) <= 80),
    CONSTRAINT task_tags_key_check CHECK(name_key <> '' AND char_length(name_key) <= 240)
);
CREATE INDEX task_tags_owner_created_idx ON task_tags(owner_id, created_at DESC, id DESC);

CREATE TABLE task_tag_assignments (
    owner_id text NOT NULL,
    task_id text NOT NULL,
    tag_id text NOT NULL,
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY(task_id, tag_id),
    CONSTRAINT task_tag_assignments_task_fk FOREIGN KEY(owner_id, task_id) REFERENCES tasks(owner_id, id) ON DELETE CASCADE,
    CONSTRAINT task_tag_assignments_tag_fk FOREIGN KEY(owner_id, tag_id) REFERENCES task_tags(owner_id, id) ON DELETE CASCADE
);
CREATE INDEX task_tag_assignments_tag_idx ON task_tag_assignments(owner_id, tag_id, task_id);

CREATE TABLE task_comments (
    id text PRIMARY KEY,
    owner_id text NOT NULL,
    task_id text NOT NULL,
    author_id text NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body text NOT NULL,
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    edited_at timestamptz(3),
    CONSTRAINT task_comments_task_fk FOREIGN KEY(owner_id, task_id) REFERENCES tasks(owner_id, id) ON DELETE CASCADE,
    CONSTRAINT task_comments_author_check CHECK(author_id = owner_id),
    CONSTRAINT task_comments_body_check CHECK(body <> '' AND body = btrim(body) AND char_length(body) <= 10000)
);
CREATE INDEX task_comments_task_created_idx ON task_comments(owner_id, task_id, created_at DESC, id DESC);

CREATE TABLE task_links (
    id text PRIMARY KEY,
    owner_id text NOT NULL,
    task_id text NOT NULL,
    kind text NOT NULL,
    label text,
    url text,
    note_id text,
    sort_order bigint NOT NULL DEFAULT 0,
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT task_links_task_fk FOREIGN KEY(owner_id, task_id) REFERENCES tasks(owner_id, id) ON DELETE CASCADE,
    CONSTRAINT task_links_kind_check CHECK((kind = 'web' AND url IS NOT NULL AND note_id IS NULL) OR (kind = 'curio_note' AND note_id IS NOT NULL AND url IS NULL)),
    CONSTRAINT task_links_label_check CHECK(label IS NULL OR char_length(label) <= 200),
    CONSTRAINT task_links_url_check CHECK(url IS NULL OR (char_length(url) BETWEEN 1 AND 2048 AND url = btrim(url))),
    CONSTRAINT task_links_note_check CHECK(note_id IS NULL OR (char_length(note_id) BETWEEN 1 AND 240 AND note_id = btrim(note_id))),
    CONSTRAINT task_links_order_check CHECK(sort_order >= 0)
);
CREATE INDEX task_links_task_order_idx ON task_links(owner_id, task_id, sort_order, id);
