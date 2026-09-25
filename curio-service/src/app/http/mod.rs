mod middleware;

use axum::Router;

pub(crate) use middleware::protected;

/// Applies cross-cutting HTTP policy to the assembled API.
pub fn create_router(api: Router, cors_allowed_origins: &[String]) -> Router {
    middleware::apply(api, cors_allowed_origins)
}
