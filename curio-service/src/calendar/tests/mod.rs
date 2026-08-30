//! Unit tests for the calendar's internals.
//!
//! These live inside the crate because they reach private module items — the
//! normalization functions and the service's rules. The endpoint behavior they
//! back up is tested from the outside in `curio-service/tests/`.

mod service;
mod time;
