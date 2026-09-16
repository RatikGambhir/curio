use crate::{
    diagnostics,
    user::{
        error::UserError,
        model::{SaveUser, SaveUserRequest, UserRecord},
        repository::UserRepository,
    },
};

#[derive(Clone)]
pub struct UserService {
    repository: UserRepository,
}

impl UserService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub async fn save_user(&self, request: SaveUserRequest) -> Result<UserRecord, UserError> {
        let id = required(&request.id)?;
        let name = required(&request.name)?;
        let email = required(&request.email)?;
        let avatar_url = optional(request.avatar_url.as_deref());

        self.repository
            .save(SaveUser {
                id,
                name,
                email,
                avatar_url,
            })
            .await
            .map_err(storage_error)
    }
}

fn required(value: &str) -> Result<&str, UserError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(UserError::Invalid(
            "A user id, name, and email are required.",
        ));
    }
    Ok(value)
}

fn optional(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn storage_error(error: sqlx::Error) -> UserError {
    if error
        .as_database_error()
        .is_some_and(|error| error.is_unique_violation())
    {
        return UserError::Conflict("That email is already in use by another account.");
    }

    diagnostics::log_database_error("user_save", &error);
    UserError::Internal
}
