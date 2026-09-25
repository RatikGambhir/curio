//! Calendar composition root: domain use cases plus PostgreSQL and HTTP adapters.
mod domain;
mod handlers;
mod repository;
mod service;
#[cfg(test)]
#[path = "../../tests/unit/calendar/mod.rs"]
mod tests;
mod time;

use axum::{Router, middleware, routing::post};

use self::{repository::PostgresCalendarRepository, service::CalendarService};
use crate::database::Database;

type Service = CalendarService<PostgresCalendarRepository>;

pub(crate) fn router(database: Database) -> Router {
    let service = CalendarService::new(PostgresCalendarRepository::new(database));
    let events = Router::new().route(
        "/events",
        post(handlers::create_event).get(handlers::list_events),
    );

    Router::new()
        .nest("/v1/calendar", events)
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(service)
}
