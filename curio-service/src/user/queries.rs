use axum::Extension;
use crate::CurrentUser;

pub async fn get_user_info(
    Extension(current_user): Extension<CurrentUser>,
) -> String {
    return 42.to_string();
}