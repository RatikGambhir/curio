//! PostgreSQL adapter for the chunk index.
//!
//! Embeddings are stored as `real[]` and ranked by exact cosine distance in
//! SQL, so no database extension is required; this scans the owner's current
//! chunks and is intended for modest per-owner corpora. Keyword search uses
//! the built-in English full-text configuration over a generated `tsvector`.
use crate::adapters::postgres::query::{
    Delete, Expr, Function, Insert, Select, SqlColumn, SqlType, call, int, rows_from, sql_columns,
    table, val,
};
use crate::domains::documents::store::repository::{FileColumn, VersionColumn};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection};

use super::model::{
    ChunkRecord, IndexedChunk, KeywordChunkHit, KeywordSearch, VectorChunkHit, VectorSearch,
};
use crate::{
    adapters::postgres::client::Database,
    domains::documents::{
        model::DocumentError, search::service::ChunkSearch, viewing::service::DocumentChunks,
    },
};

/// 14 binds per row keeps each statement far below PostgreSQL's bind limit.
const CHUNK_INSERT_BATCH: usize = 500;

sql_columns! { enum ChunkColumn {
    Id => "id", OwnerId => "owner_id", FileId => "file_id", VersionId => "version_id",
    ChunkIndex => "chunk_index", Text => "text", Embedding => "embedding", ChunkSha256 => "chunk_sha256",
    TokenCount => "token_count", PageStart => "page_start", PageEnd => "page_end",
    CharStart => "char_start", CharEnd => "char_end", SectionPath => "section_path", CreatedAt => "created_at", TextSearch => "text_search"
} }
sql_columns! { enum DerivedColumn { ChunkId => "chunk_id", Distance => "distance", Score => "score", Query => "query", StoredValue => "stored_value", QueryValue => "query_value" } }

fn current_chunks<'a>(owner: &'a str) -> Select<'a> {
    Select::from(table("document_chunks").alias("chunk"))
        .columns([
            ChunkColumn::Id.of("chunk").alias(DerivedColumn::ChunkId),
            ChunkColumn::FileId.of("chunk"),
            ChunkColumn::VersionId.of("chunk"),
            FileColumn::DisplayName.of("file"),
            ChunkColumn::ChunkIndex.of("chunk"),
            ChunkColumn::Text.of("chunk"),
            ChunkColumn::ChunkSha256.of("chunk"),
            ChunkColumn::TokenCount.of("chunk"),
            ChunkColumn::PageStart.of("chunk"),
            ChunkColumn::PageEnd.of("chunk"),
            ChunkColumn::CharStart.of("chunk"),
            ChunkColumn::CharEnd.of("chunk"),
            ChunkColumn::SectionPath.of("chunk"),
            ChunkColumn::CreatedAt.of("chunk"),
        ])
        .join(
            table("document_file_versions").alias("version"),
            VersionColumn::Id
                .of("version")
                .eq(ChunkColumn::VersionId.of("chunk"))
                .and(VersionColumn::IsCurrent.of("version")),
        )
        .join(
            table("document_files").alias("file"),
            FileColumn::Id
                .of("file")
                .eq(ChunkColumn::FileId.of("chunk"))
                .and(FileColumn::DeletedAt.of("file").is_null()),
        )
        .where_(ChunkColumn::OwnerId.of("chunk").eq(val(owner)))
}
fn pair_value<'a>(column: DerivedColumn) -> Expr<'a> {
    column.of("pair").cast(SqlType::Float8)
}
fn norm<'a>(column: DerivedColumn) -> Expr<'a> {
    call(
        Function::Sqrt,
        [call(
            Function::Sum,
            [pair_value(column).times(pair_value(column))],
        )],
    )
}
fn vector_query<'a>(search: &'a VectorSearch) -> Select<'a> {
    let similarity = Select::from(
        rows_from(call(
            Function::Unnest,
            [
                ChunkColumn::Embedding.of("chunk"),
                val(search.embedding()).cast(SqlType::RealArray),
            ],
        ))
        .alias("pair")
        .columns([DerivedColumn::StoredValue, DerivedColumn::QueryValue]),
    )
    .columns([int(1)
        .minus(
            call(
                Function::Sum,
                [pair_value(DerivedColumn::StoredValue)
                    .times(pair_value(DerivedColumn::QueryValue))],
            )
            .divided_by(call(
                Function::NullIf,
                [
                    norm(DerivedColumn::StoredValue).times(norm(DerivedColumn::QueryValue)),
                    int(0),
                ],
            )),
        )
        .alias(DerivedColumn::Distance)]);
    current_chunks(search.owner_id())
        // Filter NULL before clamping because GREATEST/LEAST ignore NULL.
        .columns([call(
            Function::Greatest,
            [
                int(0),
                call(
                    Function::Least,
                    [int(2), DerivedColumn::Distance.of("similarity")],
                ),
            ],
        )
        .alias(DerivedColumn::Distance)])
        .cross_join(similarity.lateral("similarity"))
        .where_(
            call(Function::Cardinality, [ChunkColumn::Embedding.of("chunk")]).eq(call(
                Function::Cardinality,
                [val(search.embedding()).cast(SqlType::RealArray)],
            )),
        )
        .where_(DerivedColumn::Distance.of("similarity").is_not_null())
        .order_by(DerivedColumn::Distance.of("similarity").asc())
        .order_by(ChunkColumn::Id.of("chunk").asc())
        .limit(val(search.limit()))
}
fn keyword_query<'a>(search: &'a KeywordSearch) -> Select<'a> {
    current_chunks(search.owner_id())
        .columns([call(
            Function::TsRankCd,
            [
                ChunkColumn::TextSearch.of("chunk"),
                DerivedColumn::Query.of("search"),
            ],
        )
        .cast(SqlType::Float8)
        .alias(DerivedColumn::Score)])
        .cross_join(
            rows_from(call(
                Function::WebsearchToTsquery,
                [val("english").cast(SqlType::Regconfig), val(search.text())],
            ))
            .alias("search")
            .columns([DerivedColumn::Query]),
        )
        .where_(
            ChunkColumn::TextSearch
                .of("chunk")
                .matches_text(DerivedColumn::Query.of("search")),
        )
        .order_by(DerivedColumn::Score.desc())
        .order_by(ChunkColumn::Id.of("chunk").asc())
        .limit(val(search.limit()))
}

