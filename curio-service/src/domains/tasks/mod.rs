//! User-owned tasks, hierarchy, tags, comments and opaque links.
pub(crate) mod filter;
pub(crate) mod handler;
pub(crate) mod model;
pub(crate) mod repository;
pub(crate) mod route;
pub(crate) mod service;

#[cfg(test)]
#[path = "../../../tests/unit/domains/tasks/model.rs"]
mod tests;
