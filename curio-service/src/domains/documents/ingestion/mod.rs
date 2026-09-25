//! Ingestion subdomain: upload handling, parsing, embedding, and aggregate writes.
pub(crate) mod handler;
pub(crate) mod jobs;
pub(crate) mod model;
pub(crate) mod persistence;
pub(crate) mod route;
pub(crate) mod service;
#[cfg(test)]
#[path = "../../../../tests/unit/domains/documents/ingestion/support.rs"]
pub(crate) mod test_support;
