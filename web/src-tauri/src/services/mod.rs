//! Application use cases. This layer coordinates core rules and repositories.

mod external_link_service;
mod service_proxy;

use crate::repositories::{RequestRegistry, ServiceRepository};

pub(crate) use external_link_service::ExternalLinkService;
pub(crate) use service_proxy::{ServiceEventSink, ServiceProxy};

pub(crate) struct AppServices {
    pub(crate) service_proxy: ServiceProxy,
    pub(crate) external_links: ExternalLinkService,
}

impl AppServices {
    pub(crate) fn new(service_url: &str) -> Result<Self, String> {
        Ok(Self {
            service_proxy: ServiceProxy::new(
                ServiceRepository::new(service_url)?,
                RequestRegistry::default(),
            ),
            external_links: ExternalLinkService,
        })
    }
}
