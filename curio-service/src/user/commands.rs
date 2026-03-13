use axum::Extension;
use crate::CurrentUser;

pub async fn my_handler() -> String {
    "42".to_string()
}

pub async fn handler(
    // extract the current user, set by the middleware
    Extension(current_user): Extension<CurrentUser>,
) {
    // ...
}