//! User-owned standalone Spaces. Resource membership is not implemented yet.
pub(crate) mod handler;
pub(crate) mod model;
pub(crate) mod repository;
pub(crate) mod route;
pub(crate) mod service;

pub(crate) type Service = service::SpaceService<repository::SpaceRepository>;

#[cfg(test)]
#[path = "../../../tests/unit/domains/spaces/model.rs"]
mod tests;
