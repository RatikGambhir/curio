//! Development identity carried from the bearer middleware to handlers.
//!
//! The bearer value is treated as the user ID; it has no signature, expiry, or
//! session validation and is not production authentication.

#[derive(Clone, Debug)]
pub struct CurrentUser {
    id: String,
}

impl CurrentUser {
    pub(crate) fn new(id: String) -> Self {
        Self { id }
    }

    pub(crate) fn owns(&self, user_id: &str) -> bool {
        self.id == user_id.trim()
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }
}
