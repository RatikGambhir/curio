//! PostgreSQL adapter for the chunk index.
//!
//! Embeddings are stored as `real[]` and ranked by exact cosine distance in
//! SQL, so no database extension is required; this scans the owner's current
//! chunks and is intended for modest per-owner corpora. Keyword search uses
//! the built-in English full-text configuration over a generated `tsvector`.
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection, Postgres, QueryBuilder};

use super::domain::{
    ChunkRecord, IndexedChunk, KeywordChunkHit, KeywordSearch, VectorChunkHit, VectorSearch,
};
use crate::{
    database::Database,
    documents::{
        domain::DocumentError, search::service::ChunkSearch, viewing::service::DocumentChunks,
    },
};

/// 14 binds per row keeps each statement far below PostgreSQL's bind limit.
const CHUNK_INSERT_BATCH: usize = 500;

const DELETE_VERSION_CHUNKS: &str = "DELETE FROM document_chunks WHERE version_id = $1";

const INSERT_CHUNKS_PREFIX: &str = "INSERT INTO document_chunks (id, owner_id, file_id, version_id, \
chunk_index, text, embedding, chunk_sha256, token_count, page_start, page_end, char_start, \
char_end, section_path) ";

const FILE_IS_VISIBLE: &str = r#"
SELECT EXISTS (
    SELECT 1 FROM document_files
    WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL
)
"#;

const CURRENT_FILE_CHUNKS: &str = r#"
SELECT
    chunk.id AS chunk_id, chunk.file_id, chunk.version_id, file.display_name,
    chunk.chunk_index, chunk.text, chunk.chunk_sha256, chunk.token_count,
    chunk.page_start, chunk.page_end, chunk.char_start, chunk.char_end,
    chunk.section_path, chunk.created_at
FROM document_chunks AS chunk
JOIN document_file_versions AS version
  ON version.id = chunk.version_id AND version.is_current
JOIN document_files AS file
  ON file.id = chunk.file_id AND file.deleted_at IS NULL
WHERE chunk.owner_id = $1 AND chunk.file_id = $2
ORDER BY chunk.chunk_index ASC
"#;

const VECTOR_SEARCH: &str = r#"
SELECT
    chunk.id AS chunk_id, chunk.file_id, chunk.version_id, file.display_name,
    chunk.chunk_index, chunk.text, chunk.chunk_sha256, chunk.token_count,
    chunk.page_start, chunk.page_end, chunk.char_start, chunk.char_end,
    chunk.section_path, chunk.created_at,
    -- Clamp rounding error so the public range is exactly [0, 2]. GREATEST and
    -- LEAST ignore NULL, so clamp only after NULL distances are filtered out.
    GREATEST(0, LEAST(2, similarity.distance)) AS distance
FROM document_chunks AS chunk
JOIN document_file_versions AS version
  ON version.id = chunk.version_id AND version.is_current
JOIN document_files AS file
  ON file.id = chunk.file_id AND file.deleted_at IS NULL
CROSS JOIN LATERAL (
    SELECT 1 - SUM(pair.stored_value::float8 * pair.query_value::float8)
        / NULLIF(
            SQRT(SUM(pair.stored_value::float8 * pair.stored_value::float8))
                * SQRT(SUM(pair.query_value::float8 * pair.query_value::float8)),
            0
        ) AS distance
    FROM UNNEST(chunk.embedding, $2::real[]) AS pair(stored_value, query_value)
) AS similarity
WHERE chunk.owner_id = $1
  AND cardinality(chunk.embedding) = cardinality($2::real[])
  AND similarity.distance IS NOT NULL
ORDER BY similarity.distance ASC, chunk.id ASC
LIMIT $3
"#;

const KEYWORD_SEARCH: &str = r#"
SELECT
    chunk.id AS chunk_id, chunk.file_id, chunk.version_id, file.display_name,
    chunk.chunk_index, chunk.text, chunk.chunk_sha256, chunk.token_count,
    chunk.page_start, chunk.page_end, chunk.char_start, chunk.char_end,
    chunk.section_path, chunk.created_at,
    ts_rank_cd(chunk.text_search, search.query)::float8 AS score
FROM document_chunks AS chunk
JOIN document_file_versions AS version
  ON version.id = chunk.version_id AND version.is_current
JOIN document_files AS file
  ON file.id = chunk.file_id AND file.deleted_at IS NULL
CROSS JOIN websearch_to_tsquery('english', $2) AS search(query)
WHERE chunk.owner_id = $1 AND chunk.text_search @@ search.query
ORDER BY score DESC, chunk.id ASC
LIMIT $3
"#;

