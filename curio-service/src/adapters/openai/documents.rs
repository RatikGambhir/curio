//! OpenAI documents client: batched embeddings and
//! non-streaming image descriptions. Provider bodies are never forwarded to
//! clients or logs; only status, request ID, and error classification are.
use std::time::{Duration, Instant};

use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use tracing::{error, info};

/// Keeps each request well under the provider's per-request token budget for
/// chunks of up to 800 tokens.
const EMBEDDING_BATCH_SIZE: usize = 64;
const PROVIDER_TIMEOUT: Duration = Duration::from_secs(120);
const IMAGE_DESCRIPTION_INSTRUCTIONS: &str = "You are a helpful assistant.";

#[derive(Clone)]
pub struct OpenAiDocumentsClient {
    http: Client,
    api_key: String,
    base_url: String,
    embedding_model: String,
    image_description_model: String,
}

impl OpenAiDocumentsClient {
    pub fn new(
        api_key: String,
        base_url: String,
        embedding_model: String,
        image_description_model: String,
    ) -> Self {
        Self {
            http: Client::builder()
                .timeout(PROVIDER_TIMEOUT)
                .build()
                .expect("the documents HTTP client configuration is valid"),
            api_key,
            base_url: base_url.trim_end_matches('/').to_owned(),
            embedding_model,
            image_description_model,
        }
    }

    async fn embed_batch(&self, inputs: &[String]) -> Result<Vec<Vec<f32>>, String> {
        let started_at = Instant::now();
        let response = self
            .http
            .post(format!("{}/v1/embeddings", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": self.embedding_model,
                "input": inputs,
                "encoding_format": "float",
            }))
            .send()
            .await
            .map_err(|provider_error| transport_error("openai_embeddings", &provider_error))?;
        let body = read_success_body("openai_embeddings", response, started_at).await?;
        extract_embeddings(&body, inputs.len()).map_err(|reason| {
            error!(
                api = "openai_embeddings",
                reason, "invalid provider response"
            );
            "the embedding provider returned an invalid response".to_owned()
        })
    }
}

impl OpenAiDocumentsClient {
    /// Embeds non-empty inputs in provider-sized batches, preserving order.
    pub async fn embed(&self, inputs: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
        if inputs.is_empty() {
            return Err("cannot embed an empty list of document contents".to_owned());
        }
        if let Some(index) = inputs.iter().position(|input| input.trim().is_empty()) {
            return Err(format!(
                "cannot embed empty document content at input index {index}"
            ));
        }

        let mut embeddings = Vec::with_capacity(inputs.len());
        for batch in inputs.chunks(EMBEDDING_BATCH_SIZE) {
            embeddings.extend(self.embed_batch(batch).await?);
        }
        Ok(embeddings)
    }
}

impl OpenAiDocumentsClient {
    /// Produces a retrieval-oriented description of a base64-encoded image.
    pub async fn describe_image(
        &self,
        prompt: &str,
        mime_type: &str,
        data_base64: &str,
    ) -> Result<String, String> {
        let started_at = Instant::now();
        let response = self
            .http
            .post(format!("{}/v1/responses", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": self.image_description_model,
                "instructions": IMAGE_DESCRIPTION_INSTRUCTIONS,
                "input": [{
                    "role": "user",
                    "content": [
                        { "type": "input_text", "text": prompt },
                        {
                            "type": "input_image",
                            "image_url": format!("data:{mime_type};base64,{data_base64}"),
                            "detail": "auto",
                        },
                    ],
                }],
            }))
            .send()
            .await
            .map_err(|provider_error| {
                transport_error("openai_image_description", &provider_error)
            })?;
        let body = read_success_body("openai_image_description", response, started_at).await?;
        extract_response_text(&body)
            .ok_or_else(|| "the image provider returned no description".to_owned())
    }
}

async fn read_success_body(
    api: &'static str,
    response: reqwest::Response,
    started_at: Instant,
) -> Result<Value, String> {
    let status = response.status();
    let provider_request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let body = response
        .bytes()
        .await
        .map_err(|provider_error| transport_error(api, &provider_error))?;
    let elapsed_seconds = started_at.elapsed().as_secs_f64();

    if !status.is_success() {
        error!(
            api,
            status = status.as_u16(),
            provider_request_id = provider_request_id.as_deref().unwrap_or("unknown"),
            elapsed_seconds,
            "document provider returned a non-success HTTP status"
        );
        return Err(status_message(status).to_owned());
    }
    info!(
        api,
        status = status.as_u16(),
        provider_request_id = provider_request_id.as_deref().unwrap_or("unknown"),
        elapsed_seconds,
        "document provider request completed"
    );
    serde_json::from_slice(&body).map_err(|_| {
        error!(api, "document provider returned malformed JSON");
        "the provider returned an invalid response".to_owned()
    })
}

fn status_message(status: StatusCode) -> &'static str {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            "the provider rejected the service credentials"
        }
        StatusCode::TOO_MANY_REQUESTS => "the provider is rate limiting requests",
        StatusCode::BAD_REQUEST | StatusCode::PAYLOAD_TOO_LARGE => {
            "the provider rejected the request"
        }
        status if status.is_server_error() => "the provider is temporarily unavailable",
        _ => "the provider could not complete the request",
    }
}

fn transport_error(api: &'static str, provider_error: &reqwest::Error) -> String {
    error!(
        api,
        is_timeout = provider_error.is_timeout(),
        is_connect = provider_error.is_connect(),
        "document provider request failed before a response was received"
    );
    if provider_error.is_timeout() {
        "the provider timed out".to_owned()
    } else {
        "the provider is temporarily unavailable".to_owned()
    }
}

fn extract_embeddings(
    response: &Value,
    expected_count: usize,
) -> Result<Vec<Vec<f32>>, &'static str> {
    let items = response["data"]
        .as_array()
        .ok_or("embeddings response did not include data")?;
    let mut embeddings = vec![None; expected_count];

    for item in items {
        let index = item["index"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or("embeddings response included an invalid index")?;
        let destination = embeddings
            .get_mut(index)
            .ok_or("embeddings response included an out-of-range index")?;
        if destination.is_some() {
            return Err("embeddings response included a duplicate index");
        }
        let values = item["embedding"]
            .as_array()
            .ok_or("embeddings response item did not include an embedding")?;
        *destination = Some(
            values
                .iter()
                .map(|value| {
                    value
                        .as_f64()
                        .map(|value| value as f32)
                        .ok_or("embedding contained a non-number value")
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
    }

    embeddings
        .into_iter()
        .map(|embedding| embedding.ok_or("embeddings response omitted an input"))
        .collect()
}

fn extract_response_text(response: &Value) -> Option<String> {
    if let Some(text) = response.get("output_text").and_then(Value::as_str) {
        return Some(text.trim().to_owned()).filter(|text| !text.is_empty());
    }
    let mut text = String::new();
    for item in response.get("output")?.as_array()? {
        let Some(content) = item.get("content").and_then(Value::as_array) else {
            continue;
        };
        for part in content {
            if matches!(
                part.get("type").and_then(Value::as_str),
                Some("output_text" | "text")
            ) && let Some(part_text) = part.get("text").and_then(Value::as_str)
            {
                text.push_str(part_text);
            }
        }
    }
    Some(text.trim().to_owned()).filter(|text| !text.is_empty())
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/openai/documents.rs"]
mod tests;
