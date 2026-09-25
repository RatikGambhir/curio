//! Profile use cases composed with persistence through a feature-local port.
use super::model::{SaveUserRequest, UserError, UserProfile, UserRecord};

pub trait UserStore: Send + Sync {
    fn save(
        &self,
        profile: UserProfile<'_>,
    ) -> impl Future<Output = Result<UserRecord, UserError>> + Send;
}

#[derive(Clone)]
pub struct UserService<R> {
    repository: R,
}

impl<R: UserStore> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn save_user(&self, input: SaveUserRequest) -> Result<UserRecord, UserError> {
        self.repository.save(UserProfile::parse(&input)?).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domains/users/service.rs"]
mod tests;
