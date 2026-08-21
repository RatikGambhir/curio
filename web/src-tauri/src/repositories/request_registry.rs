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

    #[cfg(test)]
    fn len(&self) -> usize {
        self.requests.lock().expect("in-flight lock").len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_removes_every_in_flight_request() {
        let registry = RequestRegistry::default();
        let first = CancellationToken::new();
        let second = CancellationToken::new();
        registry
            .register("first".to_owned(), first.clone())
            .expect("register first");
        registry
            .register("second".to_owned(), second.clone())
            .expect("register second");

        assert_eq!(registry.len(), 2);
        assert!(registry.cancel("first").expect("cancel first"));
        assert!(first.is_cancelled());
        assert_eq!(registry.len(), 1);
        registry.remove("second").expect("remove second");
        assert_eq!(registry.len(), 0);
        assert!(!registry.cancel("missing").expect("cancel missing"));
    }
}
