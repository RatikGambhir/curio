pub mod adapters;
pub mod app;
pub mod domains;
pub mod shared;

#[cfg(test)]
extern crate self as curio_service;
#[cfg(test)]
#[path = "../tests/support/postgres.rs"]
pub(crate) mod postgres_test_support;

pub use app::bootstrap::{app, app_with_config, app_with_database};
pub use app::config::ServiceConfig;
