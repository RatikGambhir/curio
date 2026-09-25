use super::*;
use std::sync::Mutex;

type SavedProfile = (String, String, String, Option<String>);

#[derive(Default)]
struct Store {
    saved: Mutex<Vec<SavedProfile>>,
    failure: Option<UserError>,
}
impl UserStore for Store {
    async fn save(&self, profile: UserProfile<'_>) -> Result<UserRecord, UserError> {
        self.saved.lock().unwrap().push((
            profile.id().into(),
            profile.name().into(),
            profile.email().into(),
            profile.avatar_url().map(str::to_owned),
        ));
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        Ok(UserRecord {
            id: profile.id().into(),
            name: profile.name().into(),
            email: profile.email().into(),
            avatar_url: profile.avatar_url().map(str::to_owned),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}
fn input() -> SaveUserRequest {
    SaveUserRequest {
        id: " owner ".into(),
        name: " Ada ".into(),
        email: " ada@example.test ".into(),
        avatar_url: Some("  ".into()),
    }
}
#[tokio::test]
async fn trims_profile_values_and_normalizes_blank_avatar_before_persistence() {
    let service = UserService::new(Store::default());
    let user = service.save_user(input()).await.unwrap();
    assert_eq!(user.name, "Ada");
    assert_eq!(
        *service.repository.saved.lock().unwrap(),
        [(
            "owner".into(),
            "Ada".into(),
            "ada@example.test".into(),
            None
        )]
    );
}
#[tokio::test]
async fn invalid_profile_never_reaches_persistence() {
    let service = UserService::new(Store::default());
    let mut input = input();
    input.name = " ".into();
    assert_eq!(service.save_user(input).await, Err(UserError::Invalid));
    assert!(service.repository.saved.lock().unwrap().is_empty());
}
#[tokio::test]
async fn persistence_conflict_is_preserved_as_a_domain_error() {
    let service = UserService::new(Store {
        failure: Some(UserError::EmailConflict),
        ..Store::default()
    });
    assert_eq!(
        service.save_user(input()).await,
        Err(UserError::EmailConflict)
    );
}
