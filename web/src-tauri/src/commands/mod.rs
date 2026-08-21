//! Tauri command handlers and outbound IPC events.

mod events;
mod external_link;
mod service;

pub(crate) use external_link::open_external_url;
pub(crate) use service::{cancel_request, service_request};
