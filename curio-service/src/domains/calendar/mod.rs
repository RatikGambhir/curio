//! Calendar domain: date-only/timed/milestone events and range queries.
pub(crate) mod handler;
pub(crate) mod model;
pub(crate) mod repository;
pub(crate) mod route;
pub(crate) mod service;
#[cfg(test)]
#[path = "../../../tests/unit/domains/calendar/mod.rs"]
mod tests;
pub(crate) mod time;

use self::{repository::CalendarRepository, service::CalendarService};

pub(crate) type Service = CalendarService<CalendarRepository>;
