use curio_service::app;

#[tokio::main]
async fn main() {
    let address =
        std::env::var("CURIO_SERVICE_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("failed to bind curio-service");

    println!(
        "curio-service listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app())
        .await
        .expect("curio-service stopped unexpectedly");
}
