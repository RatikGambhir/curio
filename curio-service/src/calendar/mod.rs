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

pub fn api_routes(database: Database) -> Router {
    let service = CalendarService::new(CalendarRepository::new(database));
    let events = Router::new().route(
        "/events",
        post(handlers::create_event).get(handlers::list_events),
    );

    Router::new()
        .nest("/v1/calendar", events)
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(service)
}