/// Replaces a version's chunks inside the caller's aggregate transaction.
pub(crate) async fn replace_version_chunks(
    connection: &mut PgConnection,
    version_id: &str,
    chunks: &[IndexedChunk],
) -> Result<(), sqlx::Error> {
    sqlx::query(DELETE_VERSION_CHUNKS)
        .bind(version_id)
        .execute(&mut *connection)
        .await?;

    for batch in chunks.chunks(CHUNK_INSERT_BATCH) {
        let mut insert = QueryBuilder::<Postgres>::new(INSERT_CHUNKS_PREFIX);
        insert.push_values(batch, |mut row, chunk| {
            row.push_bind(&chunk.chunk_id)
                .push_bind(&chunk.owner_id)
                .push_bind(&chunk.file_id)
                .push_bind(&chunk.version_id)
                .push_bind(chunk.chunk_index)
                .push_bind(&chunk.text)
                .push_bind(chunk.embedding.as_slice())
                .push_bind(&chunk.chunk_sha256)
                .push_bind(chunk.token_count)
                .push_bind(chunk.page_start)
                .push_bind(chunk.page_end)
                .push_bind(chunk.char_start)
                .push_bind(chunk.char_end)
                .push_bind(&chunk.section_path);
        });
        let result = insert.build().execute(&mut *connection).await?;
        if result.rows_affected() != batch.len() as u64 {
            return Err(sqlx::Error::Protocol(
                "document chunk insert affected an unexpected row count".to_owned(),
            ));
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct PostgresChunkIndex {
    database: Database,
}

impl PostgresChunkIndex {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl DocumentChunks for PostgresChunkIndex {
    async fn current_chunks(
        &self,
        owner_id: &str,
        file_id: &str,
    ) -> Result<Option<Vec<ChunkRecord>>, DocumentError> {
        let visible = sqlx::query_scalar::<_, bool>(FILE_IS_VISIBLE)
            .bind(file_id)
            .bind(owner_id)
            .fetch_one(self.database.pool())
            .await
            .map_err(|error| storage_error("documents_chunk_owner", error))?;
        if !visible {
            return Ok(None);
        }
        sqlx::query_as::<_, ChunkRow>(CURRENT_FILE_CHUNKS)
            .bind(owner_id)
            .bind(file_id)
            .fetch_all(self.database.pool())
            .await
            .map(|rows| Some(rows.into_iter().map(Into::into).collect()))
            .map_err(|error| storage_error("documents_current_chunks", error))
    }
}

impl ChunkSearch for PostgresChunkIndex {
    async fn search_vector(
        &self,
        search: &VectorSearch,
    ) -> Result<Vec<VectorChunkHit>, DocumentError> {
        sqlx::query_as::<_, VectorHitRow>(VECTOR_SEARCH)
            .bind(search.owner_id())
            .bind(search.embedding())
            .bind(search.limit())
            .fetch_all(self.database.pool())
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|row| VectorChunkHit {
                        chunk: row.chunk.into(),
                        distance: row.distance,
                    })
                    .collect()
            })
            .map_err(|error| storage_error("documents_vector_search", error))
    }

    async fn search_keyword(
        &self,
        search: &KeywordSearch,
    ) -> Result<Vec<KeywordChunkHit>, DocumentError> {
        sqlx::query_as::<_, KeywordHitRow>(KEYWORD_SEARCH)
            .bind(search.owner_id())
            .bind(search.text())
            .bind(search.limit())
            .fetch_all(self.database.pool())
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|row| KeywordChunkHit {
                        chunk: row.chunk.into(),
                        score: row.score,
                    })
                    .collect()
            })
            .map_err(|error| storage_error("documents_keyword_search", error))
    }
}

fn storage_error(operation: &'static str, error: sqlx::Error) -> DocumentError {
    crate::diagnostics::log_database_error(operation, &error);
    DocumentError::Internal
}

#[derive(FromRow)]
struct ChunkRow {
    chunk_id: String,
    file_id: String,
    version_id: String,
    display_name: String,
    chunk_index: i64,
    text: String,
    chunk_sha256: String,
    token_count: i64,
    page_start: Option<i64>,
    page_end: Option<i64>,
    char_start: i64,
    char_end: i64,
    section_path: String,
    created_at: DateTime<Utc>,
}

impl From<ChunkRow> for ChunkRecord {
    fn from(row: ChunkRow) -> Self {
        Self {
            chunk_id: row.chunk_id,
            file_id: row.file_id,
            version_id: row.version_id,
            display_name: row.display_name,
            chunk_index: row.chunk_index,
            text: row.text,
            chunk_sha256: row.chunk_sha256,
            token_count: row.token_count,
            page_start: row.page_start,
            page_end: row.page_end,
            char_start: row.char_start,
            char_end: row.char_end,
            section_path: row.section_path,
            created_at: row.created_at,
        }
    }
}

#[derive(FromRow)]
struct VectorHitRow {
    #[sqlx(flatten)]
    chunk: ChunkRow,
    distance: f64,
}

#[derive(FromRow)]
struct KeywordHitRow {
    #[sqlx(flatten)]
    chunk: ChunkRow,
    score: f64,
}
