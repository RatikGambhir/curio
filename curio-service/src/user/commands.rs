use axum::{Extension, Json};
use axum::extract::{FromRequest, FromRequestParts, Request};
use crate::CurrentUser;

struct Body {
    message: String,
}

pub async fn my_handler(params: Json<Body>) -> String {
    let body = params.0;
    let message = body.message;
    "42".to_string()
}

pub async fn handler(
    // extract the current user, set by the middleware
    Extension(_current_user): Extension<CurrentUser>,
) {
    // ...
}