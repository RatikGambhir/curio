use axum::{Router, middleware, routing::post};

use crate::{
    database::Database,
    task::{handler, repository::TaskRepository, service::TaskService},
};

pub fn routes(database: Database) -> Router {
    let service = TaskService::new(TaskRepository::new(database));

    Router::new()
        .route(
            "/v1/tasks",
            post(handler::create_task).get(handler::list_tasks),
        )
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(service)
}
