use curio_service::{app::config::ServiceConfig, app_with_config, shared::diagnostics};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Load .env first so local runs pick up configuration without exporting it.
    let _ = dotenvy::dotenv();
    diagnostics::init();

    let config = match ServiceConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            error!(%error, "invalid curio-service configuration");
            error!("set the required values in .env (see docs/deployment.md)");
            std::process::exit(1);
        }
    };

    let app = match app_with_config(config).await {
        Ok(app) => app,
        Err(_) => {
            error!("curio-service database connection or migration verification failed");
            error!("if migrations are pending, run `cargo run --bin curio_db -- migrate`");
            std::process::exit(1);
        }
    };

    let address = std::env::var("CURIO_SERVICE_ADDR").unwrap_or_else(|_| {
        std::env::var("PORT")
            .map(|port| format!("0.0.0.0:{port}"))
            .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
    });
    let listener = match tokio::net::TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(bind_error) => {
            error!(%bind_error, %address, "failed to bind curio-service");
            std::process::exit(1);
        }
    };

    let bound_address = listener
        .local_addr()
        .map(|address| address.to_string())
        .unwrap_or(address);
    info!(address = %bound_address, "curio-service listening");

    if let Err(server_error) = axum::serve(listener, app).await {
        error!(%server_error, "curio-service stopped unexpectedly");
        std::process::exit(1);
    }
}
