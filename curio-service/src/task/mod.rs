mod error;
mod handler;
mod model;
mod repository;
mod route;
mod service;
#[cfg(test)]
#[path = "../../tests/unit/task/mod.rs"]
mod tests;

pub use route::routes;
