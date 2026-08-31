use std::{collections::HashMap, sync::Mutex};

use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub(crate) struct RequestRegistry {
    requests: Mutex<HashMap<String, CancellationToken>>,
}

impl RequestRegistry {
    pub(crate) fn register(
        &self,
        request_id: String,
        token: CancellationToken,
    ) -> Result<(), String> {
        let mut requests = self.lock()?;
        if requests.contains_key(&request_id) {
            return Err("A service request with this identifier is already active.".to_owned());
        }
        requests.insert(request_id, token);
        Ok(())
    }

    pub(crate) fn cancel(&self, request_id: &str) -> Result<bool, String> {
        let token = self.lock()?.remove(request_id);
        if let Some(token) = token {
            token.cancel();
            return Ok(true);
        }
        Ok(false)
    }

    pub(crate) fn remove(&self, request_id: &str) -> Result<(), String> {
        self.lock()?.remove(request_id);
        Ok(())
    }

    fn lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashMap<String, CancellationToken>>, String> {
        self.requests
            .lock()
            .map_err(|_| "Desktop service cancellation state is unavailable.".to_owned())
    }

}

#[cfg(test)]
#[path = "../../tests/unit/repositories/request_registry.rs"]
mod tests;
