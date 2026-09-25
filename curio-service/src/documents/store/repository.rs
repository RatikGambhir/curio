//! PostgreSQL adapter for the stored-file aggregate.
//!
//! One transaction writes the file, its content version, the version bytes,
//! and the version's chunks, so a document is never stored without its index.
//! Single-table writes use the shared query builders; joins, row locks, and
//! casts use explicit SQL with bound parameters.
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection};

use super::domain::{
    DocumentGraph, DocumentSummary, FilePersistence, PersistedFileIdentity, StoredDocumentBlob,
};
use crate::{
    database::Database,
    documents::{
        domain::DocumentError, index::repository::replace_version_chunks,
        ingestion::service::IngestionStore, viewing::service::StoredDocuments,
    },
    query::{Comparison, Expression, FieldWriter, InsertQuery, SqlColumn, SqlField, UpdateQuery},
};

#[derive(Clone)]
pub struct PostgresDocumentStore {
    database: Database,
}

impl PostgresDocumentStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

const FIND_CURRENT_BY_HASH: &str = r#"
SELECT file.id AS file_id, file.owner_id, file.display_name, version.id AS version_id
FROM document_files AS file
JOIN document_file_versions AS version ON version.file_id = file.id
WHERE file.owner_id = $1
  AND file.deleted_at IS NULL
  AND version.is_current
  AND version.content_sha256 = $2
"#;

const LOCK_FILE: &str = r#"
SELECT owner_id, deleted_at IS NOT NULL AS deleted
FROM document_files
WHERE id = $1
FOR UPDATE
"#;

const UPSERT_FILE: &str = r#"
INSERT INTO document_files (id, owner_id, display_name, source_uri, metadata)
VALUES ($1, $2, $3, $4, $5::jsonb)
ON CONFLICT (id) DO UPDATE SET
    display_name = EXCLUDED.display_name,
    source_uri = EXCLUDED.source_uri,
    metadata = EXCLUDED.metadata,
    updated_at = CURRENT_TIMESTAMP
"#;

const FIND_VERSION: &str = r#"
SELECT id, byte_size, is_current
FROM document_file_versions
WHERE file_id = $1 AND content_sha256 = $2
"#;

const BLOB_MATCHES: &str = r#"
SELECT file_bytes = $2 FROM document_file_blobs WHERE version_id = $1
"#;

const NEXT_VERSION_NUMBER: &str = r#"
SELECT COALESCE(MAX(version_number), 0) + 1
FROM document_file_versions
WHERE file_id = $1
"#;

const READ_BACK_IDENTITY: &str = r#"
SELECT file.id AS file_id, file.owner_id, file.display_name, version.id AS version_id
FROM document_files AS file
JOIN document_file_versions AS version ON version.file_id = file.id
WHERE file.id = $1 AND version.id = $2 AND version.is_current
"#;

const LIST_CURRENT: &str = r#"
SELECT
    file.id AS file_id,
    file.display_name,
    version.mime_type,
    version.byte_size,
    version.id AS version_id,
    version.version_number,
    file.created_at,
    file.updated_at
FROM document_files AS file
JOIN document_file_versions AS version
  ON version.file_id = file.id AND version.is_current
WHERE file.owner_id = $1 AND file.deleted_at IS NULL
ORDER BY file.display_name ASC, file.id ASC
"#;

const CURRENT_BLOB: &str = r#"
SELECT file.id AS file_id, file.display_name, version.mime_type, blob.file_bytes
FROM document_files AS file
JOIN document_file_versions AS version
  ON version.file_id = file.id AND version.is_current
JOIN document_file_blobs AS blob ON blob.version_id = version.id
WHERE file.owner_id = $1 AND file.id = $2 AND file.deleted_at IS NULL
"#;

const SOFT_DELETE: &str = r#"
UPDATE document_files
SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
WHERE id = $1 AND owner_id = $2 AND deleted_at IS NULL
"#;

impl IngestionStore for PostgresDocumentStore {
    async fn find_current_by_hash(
        &self,
        owner_id: &str,
        content_sha256: &str,
    ) -> Result<Option<PersistedFileIdentity>, DocumentError> {
        let rows = sqlx::query_as::<_, IdentityRow>(FIND_CURRENT_BY_HASH)
            .bind(owner_id)
            .bind(content_sha256)
            .fetch_all(self.database.pool())
            .await
            .map_err(|error| storage_error("documents_find_by_hash", error))?;
        if rows.len() > 1 {
            return Err(DocumentError::Conflict(
                "More than one current document has this content.".to_owned(),
            ));
        }
        Ok(rows.into_iter().next().map(Into::into))
    }

