//! Calendar events, split by responsibility:
//!
//! | Layer | File | Knows about |
//! | -- | -- | -- |
//! | Handlers | `handlers.rs` | extractors, status codes, JSON |
//! | Service | `service.rs` | validation, span caps, what a failure means |
//! | Repository | `repository.rs` | SQL and row mapping |
//! | Models | `models.rs` | the types crossing those seams |
//!
//! `time.rs` is pure date-format logic the service leans on, and `error.rs`
//! carries the one error type plus its rendering. The dependencies only point
//! downward: the service never names an HTTP type, and the repository never
//! decides what a constraint violation means.

mod error;
mod handlers;
mod models;
mod repository;
mod service;
#[cfg(test)]
mod tests;
mod time;

use axum::{Router, middleware, routing::post};

use crate::{
    calendar::{repository::CalendarRepository, service::CalendarService},
    database::Database,
};

/// Authenticated calendar API routes. This is where the layers are stacked:
/// a repository over the pool, a service over the repository, and handlers over
/// the service — assembled once at startup rather than per request.
pub fn api_routes(database: Database) -> Router {
    let service = CalendarService::new(CalendarRepository::new(database.pool().clone()));

    let events = Router::new().route(
        "/events",
        post(handlers::create_event).get(handlers::list_events),
    );

    Router::new()
        .nest("/v1/calendar", events)
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(service)
}
