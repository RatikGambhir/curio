use std::sync::Arc;

use axum::{Router, routing::post};

use super::{Service, handler};

pub(crate) fn routes(calendar: Arc<Service>) -> Router {
    let events = Router::new().route(
        "/events",
        post(handler::create_event).get(handler::list_events),
    );

    Router::new()
        .nest("/v1/calendar", events)
        .with_state(calendar)
}
