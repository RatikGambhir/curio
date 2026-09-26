//! Application composition root: opens adapters, builds domain services, and
//! assembles the HTTP router.
use std::{sync::Arc, time::Duration};

use axum::Router;

use crate::{
    adapters::{
        office::converter::OfficeConverter,
        openai::{client::OpenAiClient, documents::OpenAiDocumentsClient},
        postgres::client::{Database, DatabaseOptions},
    },
    app::{
        config::ServiceConfig,
        http::{create_router, protected},
    },
    domains::{
        calendar::{self, repository::CalendarRepository, service::CalendarService},
        chat::{self, repository::ChatRepository, service::ChatService},
        documents::{
            self, index::repository::ChunkIndex, ingestion::jobs::DocumentJobService,
            ingestion::service::IngestionService, search::service::SearchService,
            store::repository::DocumentStore, viewing::service::StoredDocumentService,
        },
        spaces::{self, repository::SpaceRepository, service::SpaceService},
        system,
        tasks::{self, repository::TaskRepository, service::TaskService},
        users::{self, repository::UserRepository, service::UserService},
    },
    shared::diagnostics,
};

/// Base routes only: root, health, and the legacy placeholders. Production
/// uses [`app_with_config`].
pub fn app() -> Router {
    base_routes()
}

/// Connects to PostgreSQL, verifies migrations without applying DDL, and
/// assembles the full application.
pub async fn app_with_config(config: ServiceConfig) -> Result<Router, sqlx::Error> {
    let database = Database::connect(&DatabaseOptions {
        url: config.database_url.clone(),
        schema: config.database_schema.clone(),
        max_connections: config.database_max_connections,
        acquire_timeout: Duration::from_secs(config.database_acquire_timeout_seconds),
        application_name: "curio-service".to_owned(),
    })
    .await
    .inspect_err(|error| diagnostics::log_database_error("service_database_connect", error))?;
    database
        .verify_migrations()
        .await
        .inspect_err(|error| diagnostics::log_database_error("service_migration_verify", error))?;

    Ok(app_with_database(config, database))
}

/// Full application over an already connected database.
pub fn app_with_database(config: ServiceConfig, database: Database) -> Router {
    let api = base_routes()
        .merge(system::route::readiness_routes(database.clone()))
        .merge(assemble_api(&config, database));
    create_router(api, &config.cors_allowed_origins)
}

fn base_routes() -> Router {
    system::route::routes().nest("/user", protected(users::route::legacy_routes()))
}

fn assemble_api(config: &ServiceConfig, database: Database) -> Router {
    let documents_config = &config.documents;
    let chat_client = Arc::new(OpenAiClient::new(
        config.openai_api_key.clone(),
        config.openai_base_url.clone(),
    ));
    let documents_client = OpenAiDocumentsClient::new(
        config.openai_api_key.clone(),
        config.openai_base_url.clone(),
        documents_config.embedding_model.clone(),
        documents_config.image_description_model.clone(),
    );
    let office = OfficeConverter::new(documents_config.office_executable.clone());

    let document_files = DocumentStore::new(database.clone());
    let document_chunks = ChunkIndex::new(database.clone());

    let calendar = Arc::new(CalendarService::new(CalendarRepository::new(
        database.clone(),
    )));
    let spaces = Arc::new(SpaceService::new(SpaceRepository::new(database.clone())));
    let tasks = Arc::new(TaskService::new(TaskRepository::new(database.clone())));
    let users = Arc::new(UserService::new(UserRepository::new(database.clone())));
    let chat = Arc::new(ChatService::new(
        ChatRepository::new(database),
        chat_client,
        config.openai_model.clone(),
    ));
    let document_ingestion = IngestionService::new(
        document_files.clone(),
        documents_client.clone(),
        documents_config.max_concurrent_documents,
    );
    let document_jobs = Arc::new(DocumentJobService::new(
        document_ingestion.clone(),
        Duration::from_secs(documents_config.completed_job_retention_seconds),
    ));
    let document_search = Arc::new(SearchService::new(
        document_chunks.clone(),
        documents_client,
    ));
    let stored_documents = Arc::new(StoredDocumentService::new(
        document_files,
        document_chunks,
        office,
    ));

    Router::new()
        .merge(chat::route::routes(chat))
        .merge(protected(calendar::route::routes(calendar)))
        .merge(protected(users::route::routes(users)))
        .merge(protected(spaces::route::routes(spaces)))
        .merge(protected(tasks::route::routes(tasks)))
        .merge(protected(documents::route::routes(
            Arc::new(document_ingestion),
            document_jobs,
            document_search,
            stored_documents,
        )))
}
