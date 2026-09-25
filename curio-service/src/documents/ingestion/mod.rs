//! Ingestion subdomain: upload handling, parsing, embedding, and aggregate writes.
pub(crate) mod domain;
pub(crate) mod handlers;
pub(crate) mod jobs;
pub(crate) mod persistence;
pub(crate) mod service;
#[cfg(test)]
#[path = "../../../tests/unit/documents/ingestion/support.rs"]
pub(crate) mod test_support;
