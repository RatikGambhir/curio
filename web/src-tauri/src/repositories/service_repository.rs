use std::time::Duration;

use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client, Method, Request, Response, Url,
};

use crate::core::{ServiceMethod, ServiceRequest};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) struct ServiceRepository {
    client: Client,
    base_url: Url,
}

impl ServiceRepository {
    pub(crate) fn new(base_url: &str) -> Result<Self, String> {
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .map_err(|_| "The desktop HTTP client could not be created.".to_owned())?;

        Ok(Self {
            client,
            base_url: normalize_base_url(base_url)?,
        })
    }

    pub(crate) fn build_request(&self, request: &ServiceRequest) -> Result<Request, String> {
        let endpoint = self.endpoint(&request.path)?;
        let mut builder = self.client.request(http_method(request.method), endpoint);
        if let Some(payload) = request.payload.as_ref() {
            builder = if request.method.is_get() {
                builder.query(payload)
            } else {
                builder.json(payload)
            };
        }
        if let Some(token) = request.bearer_token.as_deref() {
            builder = builder.header(AUTHORIZATION, bearer_header(token)?);
        }
        builder
            .build()
            .map_err(|_| "The service request payload is invalid.".to_owned())
    }

    pub(crate) async fn execute(&self, request: Request) -> Result<Response, reqwest::Error> {
        self.client.execute(request).await
    }

    fn endpoint(&self, path: &str) -> Result<Url, String> {
        if !path.starts_with('/') || path.starts_with("//") {
            return Err("Service paths must begin with one forward slash.".to_owned());
        }
        if path.contains('\\') {
            return Err("Service paths cannot include backslashes.".to_owned());
        }

        let endpoint = self
            .base_url
            .join(path)
            .map_err(|_| "The service path is invalid.".to_owned())?;
        if endpoint.origin() != self.base_url.origin() {
            return Err("The service path must stay on the configured service.".to_owned());
        }
        if endpoint.fragment().is_some() {
            return Err("Service paths cannot include fragments.".to_owned());
        }
        if !endpoint.username().is_empty() || endpoint.password().is_some() {
            return Err("Service paths cannot include credentials.".to_owned());
        }
        Ok(endpoint)
    }
}

fn normalize_base_url(value: &str) -> Result<Url, String> {
    let mut url = Url::parse(value).map_err(|_| "The Curio service URL is invalid.".to_owned())?;
    let is_local_http =
        url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    if url.scheme() != "https" && !(cfg!(debug_assertions) && is_local_http) {
        return Err("The Curio service URL must use HTTPS.".to_owned());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("The Curio service URL cannot include credentials.".to_owned());
    }
    url.set_path("/");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn bearer_header(token: &str) -> Result<HeaderValue, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("The bearer token cannot be empty.".to_owned());
    }
    let mut value = HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|_| "The bearer token contains invalid characters.".to_owned())?;
    value.set_sensitive(true);
    Ok(value)
}

fn http_method(method: ServiceMethod) -> Method {
    match method {
        ServiceMethod::Get => Method::GET,
        ServiceMethod::Post => Method::POST,
        ServiceMethod::Patch => Method::PATCH,
        ServiceMethod::Delete => Method::DELETE,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/repositories/service_repository.rs"]
mod tests;
