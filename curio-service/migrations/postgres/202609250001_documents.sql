-- Documents domain: owner-scoped files, immutable content versions, version
-- bytes, and per-version chunks with embeddings and full-text search.

CREATE TABLE document_files (
    id text PRIMARY KEY,
    owner_id text NOT NULL,
    display_name text NOT NULL,
    source_uri text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    deleted_at timestamptz(3),
    CONSTRAINT document_files_owner_fk
        FOREIGN KEY (owner_id)
        REFERENCES users (id)
        ON DELETE CASCADE,
    CONSTRAINT document_files_display_name_check
        CHECK (display_name <> '' AND display_name = btrim(display_name))
);

CREATE INDEX document_files_owner_idx
    ON document_files (owner_id, display_name, id)
    WHERE deleted_at IS NULL;

CREATE TABLE document_file_versions (
    id text PRIMARY KEY,
    file_id text NOT NULL,
    version_number bigint NOT NULL,
    original_filename text NOT NULL,
    mime_type text NOT NULL,
    content_sha256 text NOT NULL,
    byte_size bigint NOT NULL,
    is_current boolean NOT NULL DEFAULT false,
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    indexed_at timestamptz(3),
    CONSTRAINT document_file_versions_file_fk
        FOREIGN KEY (file_id)
        REFERENCES document_files (id)
        ON DELETE CASCADE,
    CONSTRAINT document_file_versions_number_key UNIQUE (file_id, version_number),
    CONSTRAINT document_file_versions_content_key UNIQUE (file_id, content_sha256),
    CONSTRAINT document_file_versions_number_check CHECK (version_number > 0),
    CONSTRAINT document_file_versions_byte_size_check CHECK (byte_size > 0),
    CONSTRAINT document_file_versions_content_sha256_check
        CHECK (content_sha256 ~ '^[0-9a-f]{64}$')
);

-- At most one current version per file.
CREATE UNIQUE INDEX document_file_versions_current_idx
    ON document_file_versions (file_id)
    WHERE is_current;

CREATE INDEX document_file_versions_content_idx
    ON document_file_versions (content_sha256)
    WHERE is_current;

CREATE TABLE document_file_blobs (
    version_id text PRIMARY KEY,
    file_bytes bytea NOT NULL,
    CONSTRAINT document_file_blobs_version_fk
        FOREIGN KEY (version_id)
        REFERENCES document_file_versions (id)
        ON DELETE CASCADE,
    CONSTRAINT document_file_blobs_nonempty_check CHECK (octet_length(file_bytes) > 0)
);

CREATE TABLE document_chunks (
    id text PRIMARY KEY,
    -- Denormalized from the file so search can filter by owner first.
    owner_id text NOT NULL,
    file_id text NOT NULL,
    version_id text NOT NULL,
    chunk_index bigint NOT NULL,
    text text NOT NULL,
    text_search tsvector GENERATED ALWAYS AS (to_tsvector('english', text)) STORED,
    embedding real[] NOT NULL,
    chunk_sha256 text NOT NULL,
    token_count bigint NOT NULL,
    page_start bigint,
    page_end bigint,
    -- Exclusive UTF-8 byte offsets into the document's canonical text.
    char_start bigint NOT NULL,
    char_end bigint NOT NULL,
    section_path text NOT NULL DEFAULT '',
    created_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT document_chunks_version_fk
        FOREIGN KEY (version_id)
        REFERENCES document_file_versions (id)
        ON DELETE CASCADE,
    CONSTRAINT document_chunks_version_index_key UNIQUE (version_id, chunk_index),
    CONSTRAINT document_chunks_index_check CHECK (chunk_index >= 0),
    CONSTRAINT document_chunks_token_count_check CHECK (token_count >= 0),
    CONSTRAINT document_chunks_embedding_check
        CHECK (cardinality(embedding) > 0 AND array_ndims(embedding) = 1),
    CONSTRAINT document_chunks_char_range_check
        CHECK (char_start >= 0 AND char_start <= char_end),
    CONSTRAINT document_chunks_page_range_check
        CHECK (
            (page_start IS NULL AND page_end IS NULL)
            OR (page_start IS NOT NULL AND page_end IS NOT NULL AND page_start <= page_end)
        )
);

CREATE INDEX document_chunks_owner_file_idx
    ON document_chunks (owner_id, file_id, chunk_index);

CREATE INDEX document_chunks_text_search_idx
    ON document_chunks USING GIN (text_search);
