//! Framework-independent application rules and data contracts.

mod external_link;
mod service_request;

pub(crate) use external_link::ExternalLink;
pub(crate) use service_request::{ServiceMethod, ServicePacket, ServiceRequest};
