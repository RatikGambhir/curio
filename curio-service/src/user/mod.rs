pub mod commands;
pub mod queries;

use axum::Router;
use axum::routing::post;
use axum::routing::get;
use commands::my_handler;
use commands::handler;
pub fn gen_command_routes() -> Router {
    Router::new()
        // queries
        .route("/conversations", post(my_handler))
        .route("/conversations/:id", post(handler))

}

pub fn gen_query_routes() -> Router {
    Router::new()
    .route("/conversations/:id", get(queries::get_user_info))
}



