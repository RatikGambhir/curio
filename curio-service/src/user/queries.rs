use axum::Extension;
use crate::CurrentUser;

pub async fn get_user_info(
    Extension(_current_user): Extension<CurrentUser>,
) -> String {
    return 42.to_string();
}