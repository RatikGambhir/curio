//! Black-box integration suite for the documents domain.
//!
//! Only the crate's public surface is used: the `documents::formats` parsers
//! and the full HTTP router. Database-backed tests need
//! `CURIO_TEST_DATABASE_URL` and return early (reported as passing) without it.
#[path = "documents/access.rs"]
mod access;
#[path = "documents/formats.rs"]
mod formats;
#[path = "documents/ingestion.rs"]
mod ingestion;
#[path = "documents/jobs.rs"]
mod jobs;
#[path = "documents/library.rs"]
mod library;
#[path = "support/postgres.rs"]
mod postgres;
#[path = "documents/search.rs"]
mod search;
#[path = "documents/support.rs"]
mod support;