    async fn persist(&self, graph: DocumentGraph) -> Result<PersistedFileIdentity, DocumentError> {
        persist_graph(&self.database, &graph)
            .await
            .map_err(|error| match error {
                PersistError::Rejected(error) => error,
                PersistError::Storage(error) => storage_error("documents_persist", error),
            })
    }
}

impl StoredDocuments for PostgresDocumentStore {
    async fn list(&self, owner_id: &str) -> Result<Vec<DocumentSummary>, DocumentError> {
        sqlx::query_as::<_, SummaryRow>(LIST_CURRENT)
            .bind(owner_id)
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(|error| storage_error("documents_list", error))
    }

    async fn current_blob(
        &self,
        owner_id: &str,
        file_id: &str,
    ) -> Result<Option<StoredDocumentBlob>, DocumentError> {
        sqlx::query_as::<_, BlobRow>(CURRENT_BLOB)
            .bind(owner_id)
            .bind(file_id)
            .fetch_optional(self.database.pool())
            .await
            .map(|row| row.map(Into::into))
            .map_err(|error| storage_error("documents_current_blob", error))
    }

    async fn soft_delete(&self, owner_id: &str, file_id: &str) -> Result<bool, DocumentError> {
        sqlx::query(SOFT_DELETE)
            .bind(file_id)
            .bind(owner_id)
            .execute(self.database.pool())
            .await
            .map(|result| result.rows_affected() == 1)
            .map_err(|error| storage_error("documents_soft_delete", error))
    }
}

enum PersistError {
    /// A domain rule rejected the write; the transaction rolls back.
    Rejected(DocumentError),
    Storage(sqlx::Error),
}

impl From<sqlx::Error> for PersistError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(error)
    }
}

fn rejected(message: impl Into<String>) -> PersistError {
    PersistError::Rejected(DocumentError::Conflict(message.into()))
}

fn invariant(reason: &'static str) -> PersistError {
    tracing::error!(reason, "stored document failed an aggregate invariant");
    PersistError::Rejected(DocumentError::Internal)
}

/// The aggregate is always mutated parent-to-child inside one transaction.
async fn persist_graph(
    database: &Database,
    graph: &DocumentGraph,
) -> Result<PersistedFileIdentity, PersistError> {
    let file = &graph.file;
    let mut transaction = database.pool().begin().await?;

    validate_existing_file(&mut transaction, file).await?;
    upsert_file(&mut transaction, file).await?;

    if let Some(existing) = find_existing_version(&mut transaction, file).await? {
        verify_idempotent_version_and_blob(&mut transaction, file, &existing).await?;
        if !existing.is_current {
            clear_current_version(&mut transaction, file).await?;
            set_current_version(&mut transaction, file).await?;
        }
    } else {
        let version_number = next_version_number(&mut transaction, file).await?;
        clear_current_version(&mut transaction, file).await?;
        insert_version(&mut transaction, file, version_number).await?;
        insert_blob(&mut transaction, file).await?;
    }

    replace_version_chunks(&mut transaction, &file.version_id, &graph.chunks).await?;
    mark_version_indexed(&mut transaction, file).await?;
    let identity = read_back_identity(&mut transaction, file).await?;
    transaction.commit().await?;
    Ok(identity)
}

