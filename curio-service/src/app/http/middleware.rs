use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::Response,
};
use tower_http::cors::CorsLayer;

use crate::shared::auth::CurrentUser;

/// Exact-origin browser policy. CORS is not authorization.
pub(super) fn apply(router: Router, cors_allowed_origins: &[String]) -> Router {
    let allowed_origins = cors_allowed_origins
        .iter()
        .map(|origin| HeaderValue::from_str(origin).expect("invalid configured CORS origin"))
        .collect::<Vec<_>>();
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    router.layer(cors)
}

/// Requires the development bearer identity on every route of `router`.
pub(crate) fn protected(router: Router) -> Router {
    router.route_layer(middleware::from_fn(auth))
}

async fn auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let current_user = authorize_current_user(auth_header).ok_or(StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(current_user);

    Ok(next.run(request).await)
}

fn authorize_current_user(auth_header: &str) -> Option<CurrentUser> {
    auth_header
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty() && !token.as_bytes().iter().any(u8::is_ascii_whitespace))
        .map(|token| CurrentUser::new(token.to_owned()))
}
