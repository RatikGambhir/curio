use axum::{Router, middleware, routing::post};

use crate::{
    calendar::{handler, repository::CalendarRepository, service::CalendarService},
    database::Database,
};

pub fn routes(database: Database) -> Router {
    let service = CalendarService::new(CalendarRepository::new(database));
    let events = Router::new().route(
        "/events",
        post(handler::create_event).get(handler::list_events),
    );

    Router::new()
        .nest("/v1/calendar", events)
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(service)
}