/// Replaces a version's chunks inside the caller's aggregate transaction.
pub(crate) async fn replace_version_chunks(
    connection: &mut PgConnection,
    version_id: &str,
    chunks: &[IndexedChunk],
) -> Result<(), sqlx::Error> {
    Delete::from("document_chunks")
        .where_(ChunkColumn::VersionId.is_equal_to(version_id))
        .build()?
        .build()
        .execute(&mut *connection)
        .await?;

    for batch in chunks.chunks(CHUNK_INSERT_BATCH) {
        let result = Insert::rows(
            "document_chunks",
            [
                ChunkColumn::Id,
                ChunkColumn::OwnerId,
                ChunkColumn::FileId,
                ChunkColumn::VersionId,
                ChunkColumn::ChunkIndex,
                ChunkColumn::Text,
                ChunkColumn::Embedding,
                ChunkColumn::ChunkSha256,
                ChunkColumn::TokenCount,
                ChunkColumn::PageStart,
                ChunkColumn::PageEnd,
                ChunkColumn::CharStart,
                ChunkColumn::CharEnd,
                ChunkColumn::SectionPath,
            ],
            batch.iter().map(|chunk| {
                vec![
                    val(&chunk.chunk_id),
                    val(&chunk.owner_id),
                    val(&chunk.file_id),
                    val(&chunk.version_id),
                    val(chunk.chunk_index),
                    val(&chunk.text),
                    val(chunk.embedding.as_slice()),
                    val(&chunk.chunk_sha256),
                    val(chunk.token_count),
                    val(chunk.page_start),
                    val(chunk.page_end),
                    val(chunk.char_start),
                    val(chunk.char_end),
                    val(&chunk.section_path),
                ]
            }),
        )
        .build()?
        .build()
        .execute(&mut *connection)
        .await?;
        if result.rows_affected() != batch.len() as u64 {
            return Err(sqlx::Error::Protocol(
                "document chunk insert affected an unexpected row count".to_owned(),
            ));
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct ChunkIndex {
    database: Database,
}

impl ChunkIndex {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl DocumentChunks for ChunkIndex {
    async fn current_chunks(
        &self,
        owner_id: &str,
        file_id: &str,
    ) -> Result<Option<Vec<ChunkRecord>>, DocumentError> {
        let visible = Select::expressions([Select::from("document_files")
            .columns([int(1)])
            .where_(FileColumn::Id.is_equal_to(file_id))
            .where_(FileColumn::OwnerId.is_equal_to(owner_id))
            .where_(FileColumn::DeletedAt.is_null())
            .exists()])
        .build()
        .map_err(|error| storage_error("documents_chunk_owner", error))?
        .build_query_scalar::<bool>()
        .fetch_one(self.database.pool())
        .await
        .map_err(|error| storage_error("documents_chunk_owner", error))?;
        if !visible {
            return Ok(None);
        }
        current_chunks(owner_id)
            .where_(ChunkColumn::FileId.of("chunk").eq(val(file_id)))
            .order_by(ChunkColumn::ChunkIndex.of("chunk").asc())
            .build()
            .map_err(|error| storage_error("documents_current_chunks", error))?
            .build_query_as::<ChunkRow>()
            .fetch_all(self.database.pool())
            .await
            .map(|rows| Some(rows.into_iter().map(Into::into).collect()))
            .map_err(|error| storage_error("documents_current_chunks", error))
    }
}

impl ChunkSearch for ChunkIndex {
    async fn search_vector(
        &self,
        search: &VectorSearch,
    ) -> Result<Vec<VectorChunkHit>, DocumentError> {
        vector_query(search)
            .build()
            .map_err(|error| storage_error("documents_vector_search", error))?
            .build_query_as::<VectorHitRow>()
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
        keyword_query(search)
            .build()
            .map_err(|error| storage_error("documents_keyword_search", error))?
            .build_query_as::<KeywordHitRow>()
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
    crate::shared::diagnostics::log_database_error(operation, &error);
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
