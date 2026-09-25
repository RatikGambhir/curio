use std::sync::Arc;

use axum::Router;

use super::{Ingestion, Jobs, Search, Viewing, ingestion, search, viewing};

pub(crate) fn routes(
    ingestion: Arc<Ingestion>,
    jobs: Arc<Jobs>,
    search: Arc<Search>,
    viewing: Arc<Viewing>,
) -> Router {
    Router::new().nest(
        "/v1/documents",
        ingestion::route::routes(ingestion, jobs)
            .merge(search::route::routes(search))
            .merge(viewing::route::routes(viewing)),
    )
}
