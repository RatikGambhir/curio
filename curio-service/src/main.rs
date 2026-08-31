use curio_service::{app_with_config, config::ServiceConfig};

#[tokio::main]
async fn main() {
    // Load .env first so local runs pick up configuration without exporting it.
    let _ = dotenvy::dotenv();

    let config = match ServiceConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("invalid curio-service configuration: {error}");
            eprintln!("copy .env.example to .env and fill in the required values");
            std::process::exit(1);
        }
    };

    let app = app_with_config(config)
        .await
        .expect("failed to initialize curio-service database");

    let address =
        std::env::var("CURIO_SERVICE_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("failed to bind curio-service");

    println!(
        "curio-service listening on {}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app)
        .await
        .expect("curio-service stopped unexpectedly");
}
