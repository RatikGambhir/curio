use reqwest::Url;

pub(crate) struct ExternalLink(Url);

impl ExternalLink {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        let url =
            Url::parse(value).map_err(|_| "The external link is not a valid URL.".to_owned())?;
        if url.scheme() != "https" {
            return Err("The desktop app can only open secure HTTPS links.".to_owned());
        }
        if !url.has_host() {
            return Err("The external link must include a host.".to_owned());
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err("External links cannot include credentials.".to_owned());
        }
        Ok(Self(url))
    }

    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/core/external_link.rs"]
mod tests;