async fn validate_existing_file(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<(), PersistError> {
    let Some(existing) = sqlx::query_as::<_, ExistingFileRow>(LOCK_FILE)
        .bind(&file.file_id)
        .fetch_optional(&mut *connection)
        .await?
    else {
        return Ok(());
    };
    if existing.owner_id != file.owner_id {
        return Err(rejected("That file belongs to another owner."));
    }
    if existing.deleted {
        return Err(rejected(
            "That file was deleted and cannot be restored by an upload.",
        ));
    }
    Ok(())
}

async fn upsert_file(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<(), PersistError> {
    let result = sqlx::query(UPSERT_FILE)
        .bind(&file.file_id)
        .bind(&file.owner_id)
        .bind(&file.display_name)
        .bind(file.source_uri.as_deref())
        .bind(&file.metadata_json)
        .execute(&mut *connection)
        .await?;
    require_one_row(result.rows_affected(), "document file upsert")
}

async fn find_existing_version(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<Option<ExistingVersionRow>, PersistError> {
    Ok(sqlx::query_as::<_, ExistingVersionRow>(FIND_VERSION)
        .bind(&file.file_id)
        .bind(&file.content_sha256)
        .fetch_optional(&mut *connection)
        .await?)
}

async fn verify_idempotent_version_and_blob(
    connection: &mut PgConnection,
    file: &FilePersistence,
    existing: &ExistingVersionRow,
) -> Result<(), PersistError> {
    if existing.id != file.version_id || existing.byte_size != file.byte_size {
        return Err(invariant(
            "stored version does not match its derived identity or byte size",
        ));
    }
    let matches = sqlx::query_scalar::<_, bool>(BLOB_MATCHES)
        .bind(&existing.id)
        .bind(&file.file_bytes)
        .fetch_optional(&mut *connection)
        .await?;
    match matches {
        Some(true) => Ok(()),
        Some(false) => Err(invariant("stored blob is corrupt or collided")),
        None => Err(invariant("stored version has no blob")),
    }
}

async fn next_version_number(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<i64, PersistError> {
    Ok(sqlx::query_scalar::<_, i64>(NEXT_VERSION_NUMBER)
        .bind(&file.file_id)
        .fetch_one(&mut *connection)
        .await?)
}

async fn clear_current_version(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<(), PersistError> {
    let result = UpdateQuery::new("document_file_versions")
        .value(VersionField::IsCurrent(false))
        .filter(VersionColumn::FileId, Comparison::Equal, &file.file_id)
        .filter(VersionColumn::IsCurrent, Comparison::Equal, true)
        .build()?
        .build()
        .execute(&mut *connection)
        .await?;
    if result.rows_affected() > 1 {
        return Err(invariant("file had more than one current version"));
    }
    Ok(())
}

async fn set_current_version(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<(), PersistError> {
    let result = UpdateQuery::new("document_file_versions")
        .value(VersionField::IsCurrent(true))
        .filter(VersionColumn::Id, Comparison::Equal, &file.version_id)
        .filter(VersionColumn::FileId, Comparison::Equal, &file.file_id)
        .build()?
        .build()
        .execute(&mut *connection)
        .await?;
    require_one_row(result.rows_affected(), "document version current update")
}

async fn insert_version(
    connection: &mut PgConnection,
    file: &FilePersistence,
    version_number: i64,
) -> Result<(), PersistError> {
    let result = InsertQuery::new("document_file_versions")
        .value(VersionField::Id(&file.version_id))
        .value(VersionField::FileId(&file.file_id))
        .value(VersionField::VersionNumber(version_number))
        .value(VersionField::OriginalFilename(&file.display_name))
        .value(VersionField::MimeType(&file.mime_type))
        .value(VersionField::ContentSha256(&file.content_sha256))
        .value(VersionField::ByteSize(file.byte_size))
        .value(VersionField::IsCurrent(true))
        .build()?
        .build()
        .execute(&mut *connection)
        .await?;
    require_one_row(result.rows_affected(), "document version insert")
}

async fn insert_blob(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<(), PersistError> {
    let result = InsertQuery::new("document_file_blobs")
        .value(BlobField::VersionId(&file.version_id))
        .value(BlobField::FileBytes(&file.file_bytes))
        .build()?
        .build()
        .execute(&mut *connection)
        .await?;
    require_one_row(result.rows_affected(), "document blob insert")
}

async fn mark_version_indexed(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<(), PersistError> {
    let result = UpdateQuery::new("document_file_versions")
        .value(VersionField::IndexedAt(Expression::CurrentTimestamp))
        .filter(VersionColumn::Id, Comparison::Equal, &file.version_id)
        .build()?
        .build()
        .execute(&mut *connection)
        .await?;
    require_one_row(result.rows_affected(), "document version indexed update")
}

async fn read_back_identity(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<PersistedFileIdentity, PersistError> {
    let rows = sqlx::query_as::<_, IdentityRow>(READ_BACK_IDENTITY)
        .bind(&file.file_id)
        .bind(&file.version_id)
        .fetch_all(&mut *connection)
        .await?;
    let [row] = rows.as_slice() else {
        return Err(invariant(
            "persisted identity read-back did not return exactly one row",
        ));
    };
    if row.file_id != file.file_id
        || row.owner_id != file.owner_id
        || row.display_name != file.display_name
        || row.version_id != file.version_id
    {
        return Err(invariant(
            "persisted identity read-back did not match the validated input",
        ));
    }
    Ok(row.clone().into())
}

fn require_one_row(rows_affected: u64, operation: &'static str) -> Result<(), PersistError> {
    if rows_affected == 1 {
        Ok(())
    } else {
        tracing::error!(operation, rows_affected, "unexpected affected row count");
        Err(PersistError::Rejected(DocumentError::Internal))
    }
}

fn storage_error(operation: &'static str, error: sqlx::Error) -> DocumentError {
    if let sqlx::Error::Database(ref database_error) = error {
        if database_error.is_foreign_key_violation() {
            return DocumentError::UnknownOwner;
        }
        if database_error.is_unique_violation() {
            return DocumentError::Conflict("That document version already exists.".to_owned());
        }
    }

    crate::diagnostics::log_database_error(operation, &error);
    DocumentError::Internal
}

#[derive(Clone, FromRow)]
struct IdentityRow {
    file_id: String,
    owner_id: String,
    display_name: String,
    version_id: String,
}

impl From<IdentityRow> for PersistedFileIdentity {
    fn from(row: IdentityRow) -> Self {
        Self {
            file_id: row.file_id,
            owner_id: row.owner_id,
            display_name: row.display_name,
            version_id: row.version_id,
        }
    }
}

#[derive(FromRow)]
struct ExistingFileRow {
    owner_id: String,
    deleted: bool,
}

#[derive(FromRow)]
struct ExistingVersionRow {
    id: String,
    byte_size: i64,
    is_current: bool,
}

#[derive(FromRow)]
struct SummaryRow {
    file_id: String,
    display_name: String,
    mime_type: String,
    byte_size: i64,
    version_id: String,
    version_number: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SummaryRow> for DocumentSummary {
    fn from(row: SummaryRow) -> Self {
        Self {
            file_id: row.file_id,
            display_name: row.display_name,
            mime_type: row.mime_type,
            byte_size: row.byte_size,
            version_id: row.version_id,
            version_number: row.version_number,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(FromRow)]
struct BlobRow {
    file_id: String,
    display_name: String,
    mime_type: String,
    file_bytes: Vec<u8>,
}

impl From<BlobRow> for StoredDocumentBlob {
    fn from(row: BlobRow) -> Self {
        Self {
            file_id: row.file_id,
            display_name: row.display_name,
            mime_type: row.mime_type,
            file_bytes: row.file_bytes,
        }
    }
}

#[derive(Clone, Copy)]
enum VersionColumn {
    Id,
    FileId,
    VersionNumber,
    OriginalFilename,
    MimeType,
    ContentSha256,
    ByteSize,
    IsCurrent,
    IndexedAt,
}
impl SqlColumn for VersionColumn {
    fn sql(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::FileId => "file_id",
            Self::VersionNumber => "version_number",
            Self::OriginalFilename => "original_filename",
            Self::MimeType => "mime_type",
            Self::ContentSha256 => "content_sha256",
            Self::ByteSize => "byte_size",
            Self::IsCurrent => "is_current",
            Self::IndexedAt => "indexed_at",
        }
    }
}

enum VersionField<'a> {
    Id(&'a str),
    FileId(&'a str),
    VersionNumber(i64),
    OriginalFilename(&'a str),
    MimeType(&'a str),
    ContentSha256(&'a str),
    ByteSize(i64),
    IsCurrent(bool),
    IndexedAt(Expression),
}
impl SqlField for VersionField<'_> {
    type Column = VersionColumn;
    fn write(self, writer: &mut impl FieldWriter<VersionColumn>) {
        match self {
            Self::Id(value) => writer.bind(VersionColumn::Id, value),
            Self::FileId(value) => writer.bind(VersionColumn::FileId, value),
            Self::VersionNumber(value) => writer.bind(VersionColumn::VersionNumber, value),
            Self::OriginalFilename(value) => writer.bind(VersionColumn::OriginalFilename, value),
            Self::MimeType(value) => writer.bind(VersionColumn::MimeType, value),
            Self::ContentSha256(value) => writer.bind(VersionColumn::ContentSha256, value),
            Self::ByteSize(value) => writer.bind(VersionColumn::ByteSize, value),
            Self::IsCurrent(value) => writer.bind(VersionColumn::IsCurrent, value),
            Self::IndexedAt(expression) => writer.expression(VersionColumn::IndexedAt, expression),
        }
    }
}

#[derive(Clone, Copy)]
enum BlobColumn {
    VersionId,
    FileBytes,
}
impl SqlColumn for BlobColumn {
    fn sql(self) -> &'static str {
        match self {
            Self::VersionId => "version_id",
            Self::FileBytes => "file_bytes",
        }
    }
}

enum BlobField<'a> {
    VersionId(&'a str),
    FileBytes(&'a [u8]),
}
impl SqlField for BlobField<'_> {
    type Column = BlobColumn;
    fn write(self, writer: &mut impl FieldWriter<BlobColumn>) {
        match self {
            Self::VersionId(value) => writer.bind(BlobColumn::VersionId, value),
            Self::FileBytes(value) => writer.bind(BlobColumn::FileBytes, value),
        }
    }
}
