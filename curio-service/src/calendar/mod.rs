mod commands;
mod error;
mod queries;
mod time;

use axum::{Router, middleware, routing::post};
use serde::Deserialize;

use crate::database::Database;

/// The views the client can ask for. Deserializing `view` as an enum means an
/// unknown value fails in the `Query` extractor and returns `400` before the
/// handler runs, so the handler only ever sees one of these.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CalendarView {
    Day,
    Week,
    Month,
    Agenda,
}

/// Authenticated calendar API routes backed by the database.
pub fn api_routes(database: Database) -> Router {
    let events = Router::new().route(
        "/events",
        post(commands::create_event).get(queries::list_events),
    );

    Router::new()
        .nest("/v1/calendar", events)
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(database)
}

/// The status and priority vocabularies the calendar UI uses. Kept here so the
/// storage layer never accumulates values the client cannot render.
pub(crate) const EVENT_STATUSES: [&str; 4] = ["scheduled", "confirmed", "tentative", "cancelled"];
pub(crate) const EVENT_PRIORITIES: [&str; 3] = ["low", "medium", "high"];
