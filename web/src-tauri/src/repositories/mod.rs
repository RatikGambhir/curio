//! Infrastructure adapters for remote data and local request state.

mod request_registry;
mod service_repository;

pub(crate) use request_registry::RequestRegistry;
pub(crate) use service_repository::ServiceRepository;
