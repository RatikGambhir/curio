use crate::core::ExternalLink;

pub(crate) struct ExternalLinkService;

impl ExternalLinkService {
    pub(crate) fn validate(&self, value: &str) -> Result<ExternalLink, String> {
        ExternalLink::parse(value)
    }
}
