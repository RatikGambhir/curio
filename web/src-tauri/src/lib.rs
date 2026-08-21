mod commands;
mod core;
mod repositories;
mod services;

use commands::{cancel_request, open_external_url, service_request};
use services::AppServices;

const DEVELOPMENT_SERVICE_URL: &str = "http://127.0.0.1:3000";

fn configured_service_url() -> &'static str {
    option_env!("VITE_CURIO_SERVICE_URL").unwrap_or(DEVELOPMENT_SERVICE_URL)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let services = AppServices::new(configured_service_url())
        .expect("the configured Curio service URL must be valid");

    tauri::Builder::default()
        .manage(services)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            service_request,
            cancel_request,
            open_external_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
