use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum ServiceMethod {
    Get,
    Post,
    Patch,
    Delete,
}

impl ServiceMethod {
    pub(crate) fn is_get(self) -> bool {
        self == Self::Get
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ServiceRequest {
    pub(crate) method: ServiceMethod,
    pub(crate) path: String,
    pub(crate) payload: Option<Value>,
    pub(crate) bearer_token: Option<String>,
}

impl ServiceRequest {
    pub(crate) fn new(
        method: ServiceMethod,
        path: String,
        payload: Option<Value>,
        bearer_token: Option<String>,
    ) -> Self {
        Self {
            method,
            path,
            payload,
            bearer_token,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum ServicePacket {
    Started {
        status: u16,
    },
    /// Base64 keeps binary chunks compact across the JSON IPC boundary.
    Chunk {
        bytes: String,
    },
    End,
    Error {
        code: String,
        message: String,
    },
}

impl ServicePacket {
    pub(crate) fn chunk(bytes: &[u8]) -> Self {
        Self::Chunk {
            bytes: base64::engine::general_purpose::STANDARD.encode(bytes),
        }
    }

    pub(crate) fn error(code: &str, message: &str) -> Self {
        Self::Error {
            code: code.to_owned(),
            message: message.to_owned(),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/core/service_request.rs"]
mod tests;
