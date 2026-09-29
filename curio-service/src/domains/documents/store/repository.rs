//! PostgreSQL adapter for the stored-file aggregate.
//!
//! One transaction writes the file, its content version, the version bytes,
//! and the version's chunks, so a document is never stored without its index.
//! Reads and writes use the shared query builder, including joins and locks.
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection};

use super::model::{
    DocumentGraph, DocumentSummary, FilePersistence, PersistedFileIdentity, StoredDocumentBlob,
};
use crate::{
    adapters::postgres::client::Database,
    adapters::postgres::query::{
        Expr, Function, Insert, Select, SqlColumn, SqlField, SqlType, Update, WriteField, call,
        col, current_timestamp, int, sql_columns, table, val,
    },
    domains::documents::{
        index::repository::replace_version_chunks, ingestion::service::IngestionStore,
        model::DocumentError, viewing::service::StoredDocuments,
    },
};

#[derive(Clone)]
pub struct DocumentStore {
    database: Database,
}

impl DocumentStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

sql_columns! { pub(crate) enum FileColumn {
    Id => "id", OwnerId => "owner_id", DisplayName => "display_name", SourceUri => "source_uri",
    Metadata => "metadata", CreatedAt => "created_at", UpdatedAt => "updated_at", DeletedAt => "deleted_at"
} }
sql_columns! { enum DerivedColumn { Deleted => "deleted" } }

fn file_versions<'a>() -> Select<'a> {
    Select::from(table("document_files").alias("file")).join(
        table("document_file_versions").alias("version"),
        VersionColumn::FileId
            .of("version")
            .eq(FileColumn::Id.of("file")),
    )
}
fn identity<'a>() -> Select<'a> {
    file_versions().columns([
        FileColumn::Id.of("file").alias(VersionColumn::FileId),
        FileColumn::OwnerId.of("file"),
        FileColumn::DisplayName.of("file"),
        VersionColumn::Id.of("version").alias(BlobColumn::VersionId),
    ])
}

impl IngestionStore for DocumentStore {
    async fn find_current_by_hash(
        &self,
        owner_id: &str,
        content_sha256: &str,
    ) -> Result<Option<PersistedFileIdentity>, DocumentError> {
        let rows = identity()
            .where_(FileColumn::OwnerId.of("file").eq(val(owner_id)))
            .where_(FileColumn::DeletedAt.of("file").is_null())
            .where_(VersionColumn::IsCurrent.of("version"))
            .where_(
                VersionColumn::ContentSha256
                    .of("version")
                    .eq(val(content_sha256)),
            )
            .build()
            .map_err(|error| storage_error("documents_find_by_hash", error))?
            .build_query_as::<IdentityRow>()
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

impl StoredDocuments for DocumentStore {
    async fn list(&self, owner_id: &str) -> Result<Vec<DocumentSummary>, DocumentError> {
        file_versions()
            .columns([
                FileColumn::Id.of("file").alias(VersionColumn::FileId),
                FileColumn::DisplayName.of("file"),
                VersionColumn::MimeType.of("version"),
                VersionColumn::ByteSize.of("version"),
                VersionColumn::Id.of("version").alias(BlobColumn::VersionId),
                VersionColumn::VersionNumber.of("version"),
                FileColumn::CreatedAt.of("file"),
                FileColumn::UpdatedAt.of("file"),
            ])
            .where_(VersionColumn::IsCurrent.of("version"))
            .where_(FileColumn::OwnerId.of("file").eq(val(owner_id)))
            .where_(FileColumn::DeletedAt.of("file").is_null())
            .order_by(FileColumn::DisplayName.of("file").asc())
            .order_by(FileColumn::Id.of("file").asc())
            .build()
            .map_err(|error| storage_error("documents_list", error))?
            .build_query_as::<SummaryRow>()
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
        file_versions()
            .join(
                table("document_file_blobs").alias("blob"),
                BlobColumn::VersionId
                    .of("blob")
                    .eq(VersionColumn::Id.of("version")),
            )
            .columns([
                FileColumn::Id.of("file").alias(VersionColumn::FileId),
                FileColumn::DisplayName.of("file"),
                VersionColumn::MimeType.of("version"),
                BlobColumn::FileBytes.of("blob"),
            ])
            .where_(VersionColumn::IsCurrent.of("version"))
            .where_(FileColumn::OwnerId.of("file").eq(val(owner_id)))
            .where_(FileColumn::Id.of("file").eq(val(file_id)))
            .where_(FileColumn::DeletedAt.of("file").is_null())
            .build()
            .map_err(|error| storage_error("documents_current_blob", error))?
            .build_query_as::<BlobRow>()
            .fetch_optional(self.database.pool())
            .await
            .map(|row| row.map(Into::into))
            .map_err(|error| storage_error("documents_current_blob", error))
    }

    async fn soft_delete(&self, owner_id: &str, file_id: &str) -> Result<bool, DocumentError> {
        Update::table("document_files")
            .set(FileColumn::DeletedAt.expression(current_timestamp()))
            .set(FileColumn::UpdatedAt.expression(current_timestamp()))
            .where_(FileColumn::Id.is_equal_to(file_id))
            .where_(FileColumn::OwnerId.is_equal_to(owner_id))
            .where_(FileColumn::DeletedAt.is_null())
            .build()
            .map_err(|error| storage_error("documents_soft_delete", error))?
            .build()
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
    let Some(existing) = Select::from("document_files")
        .columns([
            col(FileColumn::OwnerId),
            FileColumn::DeletedAt
                .is_not_null()
                .alias(DerivedColumn::Deleted),
        ])
        .where_(FileColumn::Id.is_equal_to(&file.file_id))
        .for_update()
        .build()?
        .build_query_as::<ExistingFileRow>()
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
    let result = Insert::into("document_files")
        .value(FileColumn::Id.value(&file.file_id))
        .value(FileColumn::OwnerId.value(&file.owner_id))
        .value(FileColumn::DisplayName.value(&file.display_name))
        .value(FileColumn::SourceUri.value(file.source_uri.as_deref()))
        .value(FileColumn::Metadata.expression(val(&file.metadata_json).cast(SqlType::Jsonb)))
        .on_conflict(
            FileColumn::Id,
            [
                (
                    FileColumn::DisplayName,
                    FileColumn::DisplayName.of("EXCLUDED"),
                ),
                (FileColumn::SourceUri, FileColumn::SourceUri.of("EXCLUDED")),
                (FileColumn::Metadata, FileColumn::Metadata.of("EXCLUDED")),
                (FileColumn::UpdatedAt, current_timestamp()),
            ],
        )
        .build()?
        .build()
        .execute(&mut *connection)
        .await?;
    require_one_row(result.rows_affected(), "document file upsert")
}

async fn find_existing_version(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<Option<ExistingVersionRow>, PersistError> {
    Ok(Select::from("document_file_versions")
        .columns([
            VersionColumn::Id,
            VersionColumn::ByteSize,
            VersionColumn::IsCurrent,
        ])
        .where_(VersionColumn::FileId.is_equal_to(&file.file_id))
        .where_(VersionColumn::ContentSha256.is_equal_to(&file.content_sha256))
        .build()?
        .build_query_as::<ExistingVersionRow>()
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
    let matches = Select::from("document_file_blobs")
        .columns([BlobColumn::FileBytes.is_equal_to(&file.file_bytes)])
        .where_(BlobColumn::VersionId.is_equal_to(&existing.id))
        .build()?
        .build_query_scalar::<bool>()
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
    Ok(Select::from("document_file_versions")
        .columns([call(
            Function::Coalesce,
            [
                call(Function::Max, [col(VersionColumn::VersionNumber)]),
                int(0),
            ],
        )
        .plus(int(1))])
        .where_(VersionColumn::FileId.is_equal_to(&file.file_id))
        .build()?
        .build_query_scalar::<i64>()
        .fetch_one(&mut *connection)
        .await?)
}

async fn clear_current_version(
    connection: &mut PgConnection,
    file: &FilePersistence,
) -> Result<(), PersistError> {
    let result = Update::table("document_file_versions")
        .set(VersionField::IsCurrent(false))
        .where_(VersionColumn::FileId.is_equal_to(&file.file_id))
        .where_(VersionColumn::IsCurrent.is_equal_to(true))
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
    let result = Update::table("document_file_versions")
        .set(VersionField::IsCurrent(true))
        .where_(VersionColumn::Id.is_equal_to(&file.version_id))
        .where_(VersionColumn::FileId.is_equal_to(&file.file_id))
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
    let result = Insert::into("document_file_versions")
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
    let result = Insert::into("document_file_blobs")
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
    let result = Update::table("document_file_versions")
        .set(VersionField::IndexedAt(current_timestamp()))
        .where_(VersionColumn::Id.is_equal_to(&file.version_id))
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
    let rows = identity()
        .where_(FileColumn::Id.of("file").eq(val(&file.file_id)))
        .where_(VersionColumn::Id.of("version").eq(val(&file.version_id)))
        .where_(VersionColumn::IsCurrent.of("version"))
        .build()?
        .build_query_as::<IdentityRow>()
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

    crate::shared::diagnostics::log_database_error(operation, &error);
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
pub(crate) enum VersionColumn {
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
    IndexedAt(Expr<'a>),
}
impl<'a> SqlField<'a> for VersionField<'a> {
    fn into_field(self) -> WriteField<'a> {
        match self {
            Self::Id(value) => VersionColumn::Id.value(value),
            Self::FileId(value) => VersionColumn::FileId.value(value),
            Self::VersionNumber(value) => VersionColumn::VersionNumber.value(value),
            Self::OriginalFilename(value) => VersionColumn::OriginalFilename.value(value),
            Self::MimeType(value) => VersionColumn::MimeType.value(value),
            Self::ContentSha256(value) => VersionColumn::ContentSha256.value(value),
            Self::ByteSize(value) => VersionColumn::ByteSize.value(value),
            Self::IsCurrent(value) => VersionColumn::IsCurrent.value(value),
            Self::IndexedAt(expression) => VersionColumn::IndexedAt.expression(expression),
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
impl<'a> SqlField<'a> for BlobField<'a> {
    fn into_field(self) -> WriteField<'a> {
        match self {
            Self::VersionId(value) => BlobColumn::VersionId.value(value),
            Self::FileBytes(value) => BlobColumn::FileBytes.value(value),
        }
    }
}
